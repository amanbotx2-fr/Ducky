# Ducky Tauri v2 Migration Progress

**Last updated:** 27 July 2026

## Current Status

- Active phase: Phase 4 — Tray + Native Menu Migration
- Last completed task: Task 4.3 — Migrate Tray Icon
- Next task: Task 4.4 — Migrate Static Native Menus
- Blockers: None. The revised Phase 4 contract limits this phase to native
  lifecycle, static menus/actions, and the renderer-requested context-menu
  transport; feature-domain menu state remains deferred.

## Completed Tasks

### Task 0.1 — Verify repository state

**Status:** Complete

**Files changed**

- `docs/migrating/progress.md` — created the migration execution log.

**Validation performed**

- Initial working tree: clean and synchronized with `origin/main`.
- `npm ci`: passed; the lockfile was unchanged.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Baseline observations**

- npm reports 16 high-severity advisories in the existing dependency tree.
  Dependency remediation is outside Task 0.1 and no package versions were
  changed.
- `docs/migrating/migration_tasks.md` provides executable tasks through Phase
  3. Phases 4–11 currently contain literal `...` placeholders. This does not
  block Phase 0, but no undefined task will be inferred when that boundary is
  reached.

**Blockers**

- None.

**Next task**

- Task 0.2 — Create DesktopBridge abstraction.

### Task 0.2 — Create DesktopBridge abstraction

**Status:** Complete

**Files changed**

- `src/desktop/contracts.ts` — added the runtime-neutral bridge contract.
- `src/desktop/electronBridge.ts` — adapted the existing role-specific preload
  APIs without importing Electron into renderer code.
- `src/desktop/DesktopBridge.ts` — added the renderer-facing integration
  boundary.
- `tsconfig.renderer.json` — included the new desktop bridge modules in
  renderer type-checking.
- `docs/migrating/progress.md` — recorded Task 0.2.

**Validation performed**

- Confirmed `src/renderer/` and `src/desktop/` contain no direct imports from
  the `electron` package.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Blockers**

- None.

**Next task**

- Task 0.3 — Move renderer to DesktopBridge.

### Task 0.3 — Move renderer to DesktopBridge

**Status:** Complete

**Files changed**

- `src/renderer/App.tsx` — routed companion orchestration through
  `DesktopBridge`.
- `src/renderer/PreferencesApp.tsx` — routed AI Preferences operations
  through `DesktopBridge`.
- `src/renderer/components/PsyDuck.tsx` — routed cursor, context-menu, and
  drag operations through `DesktopBridge`.
- `src/renderer/hooks/usePomodoroState.ts` — routed Pomodoro snapshots and
  subscriptions through `DesktopBridge`.
- `src/renderer/hooks/usePreferencesSettings.ts` — routed Preferences
  snapshots and mutations through `DesktopBridge`.
- `src/renderer/hooks/useReminderNotifications.ts` — routed reminder events
  and snooze creation through `DesktopBridge`.
- `src/renderer/hooks/useRuntimeSettings.ts` — routed runtime settings through
  `DesktopBridge`.
- `src/renderer/hooks/useUpdateStatus.ts` — routed updater operations through
  `DesktopBridge`.
- `src/desktop/electron-globals.d.ts` — moved Electron preload global types
  behind the Electron adapter boundary.
- `src/renderer/vite-env.d.ts` — retained only Vite's renderer declaration.
- `docs/migrating/progress.md` — recorded Task 0.3 and Phase 0 completion.

**Validation performed**

- Confirmed there are no `window.psyduck` or
  `window.psyduckPreferences` references under `src/renderer/`.
- Confirmed `src/renderer/` contains no direct Electron or Tauri imports.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.
- Packaged-output Electron smoke launch: the companion started without bridge
  errors; the process was then stopped manually.

**Phase 0 exit criteria**

- Renderer code no longer depends directly on Electron APIs.
- Electron remains the active runtime through the isolated adapter.
- Existing behavior and build paths remain intact.

**Blockers**

- None.

**Next task**

- Task 1.1 — Initialize Tauri v2.

### Task 1.1 — Initialize Tauri v2

**Status:** Complete

**Files changed**

- `package.json` — added the Tauri v2 API/CLI dependencies and additive
  `tauri`, `tauri:dev`, and `tauri:build` scripts while preserving every
  Electron script.
- `package-lock.json` — locked the Tauri JavaScript dependencies.
- `src-tauri/Cargo.toml` — created the Rust package with Ducky's existing
  version and product metadata.
- `src-tauri/Cargo.lock` — locked the initial Rust dependency graph.
- `src-tauri/build.rs` — added the standard Tauri build entry point.
- `src-tauri/tauri.conf.json` — connected Tauri to the existing Vite dev
  server and renderer output without changing `vite.config.ts`.
- `src-tauri/src/lib.rs` and `src-tauri/src/main.rs` — added the minimal Tauri
  application entry points.
- `src-tauri/icons/` — generated Tauri platform icon variants from Ducky's
  existing `assets/icons/icon.png`.
- `src-tauri/capabilities/default.json` — retained the generated temporary
  single-window capability; Task 1.3 will replace it with separate,
  least-privilege companion and Preferences capabilities.
- `src-tauri/.gitignore` — excluded generated Rust target and Tauri schema
  output.
- `docs/migrating/progress.md` — recorded Task 1.1.

**Validation performed**

