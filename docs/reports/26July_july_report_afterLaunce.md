# Analytics & Download Infrastructure Progress Report

**Date:** 26 July 2026  
**Source:** [`docs/ANALYTICS_TASKS.md`](../ANALYTICS_TASKS.md)  
**Overall status:** **Partially complete** — automatic download delivery is implemented, but the visitor analytics and reporting system is not.

## Executive summary

Ducky now has a solid download-delivery foundation. Every website download action points to a permanent server route, those routes resolve the latest published GitHub release, choose a platform-specific asset, attempt to record the event in Supabase, and return a temporary redirect. No release version or release asset URL remains in the frontend.

The analytics system is not yet complete. Supabase currently stores only the platform and release tag, Vercel Analytics is not installed, there is no private dashboard, and the repository contains no database migration defining the `downloads` table. The current count represents a download redirect/request, not proof that the GitHub asset finished downloading.

## Status against `ANALYTICS_TASKS.md`

| Area | Status | Current state |
| --- | --- | --- |
| Native Next.js/Vercel foundation | **Done** | The website uses `next dev`, `next build`, and `next start`; no Vinext, Vite, Wrangler, or Cloudflare deployment packages remain. |
| Permanent platform routes | **Done** | `GET /download/mac`, `/download/windows`, and `/download/linux` share one server-side handler. |
| Latest GitHub release resolution | **Done** | The server calls GitHub's `/releases/latest` endpoint and caches the result for five minutes. |
| Platform asset selection | **Done** | macOS prefers a universal DMG, Windows prefers a setup/x64 EXE, and Linux prefers an x86-64 AppImage. Only uploaded assets under the expected repository release URL are accepted. |
| Frontend version independence | **Done** | Shared `downloadLinks` point to the three internal routes; frontend components contain no versioned GitHub release asset URLs. |
| 302 download redirect | **Done** | Successful resolution returns a `302` with `Cache-Control: no-store`; GitHub resolution failures return a controlled `502`. |
| Supabase download tracking | **Partial** | A server-only service-role client inserts one `downloads` row with `platform` and `version`. Errors are logged and do not block the redirect. |
| Download analytics fields | **Partial** | Platform and release version are stored. Timestamp depends on the external table's `created_at` default. Browser and referrer are not captured; country is not implemented. |
| Vercel Analytics | **Missing** | `@vercel/analytics` is not installed and the root layout does not render `<Analytics />`. |
| Private `/admin/analytics` dashboard | **Missing** | No route, UI, query service, middleware, or authentication/authorization exists. |
| Visitor/download conversion reporting | **Missing** | There is no common reporting layer combining visitor metrics with Supabase downloads. |
| Database schema as code | **Missing** | No Supabase SQL migration, generated database types, constraints, indexes, or retention policy exists in the repository. |

## Implemented download flow

The current flow is:

1. A platform button uses [`website/lib/siteLinks.ts`](../../website/lib/siteLinks.ts).
2. The matching App Router endpoint delegates to [`routeHandler.ts`](../../website/lib/downloads/routeHandler.ts).
3. [`githubRelease.ts`](../../website/lib/downloads/githubRelease.ts) fetches and validates the latest GitHub release, coalesces concurrent lookups, and applies a five-minute cache.
4. [`downloadTracker.ts`](../../website/lib/downloads/downloadTracker.ts) attempts the Supabase insert.
5. The handler redirects to the selected GitHub asset.

This satisfies the main version-independent architecture. A future published GitHub release should become available without a website code change, subject to the cache window and the release containing the expected `.dmg`, `.exe`, and `.AppImage` assets.

## Supabase tracking status

The Supabase implementation has several good production properties:

- It is marked `server-only`.
- It reads `SUPABASE_URL` and `SUPABASE_SERVICE_ROLE_KEY` only on the server.
- Session persistence, URL session detection, and token refresh are disabled.
- The client is reused within the server process.
- Tracking is isolated behind the existing `DownloadTracker` interface.
- A missing configuration or insert failure is logged without preventing the GitHub redirect.

