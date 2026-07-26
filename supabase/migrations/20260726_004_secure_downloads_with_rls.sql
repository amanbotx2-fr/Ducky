-- Ducky download analytics: secure the downloads table with Row Level Security.
--
-- This migration is additive and does not alter or delete download rows.
-- Reapplying these privilege statements is safe because ALTER, REVOKE, and
-- GRANT converge on the same access state.

begin;

-- 1. Protect the exposed public-schema table with RLS.
--
-- No anon/authenticated policies are created intentionally. With RLS enabled
-- and no matching policy, browser-facing Supabase roles are denied by default.
alter table public.downloads enable row level security;

-- 2. Remove all direct table privileges from browser-facing roles.
--
-- PUBLIC is revoked as well so anon/authenticated cannot inherit table access
-- through PostgreSQL's implicit PUBLIC role.
revoke all privileges on table public.downloads
  from public, anon, authenticated;

-- 3. Preserve the two server-side operations used by Ducky.
--
-- Download tracking requires INSERT, while the security-invoker analytics RPC
-- requires SELECT. Supabase's service_role has BYPASSRLS, so it does not need
-- an RLS policy; explicit table grants are still required by PostgreSQL.
grant select, insert on table public.downloads to service_role;

commit;