- Installed the stable Rust toolchain required by Tauri:
  `rustc 1.97.1` / `cargo 1.97.1`.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run dev:renderer`: Vite started on `127.0.0.1:5187`; both
  `index.html` and `preferences.html` returned successfully.
- `npm run tauri:dev`: Tauri compiled, launched the development binary, and
  used the existing Vite server successfully; it was then stopped manually.
- Confirmed no existing Electron source or configuration file was modified.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Environment note**

- The first Rust link attempt exhausted the machine's remaining disk space.
  Only generated project artifacts and the rebuildable npm cache were cleared;
  the subsequent Cargo build passed. This is not a repository blocker.

**Blockers**

- None.

**Next task**

- Task 1.2 — Configure two windows.

### Task 1.2 — Configure two windows

**Status:** Complete

**Files changed**

- `src-tauri/tauri.conf.json` — replaced the generated unnamed window with
  named `companion` and `preferences` windows, connected to the existing
  `index.html` and `preferences.html` renderer entries. The dimensions and
  minimum Preferences size use the existing Electron values; Phase 2 remains
  responsible for companion visual and native behavior parity.
- `docs/migrating/progress.md` — recorded Task 1.2.

**Validation performed**

- Parsed `tauri.conf.json` and asserted the exact `companion`/`index.html` and
  `preferences`/`preferences.html` label-route pairs.
- `npm run tauri:dev`: Tauri consumed the two-window configuration, launched
  successfully, and remained running without renderer-load errors; it was
  then stopped manually.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Blockers**

- None.

**Next task**

- Task 1.3 — Capabilities.

### Task 1.3 — Capabilities

**Status:** Complete

**Files changed**

- `src-tauri/capabilities/companion.json` — added an exact-label,
  local-only capability for the companion window.
- `src-tauri/capabilities/preferences.json` — added an exact-label,
  local-only capability for the Preferences window.
- `src-tauri/capabilities/default.json` — removed the generated shared
  capability.
- `src-tauri/tauri.conf.json` — explicitly enabled only the two named
  capabilities.
- `docs/migrating/progress.md` — recorded Task 1.3 and Phase 1 completion.

**Security boundary**

- Each window appears in exactly one capability.
- Neither capability uses a wildcard label or remote URL.
- No filesystem, HTTP, shell, process, or other plugin permission is granted.
- The permission arrays are intentionally empty until later tasks introduce a
  proven renderer API requirement.

**Validation performed**

- Parsed both capability files and asserted their identifiers, exact window
  labels, empty permission sets, and absence of remote configuration.
- Confirmed no wildcard or filesystem/HTTP/shell/process permission appears in
  `src-tauri/capabilities/`.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed, including Tauri
  capability schema validation.
- `npm run tauri:dev`: passed; the Tauri shell launched under the new
  capability boundary and was then stopped manually.
- `cargo fmt --check`: passed.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed, confirming Electron remains buildable.

**Phase 1 exit criteria**

- Tauri launches successfully with both renderer windows.
- The two renderer roles have separate deny-by-default capabilities.
- Electron still type-checks, passes all tests, and builds.

**Blockers**

- None.

**Next task**

- Task 2.1 — Companion window.

### Task 2.1 — Companion window

**Status:** Complete

**Files changed**

- `src-tauri/src/desktop/mod.rs` — added the native desktop module boundary
  from the approved migration architecture.
- `src-tauri/src/desktop/windows/mod.rs` — added the window module boundary.
- `src-tauri/src/desktop/windows/companion.rs` — created the named companion
  WebviewWindow from Rust with the existing `220 × 220` logical content size,
  `index.html` renderer, Ducky title, frameless chrome, fixed size, disabled
  maximize/minimize/fullscreen startup state, and taskbar exclusion.
- `src-tauri/src/lib.rs` — creates the companion during Tauri setup.
- `src-tauri/tauri.conf.json` — leaves the Preferences window declarative and
  removes the duplicate declarative companion definition.
- `docs/migrating/progress.md` — recorded Task 2.1.

**Validation performed**

- `cargo fmt`: applied.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run tauri:dev`: passed; the programmatic companion builder completed
  and the Tauri application remained running until stopped manually.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Scope boundary**

- Transparency, shadow removal, always-on-top behavior, work-area positioning,
  dragging, dynamic height, and cursor streaming remain intentionally deferred
  to Tasks 2.2–2.7.

**Blockers**

- None.

**Next task**

- Task 2.2 — Transparent window.

### Task 2.2 — Transparent window

**Status:** Complete

**Files changed**

- `src-tauri/src/desktop/windows/companion.rs` — enabled native window and
  WebView transparency and disabled the operating-system window shadow.
- `src-tauri/Cargo.toml` — enabled Tauri's required
  `macos-private-api` feature for transparent WKWebView backgrounds.
- `src-tauri/tauri.conf.json` — enabled `app.macOSPrivateApi`, which Tauri
  requires for transparent windows on macOS.
- `docs/migrating/progress.md` — recorded Task 2.2.

**Validation performed**

- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri build --debug --bundles app --no-sign`: passed and produced a
  debug `Ducky.app` bundle.
- `npm run tauri:dev`: passed; the transparent companion launched without a
  native runtime error and was then stopped manually.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Platform note**

- Tauri's documented transparent-background implementation uses a macOS
  private API and is incompatible with Mac App Store submission. Ducky's
  current GitHub distribution target is unaffected.
- Native screenshot inspection was attempted against the generated debug app,
  but the Mac was locked. Visual comparison remains an explicit Phase 2 exit
  gate; code and runtime acceptance for Task 2.2 are complete.

**Blockers**

- None for Task 2.3.

**Next task**

- Task 2.3 — Always on top.

### Task 2.3 — Always on top

**Status:** Complete

**Files changed**

- `src-tauri/src/desktop/windows/companion.rs` — enabled the existing default
  always-on-top behavior and visibility across workspaces/virtual desktops.
  On macOS, Tauri/Tao applies `CanJoinAllSpaces` without
  `FullScreenAuxiliary`, matching Electron's
  `visibleOnFullScreen: false` intent.
- `docs/migrating/progress.md` — recorded Task 2.3.

**Validation performed**

- `cargo fmt`: applied.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run tauri:dev`: passed; the companion launched with the native window
  flags and was then stopped manually.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Scope boundary**

- The initial value remains `true`, matching current Electron defaults.
  Settings-driven updates will be wired when the corresponding IPC/bridge
  command is migrated; Task 2.3 does not duplicate settings persistence.

**Blockers**

- None.

**Next task**

- Task 2.4 — Positioning.

### Task 2.4 — Positioning

**Status:** Complete

**Files changed**

- `src-tauri/src/desktop/windows/companion.rs` — placed the companion at the
  bottom-right of the primary display work area with the existing 24-logical-
  pixel margin. The implementation converts the margin through the monitor
  scale factor, preserves negative virtual-desktop coordinates, clamps the
  initial height to the usable display height, and keeps the window hidden
  until positioning is complete.
- `docs/migrating/progress.md` — recorded Task 2.4.

**Validation performed**

