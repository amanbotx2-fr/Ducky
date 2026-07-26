-- Ducky download analytics: optional request metadata.
--
-- This migration is additive and keeps all existing download rows valid.
-- Every new field is nullable because request metadata is best-effort and may
-- not be available outside Vercel or from privacy-conscious clients.

begin;

-- 1. Add the Phase 1 analytics fields without rewriting existing events.
alter table public.downloads
  add column if not exists browser text,
  add column if not exists operating_system text,
  add column if not exists referrer text,
  add column if not exists country text,
  add column if not exists asset_name text;

-- 2. Constrain country values to the two-letter code supplied by Vercel.
--
-- NOT VALID preserves compatibility if this migration is applied to a table
-- that already contains custom metadata. New and updated rows are still
-- checked, while NULL remains valid when no reliable country header exists.
do $migration$
begin
  if not exists (
    select 1
    from pg_constraint
    where conrelid = 'public.downloads'::regclass
      and conname = 'downloads_country_code_check'
  ) then
    alter table public.downloads
      add constraint downloads_country_code_check
      check (country is null or country ~ '^[A-Z]{2}$')
      not valid;
  end if;
end
$migration$;

-- 3. Document the privacy-conscious metadata contract.
comment on column public.downloads.browser is
  'Coarse browser family inferred from User-Agent; NULL when unknown.';

comment on column public.downloads.operating_system is
  'Coarse operating-system family inferred from User-Agent; NULL when unknown.';

comment on column public.downloads.referrer is
  'Referring HTTP(S) hostname only; paths, queries, and fragments are discarded.';

comment on column public.downloads.country is
  'Uppercase ISO 3166-1 alpha-2 code from Vercel; NULL when unavailable.';

comment on column public.downloads.asset_name is
  'GitHub release asset selected for the download redirect.';

commit;
