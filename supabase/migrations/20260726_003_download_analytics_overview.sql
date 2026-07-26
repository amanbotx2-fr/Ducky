-- Ducky download analytics: Milestone 3.1 overview query.
--
-- This migration adds one read-only aggregate function for the internal
-- analytics page. It does not alter download rows, tracking, or release data.

begin;

-- 1. Return all overview data in one database call.
--
-- The materialized CTE reads the downloads table once, then reuses that result
-- for summary and release aggregates. Calendar windows are UTC so results are
-- deterministic across Supabase and Vercel regions.
create or replace function public.get_download_analytics_overview()
returns jsonb
language sql
stable
security invoker
set search_path = ''
as $function$
  with download_rows as materialized (
    select
      platform,
      version,
      created_at
    from public.downloads
  ),
  summary as (
    select
      count(*) as total_downloads,
      count(*) filter (
        where created_at >=
          date_trunc('day', now() at time zone 'UTC') at time zone 'UTC'
      ) as downloads_today,
      count(*) filter (
        where created_at >=
          date_trunc('week', now() at time zone 'UTC') at time zone 'UTC'
      ) as downloads_this_week,
      count(*) filter (
        where created_at >=
          date_trunc('month', now() at time zone 'UTC') at time zone 'UTC'
      ) as downloads_this_month,
      count(*) filter (where platform = 'mac') as mac_downloads,
      count(*) filter (where platform = 'windows') as windows_downloads,
      count(*) filter (where platform = 'linux') as linux_downloads
    from download_rows
  ),
  release_counts as (
    select
      coalesce(nullif(btrim(version), ''), 'Unknown') as version,
      count(*) as downloads
    from download_rows
    group by coalesce(nullif(btrim(version), ''), 'Unknown')
  ),
  releases as (
    select coalesce(
      jsonb_agg(
        jsonb_build_object(
          'version', version,
          'downloads', downloads
        )
        order by downloads desc, version
      ),
      '[]'::jsonb
    ) as items
    from release_counts
  )
  select jsonb_build_object(
    'totalDownloads', summary.total_downloads,
    'downloadsToday', summary.downloads_today,
    'downloadsThisWeek', summary.downloads_this_week,
    'downloadsThisMonth', summary.downloads_this_month,
    'platforms', jsonb_build_object(
      'mac', summary.mac_downloads,
      'windows', summary.windows_downloads,
      'linux', summary.linux_downloads
    ),
    'releases', releases.items
  )
  from summary
  cross join releases;
$function$;

-- 2. Keep the aggregate endpoint server-only.
--
-- The dashboard uses SUPABASE_SERVICE_ROLE_KEY from a server component.
-- Browser-facing Supabase roles cannot execute this function.
revoke all on function public.get_download_analytics_overview() from public;
revoke all on function public.get_download_analytics_overview() from anon;
revoke all on function public.get_download_analytics_overview() from authenticated;
grant execute on function public.get_download_analytics_overview() to service_role;

-- 3. Document the function contract in the Supabase catalog.
comment on function public.get_download_analytics_overview() is
  'Returns Ducky download totals, platform counts, and release counts for the internal analytics overview.';

commit;