- Added four focused Rust tests for standard placement, negative monitor
  coordinates, Retina scaling, and the existing Electron small-work-area
  formula.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (4 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run tauri:dev`: passed; primary-monitor lookup, initial sizing,
  positioning, and window display completed without a native runtime error.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Scope boundary**

- This task covers initial placement only. Drag-time positioning and dynamic
  height anchoring remain Tasks 2.5 and 2.6.
- Native visual comparison remains part of the Phase 2 exit gate because the
  current Mac session is locked.

**Blockers**

- None.

**Next task**

- Task 2.5 — Dragging.

### Task 2.5 — Dragging

**Status:** Complete

**Files changed**

- `src-tauri/src/commands/companion.rs` and
  `src-tauri/src/commands/mod.rs` — added the scoped
  `move_companion_window` command, caller-label authorization, finite/bounded
  coordinate validation, Electron-compatible rounding, and focused tests.
- `src-tauri/src/desktop/windows/companion.rs` — added the native logical
  position operation used by the command.
- `src-tauri/build.rs`,
  `src-tauri/permissions/autogenerated/move_companion_window.toml`, and
  `src-tauri/capabilities/companion.json` — generated an application-command
  permission and granted it only to the exact `companion` window. The
  `preferences` capability remains empty.
- `src-tauri/src/lib.rs` — registered the movement command.
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` — added the direct Serde
  derive dependency required for validated command DTOs and sanitized errors.
- `src/shared/types.ts`, `src/desktop/contracts.ts`,
  `src/desktop/electronBridge.ts`, `src/desktop/tauriBridge.ts`, and
  `src/desktop/DesktopBridge.ts` — added a narrow companion-window bridge and
  selected the real runtime adapter without exposing Tauri APIs to React.
  Unmigrated Tauri domain bridges remain unavailable instead of using
  placeholders.
- `src/renderer/components/PsyDuck.tsx` — routed the existing
  `DragController` through the companion-window bridge. The gesture,
  click-versus-drag tolerance, animation pause, and eye-tracking behavior were
  not changed.
- `docs/migrating/progress.md` — recorded Task 2.5.

**Validation performed**

- Added two Rust command tests covering Electron-compatible rounding and
  rejection of non-finite/out-of-range coordinates.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (6 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission ls`: confirmed generated allow/deny permissions for
  `move_companion_window`.
- Generated capabilities confirmed that only `companion` receives
  `allow-move-companion-window`; `preferences` receives no permission.
- `npm run tauri:dev`: passed with the Tauri adapter and command manifest
  loaded; no native or authority startup error occurred.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.
- Electron packaged-output smoke launch: passed with the existing preload
  bridge selected and no bridge errors; the process was then stopped
  manually.

**Scope boundary**

- Only the native window movement capability moved to the Tauri adapter.
  Full companion domain IPC remains on the later Phase 3 migration path.
- Hands-on pointer movement remains part of the Phase 2 visual/input exit
  gate because the current Mac session is locked.

**Blockers**

- None.

**Next task**

- Task 2.6 — Dynamic height.

### Task 2.6 — Dynamic height

**Status:** Complete

**Files changed**

- `src-tauri/src/commands/companion.rs` — added the
  `set_companion_content_height` command with exact caller-label
  authorization and the existing finite, positive, 10,000-pixel payload
  bound.
- `src-tauri/src/desktop/windows/companion.rs` — implemented current-monitor
  resizing in physical coordinates while consuming renderer height in
  logical pixels. The companion retains its width and bottom edge, grows
  upward, keeps the existing 220-pixel minimum, and clamps to the matching
  work area.
- `src-tauri/build.rs`,
  `src-tauri/permissions/autogenerated/set_companion_content_height.toml`,
  `src-tauri/capabilities/companion.json`, and `src-tauri/src/lib.rs` —
  registered the command and granted its generated permission only to the
  `companion` window.
- `src/shared/types.ts` and `src/desktop/tauriBridge.ts` — extended the
  migrated companion-window bridge with the real Tauri resize command.
- `src/renderer/App.tsx` — routed the existing `ResizeObserver` height reports
  through the companion-window bridge without changing measurement or widget
  behavior.
- `docs/migrating/progress.md` — recorded Task 2.6.

**Validation performed**

- Added one command validation test and three window-bounds tests covering
  upward growth/bottom anchoring, monitor/work-area clamping with Retina
  scaling and negative coordinates, and minimum-height restoration.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (10 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission ls`: confirmed generated allow/deny permissions for
  `set_companion_content_height`.
- Generated capabilities confirmed that only `companion` receives the resize
  permission; `preferences` remains denied.
- `npm run tauri:dev`: passed. The mounted companion reported its initial
  content height through the migrated bridge with no command/authority error.
  Tauri's development custom-protocol transport logged its supported fallback
  to `postMessage`.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Scope boundary**

- Native window APIs do not expose Electron's single atomic `setBounds`
  operation, so Tauri applies size and position consecutively. The computed
  final bounds match Electron; visual transition QA remains part of the Phase
  2 platform exit gate.

**Blockers**

- None.

**Next task**

- Task 2.7 — Cursor channel.

### Task 2.7 — Cursor channel

**Status:** Complete

**Files changed**

- `src-tauri/src/commands/companion.rs` — added an authorized cursor snapshot
  command and an ordered Tauri `Channel<ScreenPoint>` stream. The stream
  samples at 30 Hz, emits only changed positions, supersedes older renderer
  subscriptions by generation, stops when its channel/window becomes
  unavailable, converts physical coordinates to the companion window's
  logical scale, and preserves negative virtual-desktop coordinates.
- `src-tauri/build.rs`,
  `src-tauri/permissions/autogenerated/get_cursor_position.toml`,
  `src-tauri/permissions/autogenerated/stream_cursor_positions.toml`,
  `src-tauri/capabilities/companion.json`, and `src-tauri/src/lib.rs` —
  registered both commands, managed the stream generation state, and granted
  the generated permissions only to the exact `companion` window.
  `preferences` remains denied.
- `src/shared/types.ts` — completed the narrow companion-window bridge
  contract with cursor snapshot and subscription operations.
- `src/desktop/tauriBridge.ts` — adapted the Tauri channel to the existing
  renderer listener contract, retained the latest position for initial
  snapshots, and kept channel construction lazy so Electron never initializes
  Tauri runtime objects.
- `src/renderer/components/PsyDuck.tsx` — moved the existing `EyeTracker` to
  the narrow companion-window bridge without changing its smoothing, eye
  origin, or animation behavior.
- `docs/migrating/progress.md` — recorded Task 2.7 and the Phase 2 exit gate.

**Validation performed**

- Added two focused Rust tests for Retina/negative-coordinate conversion and
  deterministic supersession of an older stream generation.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (12 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission ls`: confirmed generated allow/deny permissions for
  both cursor commands.
- Capability inspection confirmed that only `companion` receives
  `allow-get-cursor-position` and `allow-stream-cursor-positions`;
  `preferences` remains empty.
- `npm run tauri:dev`: passed. The renderer established its eye-tracking
  subscription and remained running without a cursor-command, channel, or
  authority error. Tauri's development custom-protocol transport used its
  supported `postMessage` fallback.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.
- Electron smoke launch: passed with the legacy preload adapter selected and
  no bridge errors; the process was then stopped manually.

**Scope boundary**

- Task 2.7 migrates the cursor transport needed by the existing eye tracker.
  The remaining companion and Preferences domain IPC stays on the Phase 3
  path.
- A newer subscription intentionally supersedes the prior native sampling
  loop, preventing duplicate streams after a WebView reload while retaining
  ordered delivery.

**Blockers**

- None for Task 2.7 itself.

**Next task**

- Phase 2 exit validation.

### Phase 2 validation fix — Startup window lifecycle

**Status:** Complete

**Root cause**

- `src-tauri/tauri.conf.json` declaratively created the Preferences window
  without `visible: false`, so Tauri opened and focused it automatically.
- The companion was created successfully, its renderer mounted, and the
  native window reported visible. However, its hidden-window physical
  `set_position` call during `setup` did not apply correctly on macOS.
  On the Retina primary display, the companion remained centered at physical
  `(1490, 468)` instead of bottom-right `(2932, 1584)`.
- The auto-visible Preferences window occupied `(1070, 252)` at
  `1280 × 1300`, completely covering the centered `440 × 440` companion.
  This made the companion appear absent even though creation and rendering
  both succeeded.

**Files changed**

- `src-tauri/tauri.conf.json` — made the configured Preferences window hidden
  on startup. It remains available for the explicit open flow that will be
  connected during native menu/tray IPC migration.
- `src-tauri/src/desktop/windows/companion.rs` — computes initial placement
  in Tauri builder logical coordinates, including Retina and negative monitor
  origins. The companion remains hidden until the first
  `PageLoadEvent::Finished`, then a one-shot callback shows it. Reloads cannot
  unexpectedly reopen it, and a native show failure is logged.
- `docs/migrating/progress.md` — recorded the diagnosis, fix, and validation.

**Validation performed**

- `npm run tauri:dev`: passed with the final implementation.
- Native runtime diagnostics confirmed the companion starts visible at
  physical `(2932, 1584)` with size `440 × 440` on the 2× primary display.
- After renderer content measurement, the companion settled at
  `(2932, 1480)` with size `440 × 544`, preserving the same bottom edge.
- A native desktop capture confirmed that the transparent companion is
  visible at the bottom-right and Preferences is not present.
- Added/updated focused Rust coverage for standard, negative-coordinate,
  Retina, and constrained-work-area initial placement.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (12 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- Parsed `tauri.conf.json` and asserted that `preferences.visible` is exactly
  `false`.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.

**Blockers**

- None for the startup-window issue.

**Next task**

- Continue the remaining Phase 2 visual/input matrix.

## Phase 2 Exit Gate

**Status:** Complete

The project owner confirmed on 27 July 2026 that Phase 2 and its manual
validation gate are complete. The earlier workspace limitation and the checks
that were outstanding at that time remain below as historical context; no
additional validation results were produced in this workspace.

All seven implementation tasks are complete, but the source-of-truth exit
criterion is: “Behavior visually matches Electron.” The architecture report
also classifies transparency/drag parity as requiring a physical three-OS
visual/input test matrix and calls out mixed-DPI/negative-coordinate testing.

The following hands-on checks remain required:

- transparency, shadow removal, always-on-top behavior, macOS Spaces, and
  full-screen interaction;
- initial positioning and dynamic-height bottom anchoring without visible
  jitter;
- drag movement, click-versus-drag behavior, and bounds across monitor edges;
- cursor/eye tracking on normal DPI, Retina/mixed DPI, and negative-coordinate
  monitor arrangements;
- equivalent behavior on macOS, Windows, and Linux system WebViews.

The macOS startup path now passes visual validation: Preferences stays hidden,
the transparent companion appears at the expected bottom-right location, and
dynamic height remains bottom-anchored. The remaining macOS interaction cases
and equivalent Windows/Linux checks still require hands-on validation. No
Windows or Linux validation host is available in this workspace, and unit,
build, capability, and launch checks cannot establish cross-platform visual
parity on their own.

At the time of the original gate review, Phase 3 did not start because these
checks could not be completed in the workspace. The project owner's later
Phase 2 completion confirmation supersedes that temporary execution blocker.

## Phase 3 Tasks

### Task 3.1 — Commands

**Status:** Complete

**Scope clarification**

- The project owner clarified that Phase 3 migrates IPC infrastructure only.
- Task 3.1 includes the Tauri command manifest, registration, dispatch, and
  only commands required by Phases 1–3.
- Menus, settings, credentials, reminders, Pomodoro, AI, updater, and their
  commands remain assigned to their later phases.
- Existing Electron IPC handlers remain authoritative and unchanged.

**Files changed**

- `src-tauri/src/commands/manifest.rs` — created the reviewed Phase 1–3
  command manifest and added a uniqueness/exact-scope test.
- `src-tauri/src/commands/mod.rs` — centralized Tauri command state and
  handler registration.
- `src-tauri/src/lib.rs` — delegates command setup to the command registry
  instead of maintaining a second inline handler list.
- `src-tauri/build.rs` — builds Tauri application-command permissions from
  the shared Rust command manifest.
- `src/desktop/tauriCommands.ts` — added the typed renderer-side command
  registry and dispatch boundary.
- `src/desktop/tauriBridge.ts` — routes the migrated companion operations
  through typed dispatch instead of raw `invoke` strings.
- `docs/migrating/progress.md` — recorded the accepted scope clarification
  and Task 3.1 completion.

**Command surface**

- `get_cursor_position`
- `move_companion_window`
- `set_companion_content_height`
- `stream_cursor_positions`

No later-phase commands were registered. Existing exact-label capabilities
and command-level label checks remain in force. Task 3.2 events were not
implemented or modified.

**Validation performed**

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (13 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission ls`: confirmed allow/deny permissions for exactly the
  four Phase 1–3 commands.
- `npm run typecheck`: passed.
- `npm test`: passed (118 tests across 33 suites).
- `npm run build`: passed.
- `npm run tauri:dev`: passed. Tauri loaded the centralized application
  command manifest, launched the companion, and dispatched its cursor/resize
  operations without command or authority errors. The development transport
  used Tauri's supported `postMessage` fallback.
- Electron smoke launch: passed with the existing preload bridge and IPC
  handlers intact; the process was then stopped manually.

**Blockers**

- None.

**Next task**

- Task 3.2 — Events.

### Task 3.2 — Events

**Status:** Complete

**Scope clarification**

- Phase 3 remains limited to IPC infrastructure.
- Task 3.2 establishes typed, targeted low-frequency event dispatch and
  subscription without producing or consuming feature-domain events.
- Settings snapshots, one-shot recovery queues, reminder delivery, Pomodoro
  state, updater status, and native menu requests remain owned by their later
  feature phases.
- Cursor samples remain on the existing ordered Tauri channel and were
  intentionally excluded from this task.
- Existing Electron preload listeners, early-event buffers, and main-process
  event delivery remain authoritative and unchanged.

**Files changed**

- `src-tauri/src/events.rs` — added the Rust low-frequency event registry,
  exact companion/Preferences routing, targeted `WebviewWindow` emission, and
  focused route/uniqueness tests.
- `src-tauri/src/lib.rs` — compiles and tests the event infrastructure while
  later feature phases remain responsible for connecting producers.
- `src/desktop/tauriEvents.ts` — added the typed renderer event registry and
  exact-label subscription helper with race-safe asynchronous cleanup.
- `src-tauri/capabilities/companion.json` — granted only event listen and
  unlisten permissions to the companion.
- `src-tauri/capabilities/preferences.json` — granted only event listen and
  unlisten permissions to Preferences.
- `tests/tauri-event-capabilities.test.cjs` — verifies both renderers can
  subscribe but cannot emit events.
- `docs/migrating/progress.md` — recorded Task 3.2 completion.

**Event infrastructure**

- Preserves the 11 existing low-frequency backend event names.
- Routes companion-only events only to `companion`.
- Routes update status only to `preferences`.
- Routes secret-free runtime settings changes to both renderer labels.
- Uses the exact `WebviewWindow` event target rather than global broadcasts.
- Uses `null` for no-payload events, matching Tauri's serialized Rust unit
  payload.
- Rejects invalid renderer/event pairings in the TypeScript adapter.
- Handles unmount-before-registration without leaking Tauri listeners.
- Grants no renderer-side `emit`, `emit-to`, or default event permission.

No feature-domain producer, bridge listener, recovery queue, cursor transport,
or Electron event implementation changed. Task 3.3 was not started.

**Validation performed**

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (15 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run typecheck`: passed.
- `npm test`: passed (120 tests across 34 suites).
- `npm run build`: passed.
- `npx tauri permission ls`: confirmed the narrow event permission set.
- `npm run tauri:dev`: passed. Both capabilities loaded without event
  authority or runtime errors; the companion remained operational.
- Electron smoke launch: passed with the existing preload listeners and IPC
  delivery intact; the process was then stopped manually.

**Blockers**

- None.

**Next task**

- Task 3.3 — Cursor streaming.

### Task 3.3 — Cursor streaming

**Status:** Complete

**Phase 2 relationship**

- Task 2.7 already implemented the authoritative native cursor sampler,
  ordered `Channel<ScreenPoint>`, changed-position filtering, generation-based
  supersession, and physical-to-logical coordinate conversion.
- Task 3.3 reuses that implementation rather than introducing another cursor
  source or renderer event. It completes the IPC transport lifecycle around
  the existing channel.
- Existing TypeScript eye smoothing and CSS rendering remain unchanged.
- Existing Electron sampling, preload subscription, and IPC delivery remain
  authoritative and unchanged.

**Files changed**

- `src-tauri/src/commands/companion.rs` — added an authorized stop command
  that invalidates the active stream generation, plus focused lifecycle
  coverage.
- `src-tauri/src/commands/mod.rs` and
  `src-tauri/src/commands/manifest.rs` — registered the stop operation in the
  existing Phase 1–3 command infrastructure.
- `src-tauri/permissions/autogenerated/stop_cursor_positions.toml` and
  `src-tauri/capabilities/companion.json` — generated and granted the narrow
  companion-only stop permission.
- `src/desktop/tauriCommands.ts` — added typed dispatch for the stop
  operation.
- `src/desktop/tauriCursorStream.ts` — isolated the reused Phase 2 channel
  behind a single renderer transport. It serializes lifecycle changes, starts
  one native stream for the first subscriber, ignores stale channel
  callbacks, shares the latest position, and stops sampling after the final
  subscriber disconnects.
- `src/desktop/tauriBridge.ts` — delegates cursor snapshot/subscription
  operations to the dedicated transport without changing the
  `CompanionWindowBridge` contract.
- `docs/migrating/progress.md` — recorded Task 3.3 completion.

**Validation performed**

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (16 tests),
  including active-generation invalidation on stream stop.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission ls`: confirmed generated allow/deny entries for the
  stop command alongside the existing cursor commands.
- `npm run typecheck`: passed.
- `npm test`: passed (120 tests across 34 suites).
- `npm run build`: passed.
- `npm run tauri:dev`: passed. Tauri launched the companion, established the
  cursor channel, and reported no command or capability errors; the process
  was then stopped manually.
- Electron smoke launch: passed with the existing main-process sampler,
  preload listener, and renderer eye-tracking integration intact; the process
  was then stopped manually.

**Scope boundary**

- Task 3.3 changes only cursor streaming IPC transport and lifecycle.
- No low-frequency event domain, feature command, Electron implementation, or
  Task 3.4 authorization work was modified.

**Blockers**

- None.

**Next task**

- Task 3.4 — Authorization.

### Task 3.4 — Authorization

**Status:** Complete

**Authorization model**

- The application command manifest and command-to-renderer role assignments
  now share one typed Rust source of truth.
- Every completed Phase 1–3 command is authorized only for the exact
  `companion` renderer role.
- Each command still accepts the invoking `WebviewWindow` and rejects an
  unexpected or unknown label as defense in depth.
- Tauri capabilities now explicitly apply only to local content and the exact
  `companion` or `preferences` webview label. They grant no wildcard window,
  remote origin, broad core default, renderer event emission, filesystem,
  HTTP, shell, process, clipboard, or global-shortcut authority.
- Backend events reuse the same renderer-role identities and remain emitted
  only to their approved exact targets.
- Existing Electron role/channel/URL/subframe authorization and permission
  denial policy remain unchanged.

**Files changed**

- `src-tauri/src/authorization.rs` — added renderer roles, exact labels,
  typed command authorization records, centralized deny-by-default command
  checks, the build-time command name manifest, and focused policy tests.
- `src-tauri/src/commands/manifest.rs` — removed after its command list moved
  into the centralized authorization source of truth.
- `src-tauri/build.rs` — generates application-command permissions from the
  authorization manifest.
- `src-tauri/src/commands/companion.rs` — routes every current command through
  the centralized role/capability check while retaining the existing
  sanitized unauthorized error.
- `src-tauri/src/commands/mod.rs` and `src-tauri/src/lib.rs` — register the
  authorization module without changing command dispatch.
- `src-tauri/src/desktop/windows/companion.rs` and
  `src-tauri/src/events.rs` — reuse the centralized renderer labels/roles so
  command and event routing cannot drift.
- `src-tauri/capabilities/companion.json` and
  `src-tauri/capabilities/preferences.json` — narrowed capability selection
  from whole-window inheritance to exact local webview labels.
- `tests/tauri-ipc-authorization.test.cjs` — verifies exact capability
  loading, local-only scope, no wildcards/remotes, companion-only command
  grants, no broad plugin permissions, and one narrow generated allow/deny
  pair per application command.
- `docs/migrating/progress.md` — recorded Task 3.4 completion.

**Validation performed**

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (18 tests),
  including companion allow, Preferences deny, unknown-label deny, manifest
  uniqueness, and event-role routing.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- Focused Tauri authorization/event capability tests: passed (6 tests).
- `npx tauri permission ls`: confirmed generated narrow allow/deny
  permissions for exactly the five completed commands.
- `npm run typecheck`: passed.
- `npm test`: passed (124 tests across 35 suites).
- `npm run build`: passed.
- `npm run tauri:dev`: passed. Exact-webview capabilities authorized the
  companion cursor/window operations with no command or capability errors;
  the process was then stopped manually.
- Electron smoke launch: passed with the existing preload bridge, IPC
  authorizer, and deny-by-default WebContents permission policy intact; the
  process was then stopped manually.

**Scope boundary**

- Task 3.4 implements only IPC authorization policy and least-privilege
  capability enforcement.
- No Electron IPC file or renderer DesktopBridge adapter changed.
- Task 3.5 was not started.

**Blockers**

- None.

**Next task**

- Task 3.5 — DesktopBridge → Tauri.

### Task 3.5 — DesktopBridge → Tauri

**Status:** Complete

**Bridge architecture**

- Electron and Tauri continue to implement one internal `DesktopBridge`
  adapter contract, selected once at renderer startup.
- The complete runtime adapter is now private. Renderers import either a
  companion-only view or a Preferences-only view, so TypeScript prevents one
  renderer role from requesting native capabilities assigned to the other.
- React, engine, personality, and shared code contain no direct Electron
  imports, Tauri imports, or Electron preload-global access.
- The Electron adapter and both preload implementations remain unchanged and
  authoritative for Electron.
- The Tauri adapter retains the completed Phase 1–3 cursor/window command
  surface and reuses the existing typed command, event, and cursor transports.

**Files changed**

- `src/desktop/contracts.ts` — split the renderer-facing API into
  `CompanionDesktopBridge` and `PreferencesDesktopBridge` role contracts while
  retaining the complete internal adapter contract.
- `src/desktop/DesktopBridge.ts` — keeps runtime detection inside the desktop
  boundary, makes the full selected adapter private, and exports frozen
  role-scoped bridge views.
- `src/renderer/App.tsx`, `src/renderer/components/PsyDuck.tsx`,
  `src/renderer/hooks/usePomodoroState.ts`,
  `src/renderer/hooks/useReminderNotifications.ts`, and
  `src/renderer/hooks/useRuntimeSettings.ts` — consume only the companion
  bridge view.
- `src/renderer/PreferencesApp.tsx`,
  `src/renderer/hooks/usePreferencesSettings.ts`, and
  `src/renderer/hooks/useUpdateStatus.ts` — consume only the Preferences
  bridge view.
- `tests/desktop-bridge-boundary.test.cjs` — prevents direct runtime API
  imports, preload-global access, complete-adapter exports, and renderer-role
  bridge crossover.
- `docs/migrating/progress.md` — recorded Task 3.5 and the Phase 3
  infrastructure boundary.

**Validation performed**

- Focused DesktopBridge boundary tests: passed (3 tests).
- `npm run typecheck`: passed.
- `npm test`: passed (127 tests across 36 suites).
- `npm run build`: passed, including the Electron main process and both
  renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (18 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run tauri:dev`: passed. Tauri compiled, launched, and remained running
  through the role-scoped bridge boundary; it was then stopped manually.
- Electron smoke launch: passed through the unchanged preload adapter. The
  existing deny-by-default permission checks remained active; the process was
  then stopped manually.

**Phase 3 scope boundary**

- Under the accepted project clarification, Phase 3 migrates IPC
  infrastructure only. It does not implement feature domains assigned to
  later phases.
- Tauri feature-domain bridge methods that are not part of completed
  Phases 1–3 remain unavailable by design. No menus, settings, credentials,
  reminders, Pomodoro, AI, updater, or other later-phase behavior was added.
- No Electron code was removed, and Phase 4 was not started.

**Blockers**

- None for Task 3.5.
- `docs/migrating/migration_tasks.md` contains placeholders for Phases 4–11,
  so no later task will be inferred without an executable task definition.

**Next task**

- Phase 4 — not started.

## Phase 4 — Tray + Native Menu Migration

### Task 4.1 — Audit Existing Tray Behaviour

**Status:** Complete — contract clarification applied

**Electron tray lifecycle**

- `src/main/main.ts` owns one process-wide `Tray | null`.
- Startup waits for `app.whenReady()`, loads every current service, registers
  IPC, applies settings, and creates the companion before calling
  `createSystemTray()`.
- Tray creation is best effort. A load/construction failure is logged as
  `[tray] create_failed`; it does not terminate startup and is not retried.
- `src/main/tray.ts` creates exactly one tray icon, sets the `Ducky` tooltip,
  assigns one context menu, and returns the retained native tray object.
- There is no tray click, double-click, balloon, drag, visibility toggle, or
  icon-state event.
- There is no `window-all-closed` quit handler. Closing both windows leaves
  the process and tray alive. macOS `activate` shows or recreates the
  companion.
- `before-quit` removes application listeners, disposes the current services
  and IPC handlers, explicitly destroys the tray, and clears its reference.
- Restart cancels active AI work, disposes Pomodoro, calls `app.relaunch()`,
  then exits. Quit calls `app.quit()`, which reaches the shared shutdown path.

**Tray icon**

- Electron uses only `assets/icons/icon.png`; there are no alternate,
  selected, disabled, light/dark, or animated tray states.
- `nativeImage.createFromPath()` loads the packaged/development application
  icon. An empty image aborts tray creation.
- Electron resizes the icon with `quality: "best"` to `18 × 18` on macOS and
  `20 × 20` on Windows/Linux.
- Electron does not mark the macOS image as a template image, so the existing
  full-color artwork is the parity reference.
- Tauri already contains generated application icons under
  `src-tauri/icons/`, but no tray-specific asset or loader exists.

**Native menu hierarchy**

- The static tray menu is created once at startup:
  1. `Show Ducky`
  2. `Preferences…`
  3. `About Ducky`
  4. separator
  5. `Restart`
  6. `Quit`
- The macOS application menu is created before `whenReady()`:
  - `Ducky`: `About Ducky`, separator, Services, separator, Hide Ducky,
    Hide Others, Show All, separator, Quit Ducky.
  - the standard Edit menu, which preserves native text-editing shortcuts.
- The companion context menu is rebuilt on every renderer right-click so its
  dynamic state is current:
  - `Pomodoro`: `25 min` and `50 min` radio items, separator, `Custom…`
    radio item, separator, and enabled-state-aware Pause/Resume/Stop items.
  - separator.
  - `Personal Assistant`: Set My Name, separator, Reminders
    (New/Manage), Daily Planner, separator, Sticky Message
    (Set/Clear, with Clear enabled only when a message exists).
  - separator.
  - `Water Reminders`: Enabled checkbox and Reminder Interval
    (`30/45/60 min` radio items).
  - separator.
  - Eye Tracking and Always On Top checkboxes.
  - separator.
  - Preferences, About Ducky.
  - separator.
  - Restart, Quit.

**Menu state and actions**

- The application and tray menus are static; neither is rebuilt after
  startup and neither contains checked or disabled state.
- The context menu snapshots `AppSettings` and `PomodoroState` every time it
  opens. There is no subscription-driven native-menu state cache.
- Tray actions show/focus or recreate the companion, show/focus the existing
  Preferences window or create it, show About, restart, and quit.
- Context-menu actions additionally mutate water/eye/always-on-top/sticky
  settings, control Pomodoro, request a custom Pomodoro duration, and request
  the name, sticky-message, reminder creation/management, and daily-planner
  renderer panels.
- The Electron reference has no Hide Companion or Hide Preferences menu
  action. Its macOS Hide Ducky item is the standard application-level role,
  not a window-specific tray command.

**Renderer and DesktopBridge integration**

- `PsyDuck.tsx` prevents the browser context menu and calls
  `companionDesktopBridge.getCompanionBridge()?.showCompanionContextMenu()`.
- Electron adapts that operation to the authorized
  `psyduck:show-context-menu` preload event. The main process validates the
  companion role and opens the native menu against the companion window.
- Native panel actions return to the companion through the existing targeted
  events for user name, sticky message, reminder creation, reminder
  management, daily planner, and custom Pomodoro duration.
- `App.tsx` subscribes to those actions through `CompanionBridge`; it contains
  no Electron/Tauri detection.
- The Tauri adapter intentionally returns `undefined` for the full
  `CompanionBridge` until its later feature domains are migrated. It currently
  exposes only the completed Phase 1–3 `CompanionWindowBridge`.
- The architecture-compatible Phase 4 request path would place a scoped
  `show_companion_context_menu` command behind the companion DesktopBridge
  view, authorize only the exact companion webview, build the menu in
  `src-tauri/src/desktop/menus.rs`, and retain the tray in
  `src-tauri/src/desktop/tray.rs`. Native-only window/quit/restart callbacks
  belong in Rust; panel requests use the existing targeted event
  infrastructure.
- Backend-created tray/menu objects require no renderer plugin permission.
  Only a renderer-requested companion context-menu command needs a generated,
  companion-only application-command permission. No generic menu/tray,
  filesystem, shell, process, or wildcard capability is required.

**Source-of-truth discrepancies**

1. Phase 4 requires the complete native menu, identical dynamic checked and
   enabled state, and every action to function. The Electron context menu
   obtains that state from Settings and Pomodoro and mutates Settings,
   Pomodoro, Reminders, and Daily Planner. The same Phase 4 contract expressly
   prohibits migrating Settings, Reminders, and Pomodoro before their later
   phases. A static/default/disabled menu would be a production placeholder
   and would fail parity.
2. Task 4.6 requires Hide Companion and Hide Preferences. Neither operation
   exists in the Electron tray or companion context menus. Adding them would
   invent new behavior instead of matching Electron.
3. Task 4.5 says every native menu action must route “through DesktopBridge.”
   `migration_codex.md` defines DesktopBridge as the renderer-to-runtime
   boundary while the Rust application core owns windows/tray/menus. Native
   show/focus/quit/restart callbacks therefore should remain inside Rust;
   only renderer context-menu requests and targeted panel events cross the
   bridge. Routing native-only callbacks through a renderer would reverse the
   prescribed trust boundary.

**Required contract decision**

- Either limit Phase 4 parity to the static tray menu, macOS application
  menu, native lifecycle, and their existing window/About/restart/quit
  actions, explicitly deferring the dynamic companion context menu to its
  Settings/Reminder/Pomodoro phases; or
- authorize the minimum Settings, Reminder, Daily Planner, and Pomodoro
  services to move early with the context menu, which changes the mandated
  phase order; and
- clarify that DesktopBridge covers renderer-originated menu requests and
  targeted renderer events, while native-only actions remain in Rust.
- Remove the two non-reference hide actions from Task 4.6 or identify the
  Electron behavior they are intended to preserve.

No application, Electron, Tauri, renderer, asset, dependency, capability, or
test file was changed. Per the architecture rule, implementation stopped
before Task 4.2 rather than guessing.

**Validation performed**

- Static source audit covered `src/main/main.ts`, `src/main/tray.ts`,
  `src/main/menus.ts`, both window constructors, branding, preload/IPC
  authorization, renderer menu subscriptions, DesktopBridge adapters, the
  Tauri command/event/authorization infrastructure, capabilities, icons, and
  Tauri configuration.
- Confirmed no direct Electron or Tauri API is used by the renderer for menu
  behavior.
- No build or runtime validation was required because Task 4.1 made no
  production-code change.

**Commits**

- None. Task 4.1 explicitly requires no commit.

**Contract clarification**

- The updated `docs/migrating/migration_tasks.md` resolves every discrepancy
  recorded during discovery.
- Phase 4 now covers Rust-owned tray lifecycle, the static tray and macOS
  application menus, their native callbacks, the renderer-requested context
  menu transport, and least-privilege authorization.
- Dynamic Settings, Pomodoro, Reminder, Daily Planner, Profile, Sticky
  Message, AI, and Updater menu behavior remains deferred to the feature phase
  that owns each domain.
- DesktopBridge is used only for renderer-originated context-menu requests.
  Native tray and application-menu callbacks remain in Rust.
- No new hide actions are required.

**Blockers**

- None. The clarified Phase 4 execution contract is architecture-compatible.

**Next task**

- Task 4.2 — Create Native Tray Infrastructure.

### Task 4.2 — Create Native Tray Infrastructure

**Status:** Complete

**Implementation summary**

- Added `src-tauri/src/desktop/tray.rs` as the Rust-owned tray lifecycle
  module.
- The tray uses a stable `ducky-tray` identifier and returns the existing
  native instance when initialization is requested more than once.
- Startup creates the tray after the companion window is created. The tray is
  retained by Tauri's native resource table and remains independent of every
  renderer.
- An empty native menu is attached during this infrastructure task so Linux
  status notifier implementations can display the tray. Task 4.4 replaces it
  with Ducky's static menu.
- Added `src-tauri/src/desktop/lifecycle.rs`. Closing the final window on
  Windows/Linux does not terminate the tray application; explicit exit and
  restart requests remain authoritative. macOS retains its native
  application lifecycle.
- `RunEvent::Exit` removes and drops the registered tray, matching Electron's
  explicit `before-quit` cleanup.
- Enabled only Tauri's built-in `tray-icon` feature. No renderer plugin,
  renderer permission, filesystem permission, shell permission, or process
  permission was introduced.
- The Electron tray, renderer, DesktopBridge, and application behavior were
  not changed.

**Files changed**

- `docs/migrating/migration_tasks.md` — accepted the clarified Phase 4
  execution contract as the source of truth.
- `src-tauri/Cargo.toml` — enabled Tauri's built-in `tray-icon` feature.
- `src-tauri/src/desktop/mod.rs` — registered the lifecycle and tray modules.
- `src-tauri/src/desktop/lifecycle.rs` — added tray-resident application
  lifecycle and shutdown cleanup.
- `src-tauri/src/desktop/tray.rs` — added singleton native tray creation and
  destruction.
- `src-tauri/src/lib.rs` — created the tray during setup and connected the
  native run-event lifecycle.
- `docs/migrating/progress.md` — recorded the clarified contract and Task 4.2.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (127 tests across 36 suites).
- `npm run build`: passed, including the Electron main process and both
  renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (21 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; no tray/menu renderer capability was
  added.
- Electron development smoke launch: passed. The unchanged companion renderer
  loaded from the repository build and Electron stayed alive until stopped
  manually.
- Tauri development smoke launch: passed. Rust compiled, the companion loaded,
  and the process stayed alive with the native tray registered until stopped
  manually.

**Manual verification**

- Confirmed the Tauri companion remains the startup window; Preferences stays
  hidden.
- Confirmed the Tauri process remains alive after startup with no tray
  construction error or permission warning.
- Native menu hierarchy and actions are intentionally verified in Tasks
  4.4–4.5 after they exist.

**Blockers**

- None.

**Next task**

- Task 4.3 — Migrate Tray Icon.

### Task 4.3 — Migrate Tray Icon

**Status:** Complete

**Implementation summary**

- The native tray embeds the existing `assets/icons/icon.png` at compile time;
  no duplicate tray asset was created.
- The image is decoded from the original 1024 × 1024 RGBA PNG and resized with
  a Lanczos3 filter, the high-quality resampling equivalent of Electron's
  `quality: "best"` path.
- macOS receives an 18 × 18 full-color icon. Windows and Linux receive a
  20 × 20 full-color icon, preserving the Electron platform behavior.
- `icon_as_template(false)` preserves the original artwork instead of
  recoloring it as a macOS template image.
- Invalid embedded image data remains a startup tray construction error,
  matching Electron's explicit empty-image guard.
- Added a focused Rust test that verifies the shared source is a PNG and the
  native output has the correct target dimensions and RGBA length.

**Files changed**

- `src-tauri/Cargo.toml` — added a pinned, PNG-only `image` dependency for
  deterministic high-quality resizing.
- `src-tauri/Cargo.lock` — locked the direct image decoder/resizer dependency.
- `src-tauri/src/desktop/tray.rs` — loads, resizes, and registers the existing
  application icon.
- `docs/migrating/progress.md` — recorded Task 4.3.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (127 tests across 36 suites).
- `npm run build`: passed, including the Electron main process and both
  renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (22 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; icon loading is backend-owned and
  exposes no renderer permission.
- Electron development smoke launch: passed with the unchanged repository
  renderer and application icon behavior.
- Tauri development smoke launch: passed. The icon decoded and the tray
  registered without a construction error; the application stayed alive
  until stopped manually.

**Manual verification**

- Confirmed the Tauri companion remains the startup window and the tray icon
  construction path emits no error.
- Exact menu-bar appearance and interaction are included in the complete
  Phase 4 manual parity pass after the static menu exists.

**Blockers**

- None.

**Next task**

- Task 4.4 — Migrate Static Native Menus.