The repository cannot currently prove that production storage is reproducible. The `downloads` table and its `created_at` default exist outside source control, and the required Supabase environment variables are not documented in the website README. There is also no timeout, retry, queue, or health signal for failed inserts, so a transient Supabase problem can silently lose events and a slow request can delay the redirect.

## Download analytics fields

| Field requested | Status | Notes |
| --- | --- | --- |
| Platform | **Stored** | Values are `mac`, `windows`, or `linux`. |
| Release version | **Stored** | The exact GitHub `tag_name` is inserted as `version`. |
| Timestamp | **Indirect** | The code intentionally omits it and relies on Supabase's `created_at` default; no checked-in migration verifies that default. |
| Browser | **Missing** | The route does not accept or parse the incoming request headers. |
| Referrer | **Missing** | The `Referer` header is not captured. |
| Country | **Missing/optional** | No Vercel geo header or other location source is read. |
| Asset name | **Captured, not stored** | `assetName` is present in the tracker event but excluded from the insert. |

The current event should be described as a **download request/redirect**, not a confirmed completed download. GitHub serves the binary after the redirect, so the website cannot verify transfer completion with this architecture.

## Vercel Analytics

The website is ready to host as a native Next.js application on Vercel, but Vercel Analytics itself has not been integrated. Consequently, visitors, sessions, countries, browsers, devices, referrers, and popular pages are not available from this codebase.

Adding Vercel Analytics will provide the hosted Vercel analytics view, but the custom dashboard requirement needs a separate decision: either query an appropriate Vercel API/data source for visitor metrics, collect first-party visitor events into the analytics database, or keep visitor reporting in Vercel and limit `/admin/analytics` to download data.

## Private analytics dashboard

No `/admin/analytics` implementation exists. Completing it requires:

- server-side aggregate queries for total downloads, platform split, release adoption, and downloads over time;
- a visitor-data source for totals and conversion calculations;
- authentication and authorization that protect both the page and its data endpoints;
- time-zone definitions for “today,” “week,” and “month”;
- empty, loading, and query-failure states;
- indexes supporting time, platform, and release aggregation.

The route being unlinked would not make it private; access control is mandatory before deployment.

## Risks and remaining gaps

- **Schema drift:** production depends on an externally created table with no migration or type contract.
- **Silent data loss:** tracking failures are fail-open by design, but there is no alerting, retry, or failure counter.
- **Latency:** the redirect awaits Supabase and no explicit timeout is applied.
- **GitHub limits:** latest-release requests are unauthenticated; caching reduces traffic, but GitHub API rate limits remain an operational dependency.
- **Cache delay:** a new release can take up to approximately five minutes to appear through the website.
- **Counting semantics:** bots, retries, and repeat clicks are counted; successful file transfer is not measurable.
- **Limited test depth:** tests verify the production page and source wiring, but do not exercise a mocked end-to-end GitHub → Supabase → 302 route flow under native Next.js.
- **No privacy/retention policy:** this becomes necessary before collecting user-agent, referrer, or geographic data.
- **No analytics observability:** there is no health check or dashboard warning when Supabase configuration is absent.

## Recommended next steps

1. **Define the data contract first:** add a Supabase migration for `downloads` with `created_at`, constrained platform values, version, optional browser/referrer/country fields, and indexes for time/platform/version reporting. Document the two server-only Vercel environment variables.
2. Update the shared route handler and tracker to capture the approved request metadata, with a short tracking timeout and clear failure telemetry while preserving fail-open redirects.
3. Add `@vercel/analytics` and `<Analytics />` to establish visitor analytics.
4. Add deterministic route-level tests for asset selection, Supabase success/failure, caching, missing assets, and non-blocking redirects.
5. Build an authenticated `/admin/analytics` dashboard after choosing how visitor data will be made queryable alongside Supabase download data.

The best immediate development step is **checking in the Supabase schema and finalizing the download event contract**. That creates a reliable base for richer tracking, aggregation queries, and the eventual private dashboard.
