-- Ducky download analytics: initial schema baseline.
--
-- This migration describes the table used by the production download tracker.
-- It is intentionally additive so it can bootstrap a new Supabase project or
-- reconcile the existing production table without deleting rows or columns.

begin;

-- 1. Bootstrap the table for new environments.
--
-- On production, where public.downloads already exists, this statement is a
-- no-op. Extra production columns are intentionally preserved.
create table if not exists public.downloads (
  platform text not null,
  version text not null,
  created_at timestamptz not null default now(),
  constraint downloads_platform_check
    check (platform in ('mac', 'windows', 'linux'))
);

-- 2. Reconcile required columns additively.
--
-- ADD COLUMN IF NOT EXISTS avoids changing the type or contents of an existing
-- production column. Missing columns remain nullable initially so legacy rows
-- are never rejected or rewritten during this step.
alter table public.downloads
  add column if not exists platform text,
  add column if not exists version text,
  add column if not exists created_at timestamptz default now();

-- 3. Preserve the server-generated timestamp contract.
--
-- The website omits created_at when inserting, so PostgreSQL must continue to
-- assign the download timestamp.
alter table public.downloads
  alter column created_at set default now();

-- 4. Enforce the supported platform values without deleting legacy data.
--
-- NOT VALID avoids rejecting the migration if unexpected historical rows are
-- present, while PostgreSQL still enforces the constraint for new or changed
-- rows. If all existing rows conform, the constraint is validated and the
-- three required columns are promoted to NOT NULL.
do $migration$
begin
  if not exists (
    select 1
    from pg_constraint
    where conrelid = 'public.downloads'::regclass
      and conname = 'downloads_platform_check'
  ) then
    alter table public.downloads
      add constraint downloads_platform_check
      check (
        platform is not null
        and platform in ('mac', 'windows', 'linux')
      )
      not valid;
  end if;

  if not exists (
    select 1
    from public.downloads
    where platform is null
      or platform not in ('mac', 'windows', 'linux')
  ) then
    alter table public.downloads
      validate constraint downloads_platform_check;
    alter table public.downloads
      alter column platform set not null;
  end if;

  if not exists (
    select 1
    from public.downloads
    where version is null
  ) then
    alter table public.downloads
      alter column version set not null;
  end if;

  if not exists (
    select 1
    from public.downloads
    where created_at is null
  ) then
    alter table public.downloads
      alter column created_at set not null;
  end if;
end
$migration$;

-- 5. Add indexes used by download totals, trends, platform splits, and release
-- adoption queries. IF NOT EXISTS makes repeated application harmless.
create index if not exists downloads_created_at_idx
  on public.downloads (created_at desc);

create index if not exists downloads_platform_created_at_idx
  on public.downloads (platform, created_at desc);

create index if not exists downloads_version_created_at_idx
  on public.downloads (version, created_at desc);

-- 6. Document the database contract in the Supabase catalog.
comment on table public.downloads is
  'Records Ducky website download redirects before users reach GitHub assets.';

comment on column public.downloads.platform is
  'Requested download platform: mac, windows, or linux.';

comment on column public.downloads.version is
  'GitHub release tag resolved when the download redirect was requested.';

comment on column public.downloads.created_at is
  'Server-generated timestamp for the download redirect request.';

commit;
