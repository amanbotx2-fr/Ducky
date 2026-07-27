# Ducky Tauri v2 Migration Progress

**Last updated:** 28 July 2026

## Current Status

- Active work: None
- Last completed phase: Phase 8 — Pomodoro Migration
- Last completed task: Phase 9 architecture clarification
- Next task: Task 9.2 — Create Native AI Runtime. Do not begin without a
  separate implementation instruction.
- Blockers: None. The Phase 9 contract now follows the Electron final-response,
  lifecycle-cancellation, connection-diagnostics, and one-request-per-role
  behavior. Claude remains the only approved functional expansion.

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

### Task 4.4 — Migrate Static Native Menus

**Status:** Complete

**Implementation summary**

- Added a Rust-owned static tray menu with the Electron ordering and labels:
  Show Ducky, Preferences…, About Ducky, separator, Restart, Quit.
- About uses Tauri's native predefined About item with Ducky name, version,
  description, copyright, and runtime credits. It does not pass through a
  renderer.
- On macOS, startup installs the two-menu Electron hierarchy:
  - Ducky: About Ducky, separator, Services, separator, Hide Ducky, Hide
    Others, Show All, separator, Quit Ducky.
  - Edit: Undo, Redo, separator, Cut, Copy, Paste, Select All.
- Services, hide, hide-others, show-all, quit, and all Edit operations use
  native predefined roles so platform shortcuts and responder-chain behavior
  remain native.
- Windows and Linux retain Electron parity by not installing an application
  menu. Their tray menu is still native.
- Static action identifiers are centralized in `desktop/menus.rs`; focused
  tests lock the tray hierarchy and ensure IDs cannot collide.
- No dynamic companion context menu, checked state, Settings, Pomodoro,
  Reminder, Daily Planner, Sticky Message, AI, or Updater behavior was added.

**Files changed**

- `src-tauri/src/desktop/menus.rs` — added static tray and macOS application
  menu construction.
- `src-tauri/src/desktop/mod.rs` — registered the native menu module.
- `src-tauri/src/desktop/tray.rs` — replaced the Linux compatibility menu with
  Ducky's static tray menu.
- `src-tauri/src/lib.rs` — installs the macOS application menu during native
  setup.
- `docs/migrating/progress.md` — recorded Task 4.4.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (127 tests across 36 suites).
- `npm run build`: passed, including the Electron main process and both
  renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (24 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; all menu construction remains backend
  owned.
- Electron development smoke launch: passed with its original application and
  Edit menus unchanged.
- Tauri development smoke launch: passed. Both native menus installed without
  a construction error, the companion loaded, and the process remained alive
  until stopped manually.

**Manual verification**

- The Electron menu hierarchy was read from the running reference application
  and matched against the Rust menu specification.
- The Tauri native menu hierarchy will receive its complete interactive pass
  together with Task 4.5 callbacks, when every static item is actionable.

**Blockers**

- None.

**Next task**

- Task 4.5 — Migrate Native Menu Actions.

### Task 4.5 — Migrate Native Menu Actions

**Status:** Complete

**Implementation summary**

- Added a closed Rust dispatch table for the four custom tray action IDs.
  Unknown menu IDs are ignored and cannot reach a native operation.
- Show Ducky now shows and focuses an existing companion or recreates it with
  the original positioning, size, security, and page-load behavior after the
  window has been destroyed.
- Preferences… now shows and focuses the declarative hidden Preferences
  window. If that window was destroyed, Rust recreates the same label, URL,
  dimensions, minimum dimensions, title, resizability, and hidden-before-load
  behavior.
- Restart uses `AppHandle::request_restart`, allowing Tauri to deliver
  `ExitRequested` and `Exit` so tray cleanup occurs before the process is
  relaunched.
- Quit uses `AppHandle::exit(0)`, allowing the same shutdown cleanup.
- About Ducky remains Tauri's native predefined About action with backend-owned
  metadata. macOS Quit Ducky remains the native quit role.
- macOS `RunEvent::Reopen` now shows/focuses or recreates the companion,
  preserving Electron's `activate` behavior.
- Menu failures are logged at the Rust boundary without involving a renderer.
- No DesktopBridge method, feature-domain state, or Electron implementation
  was changed.

**Files changed**

- `src-tauri/src/desktop/menus.rs` — added closed native action dispatch and
  tray callbacks.
- `src-tauri/src/desktop/lifecycle.rs` — dispatches native menu events and
  handles macOS reopen.
- `src-tauri/src/desktop/windows/companion.rs` — added reusable native
  show/focus/recreate behavior.
- `src-tauri/src/desktop/windows/preferences.rs` — added native
  show/focus/recreate behavior for Preferences.
- `src-tauri/src/desktop/windows/mod.rs` — registered the Preferences window
  module.
- `docs/migrating/progress.md` — recorded Task 4.5.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (127 tests across 36 suites).
- `npm run build`: passed, including the Electron main process and both
  renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (26 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; native callbacks require no renderer
  permission.
- Electron development smoke launch: passed with its original native actions
  and lifecycle unchanged.
- Tauri development smoke launch: passed. Native dispatch registered, the
  companion loaded, and no menu action or permission warning was logged.

**Manual verification**

- Native action IDs and their dispatch targets are covered by focused Rust
  tests, including rejection of unknown IDs.
- Window recreation constants are locked against the declarative Preferences
  configuration by a focused Rust test.
- The final Phase 4 interactive pass verifies each tray/application menu item,
  restart, and quit from a packaged native app so it can be selected
  independently from the installed Electron reference.

**Blockers**

- None.

**Next task**

- Task 4.6 — Renderer Context Menu Bridge.

### Task 4.6 — Renderer Context Menu Bridge

**Status:** Complete

**Implementation summary**

- The companion renderer now requests its native context menu through the
  narrow `CompanionWindowBridge` surface. It does not import or detect either
  Electron or Tauri at runtime.
- The Electron adapter continues using the existing preload implementation;
  its context-menu behavior and feature-domain items are unchanged.
- The Tauri adapter dispatches one typed `show_companion_context_menu`
  command. Rust authorizes the calling companion window, constructs the menu,
  displays it at the native cursor position, and owns every callback.
- The Phase 4 Tauri context menu contains only the migrated static slice:
  Preferences…, About Ducky, separator, Restart, and Quit. Runtime Settings,
  Pomodoro, Reminders, Daily Planner, Sticky Message, AI, and Updater items
  remain deferred to their owning phases as required by the revised contract.
- The static tray and companion-context specifications share the same native
  menu construction and action dispatch paths, avoiding a second callback
  architecture.
- Added a renderer-boundary test that prevents future context-menu requests
  from expanding back to the feature-domain bridge.

**Files changed**

- `src/shared/types.ts` — moved the context-menu request onto the narrow
  companion-window contract.
- `src/renderer/components/PsyDuck.tsx` — requests the menu through
  `getCompanionWindowBridge`.
- `src/desktop/tauriCommands.ts` — registered the typed context-menu command.
- `src/desktop/tauriBridge.ts` — added the Tauri bridge adapter request.
- `src-tauri/src/commands/companion.rs` — added the authorized native command.
- `src-tauri/src/commands/mod.rs` — registered the command handler.
- `src-tauri/src/desktop/menus.rs` — added the migrated static context menu
  using the existing native menu/action architecture.
- `tests/desktop-bridge-boundary.test.cjs` — locks the narrow bridge boundary.

**Blockers**

- None.

### Task 4.7 — Migrate Tray and Menu Permissions

**Status:** Complete

**Implementation summary**

- Added one generated application-command permission for
  `show_companion_context_menu`.
- Granted that permission only to the companion capability. Preferences
  cannot invoke it.
- The command performs a second Rust authorization check against the exact
  companion window label before creating native UI.
- No renderer-facing tray or menu plugin permission was granted. Tray
  lifecycle, construction, application menus, and all native callbacks remain
  backend-owned.
- Authorization and capability tests now reject generic menu/tray permissions
  and lock the complete migrated command set.

**Files changed**

- `src-tauri/src/authorization.rs` — added the companion-only command policy
  and extended the migrated-command manifest.
- `src-tauri/build.rs` — generates permissions from the full migrated command
  manifest.
- `src-tauri/permissions/autogenerated/show_companion_context_menu.toml` —
  generated the exact application-command permission.
- `src-tauri/capabilities/companion.json` — grants only that permission to the
  companion.
- `tests/tauri-ipc-authorization.test.cjs` — locks least privilege and rejects
  generic native-menu/tray access.

**Validation performed for Tasks 4.6–4.7**

- `npm run typecheck`: passed.
- `npm test`: passed (128 tests across 36 suites).
- `npm run build`: passed, including the Electron main process and both
  renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (27 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; the generated command permission was
  accepted and no generic menu/tray permission is present.
- Electron development smoke launch: passed with the original preload and
  context menu unchanged.
- Tauri development smoke launch: passed. The companion loaded, the process
  remained alive, and no command-scope or permission warning was logged.

**Manual verification**

- Interactive context-menu actions and the complete tray/application menu
  parity pass are the final Phase 4 gate and will be recorded together from a
  packaged debug application.

**Blockers**

- None.

**Next task**

- Complete the Phase 4 manual parity verification; do not begin Phase 5.

### Phase 4 Final Verification Gate

**Status:** Complete

**Packaged application verification completed**

- Built a dedicated debug macOS bundle with
  `npm run tauri:build -- --debug --bundles app`.
- Confirmed startup displays only the companion window at
  `tauri://localhost`; Preferences does not open automatically.
- Confirmed the companion mascot is visible and uncropped.
- Confirmed the macOS Ducky menu contains About Ducky, Services, Hide Ducky,
  Hide Others, Show All, and Quit Ducky in the expected native hierarchy.
- Confirmed the Edit menu contains Undo, Redo, Cut, Copy, Paste, and Select
  All, with macOS-provided system additions remaining native.
- Opened About Ducky and confirmed Ducky 1.1.0 metadata and copyright.
- Opened the companion context menu and confirmed its migrated static ordering:
  Preferences…, About Ducky, Restart, Quit.
- Selected Preferences… and confirmed the native Preferences window opens only
  on request while the companion remains available.
- Selected Restart and confirmed the original process exited, a new process
  started with a different PID, and only the companion appeared after restart.
- Selected Quit and confirmed the packaged Tauri process exited completely.
- No renderer error, command-scope warning, or permission warning appeared
  during packaged verification. Tauri development mode emitted only its known
  custom-protocol-to-`postMessage` transport fallback warning.

**Tray evidence completed**

- Tauri setup creates one tray resource with the stable `ducky-tray` ID and
  fails startup if icon decoding or tray construction fails.
- The packaged process remained alive through startup and successfully
  restarted with no tray construction error.
- Focused Rust tests lock the tray identity, resized icon output, exact menu
  ordering/labels/separator, and closed native action dispatch.
- Tray actions and the companion static context menu use the same Rust callback
  dispatcher; Preferences, About, Restart, and Quit were exercised
  interactively through that shared path.

**Tray-only manual verification completed**

- Confirmed the 18 × 18 full-color Ducky tray icon is visible in the macOS menu
  bar.
- Confirmed the tray menu contains Show Ducky, Preferences…, About Ducky,
  separator, Restart, and Quit in the expected order.
- Confirmed Show Ducky shows and focuses the companion after it is hidden or
  covered.

**Final validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (128 tests across 36 suites).
- `npm run build`: passed, including the Electron main process and both
  renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (27 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed.
- Electron development smoke launch: passed; the original Electron
  implementation remained unchanged.
- Tauri development smoke launch: passed; the companion loaded and the process
  remained alive until stopped manually.
- Packaged debug Tauri build and interactive verification: passed for all
  accessible checks listed above.

**Phase 4 completion**

- Phase 4 is complete. Native tray lifecycle, icon, menu hierarchy, actions,
  context-menu bridge, permissions, Electron parity, and the required manual
  verification all passed.
- Phase 5 has not been started.

**Next task**

- Resolve and revalidate the Phase 2 cursor-position regression documented
  below. Do not begin Phase 5 until the existing eye-tracking parity defect is
  corrected.

### Phase 2 Cursor-Position Regression Investigation

**Status:** Resolved and revalidated

**Observed regression**

- Cursor samples continue reaching the companion and `EyeTracker` continues
  animating the pupils.
- On a mixed-DPI desktop, the pupils point toward a consistently offset
  location instead of the actual cursor.

**Environment reproduced**

- The primary display is 1710 × 1112 logical points at scale factor 2.
- A secondary display is 1920 × 1080 logical points at virtual-desktop origin
  `(1710, 32)` and scale factor 1.
- This is the exact class of mixed-DPI layout required by the Phase 2 exit
  criteria.

**Root cause**

- Tao's macOS cursor implementation reads `NSEvent.mouseLocation`, converts it
  to top-left logical desktop coordinates, and then converts the complete
  global point to `PhysicalPosition` using the **primary monitor** scale
  factor.
- `src-tauri/src/commands/companion.rs` receives that physical point from
  `window.cursor_position()` but converts it back to logical coordinates using
  the **companion window** scale factor.
- When the primary display and companion window use different scale factors,
  the resulting cursor point is multiplied by
  `primary_scale / companion_scale`. The conversion also scales the virtual
  monitor origin, producing the observed stable offset.
- `src/renderer/components/PsyDuck.tsx` correctly computes the eye origin from
  `window.screenX`, `window.screenY`, and DOM bounds in CSS logical pixels.
  `src/engine/EyeTracker.ts` correctly subtracts and normalizes the two input
  points. The defect is that the Rust cursor point is not guaranteed to use
  that same logical coordinate space.
- Electron is unaffected because `screen.getCursorScreenPoint()` already
  returns desktop DIP coordinates matching the renderer's screen-coordinate
  contract.

**Coverage gap**

- The existing Rust conversion test uses one physical point and one scale
  factor. It proves a same-scale division but does not model distinct primary,
  cursor-monitor, and companion-window scale factors or monitor origins.
- Phase 2 development smoke validation proved that the channel operated, but
  the locked-session gate explicitly left mixed-DPI visual/input parity for
  later manual verification.

**Correction contract**

- Keep the renderer and `EyeTracker` in CSS logical desktop coordinates.
- Normalize the native cursor sample to that coordinate system without using
  the companion window's scale factor as a global-desktop conversion factor.
- Add focused coverage for primary/companion scale-factor mismatches, cursor
  movement across monitor origins, negative origins, and same-scale behavior.
- Re-run Electron and Tauri eye-tracking parity on Retina, 1×, and mixed-DPI
  monitor arrangements before resuming the migration.

**Implementation**

- `src-tauri/src/commands/companion.rs` now converts Tao's physical cursor
  payload with the primary monitor scale factor that Tao used to create it.
- The companion window scale factor is retained only as a defensive fallback
  for a transient or unsupported state where no primary monitor is available.
- The cursor channel payload once again matches Electron desktop DIP
  coordinates and the renderer's CSS logical `window.screenX` /
  `window.screenY` contract.
- The renderer, `EyeTracker`, DesktopBridge, cursor channel lifecycle,
  Electron implementation, and all Phase 4 functionality remain unchanged.

**Regression tests**

- Mixed-DPI conversion with a 2× primary monitor and 1× companion monitor.
- A 1× primary monitor with a 2× Retina companion monitor, proving the
  companion factor does not rescale a primary-scaled cursor payload.
- A Retina-primary to 1× external-monitor example that preserves the external
  monitor origin and cursor offset.
- Companion-scale fallback behavior when a primary monitor is unavailable.

**Validation performed**

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (30 tests),
  including all four cursor-coordinate regression tests.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; the existing companion-only cursor
  permissions remain unchanged.
- `npm run typecheck`: passed.
- `npm test`: passed (128 tests across 36 suites).
- `npm run build`: passed, including the Electron main process and both
  renderer entries.
- Electron production-output smoke launch: passed using the unchanged
  Electron cursor sampler and renderer bridge.
- `npm run tauri:dev`: passed. The companion and cursor channel started with
  no command, scope, or permission error; only the known development
  custom-protocol-to-`postMessage` fallback warning appeared.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  packaged debug macOS application.
- Packaged mixed-DPI visual verification: passed. The companion was moved from
  the 2× 1710 × 1112 primary display fully onto the 1× 1920 × 1080 external
  display. Its pupils then followed cursor positions at both the top-right and
  bottom-left of the companion without the prior scale/origin offset.

**Environment note**

- The first Rust validation attempt failed before tests ran because the disk
  was full. Only rebuildable Cargo output under `src-tauri/target` was cleaned;
  the complete validation matrix then passed from a clean Rust rebuild.

**Scope**

- No Phase 4 tray, menu, permission, or lifecycle implementation was changed.
- Phase 5 has not started.

### Phase 2 Native Drag-Anchor Regression

**Status:** Resolved

**Observed regression**

- The Tauri companion moved when dragged, but did not preserve the original
  point where the pointer grabbed it.
- Repeated drags introduced additional vertical error and made the companion
  appear to slide away from the cursor.
- Electron retained the exact grab point.

**Root cause**

- `PsyDuck.tsx` supplied `window.screenX` and `window.screenY` as the current
  native window position to `DragController`.
- Those browser window-origin values are not authoritative in macOS
  WKWebView. The resulting pointerdown offset was therefore calculated from
  an incorrect window origin and was carried into every absolute move.
- The controller already kept its offset immutable during each drag.
  DesktopBridge and the Rust move command passed absolute logical coordinates
  without another scale conversion, and absolute rounding could not create
  the accumulating error.

**Implementation**

- Added `getWindowPosition()` to the role-scoped `CompanionWindowBridge`.
- Electron now serves that operation from the authorized companion
  `BrowserWindow` bounds while retaining its existing absolute
  `setPosition()` movement implementation.
- Tauri now exposes a companion-only
  `get_companion_window_position` command. Rust reads the native outer
  position and converts it to the current monitor's logical coordinate space
  before returning it.
- `DragController` now requests the native position once on pointerdown and
  creates the immutable drag anchor from that position plus the pointer's
  `clientX` / `clientY`.
- Pointer movement that arrives while the native request is resolving is
  retained and applied once initialization completes. A generation guard
  prevents a late response from reviving a completed drag.
- Absolute logical movement, pointer capture, drag state callbacks, Electron
  IPC, and Tauri command dispatch remain otherwise unchanged.
- Added and granted only the generated companion command permission;
  Preferences receives no new authority.

**Regression tests**

- A macOS WKWebView case with deliberately invalid browser
  `window.screenX` / `window.screenY` values proves they cannot influence the
  anchor.
- Exact client-space grab-point preservation is verified after movement.
- Two consecutive drags with different grab points prove that no position
  error accumulates.
- Movement received before the native window-position request resolves is
  preserved.
- Tauri authorization tests verify the new command is companion-only and has
  one narrow generated allow/deny permission pair.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (131 tests across 37 suites).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (30 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed and listed the new scoped
  `allow-get-companion-window-position` permission.
- Electron production-output smoke launch: passed; the companion window
  opened with the existing Electron movement implementation and no IPC
  authorization error.
- `npm run tauri:dev`: passed; the companion opened and the new command
  registry/capability configuration produced no command or permission error.
  The only warning was the existing development custom-protocol transport
  fallback.

**Commit**

- `fix(tauri): preserve drag anchor using native window position`

**Scope**

- No Phase 4 tray or menu behavior was changed.
- Eye tracking and the cursor stream were not changed.
- Phase 5 has not started.

## Phase 5 — Settings Migration

### Task 5.1 — Audit Existing Settings

**Status:** Complete

**Storage backend and persistence contract**

- Electron owns one process-wide `SettingsService` in
  `src/main/SettingsService.ts`.
- The service reads and writes `settings.json` under Electron's
  `app.getPath('userData')`.
- The current production document has no literal schema-version field.
  Compatibility is maintained by strict parsing, additive optional fields,
  default merging, and canonical rewrites. Tauri must preserve that actual
  document contract rather than introduce a new wrapper.
- Mutations are serialized through one operation queue. Each accepted
  snapshot is written to `settings.json.tmp` with mode `0600` and renamed
  over the authoritative file.
- Invalid JSON or an invalid document is renamed to
  `settings.json.invalid-<timestamp>` before defaults are written.
- Missing files materialize the defaults. Unknown keys, invalid enum values,
  invalid lengths, malformed reminders, malformed credentials, and invalid
  endpoint URLs are rejected.
- Credentials share the Electron snapshot, but their encryption, decryption,
  and plaintext migration are explicitly Phase 6 work. Phase 5 must preserve
  credential and deferred-domain data without exposing it to a renderer.

**Persisted schema and defaults**

- `userName`: `"Friend"`.
- `stickyMessage`: `null`.
- `reminders`: an empty array; reminder behavior remains deferred.
- `general.alwaysOnTop`: `true`.
- `general.launchAtStartup`: `false`.
- `general.eyeTracking`: `true`.
- `water.enabled`: `true`; water behavior/configuration remains deferred.
- `water.interval`: `30`, constrained to `15`, `30`, `45`, `60`, `90`, or
  `120`.
- `notificationSounds.enabled`: `true`.
- `notificationSounds.sound`: `"soft-bell"`, constrained to the four built-in
  sound IDs.
- `notificationSounds.volume`: `70`, constrained to integer `0…100`.
- `updates.automatic`: `false`; updater behavior/preferences remain deferred.
- `ai.enabled`: `false`, `provider`: `""`, `model`: `""`,
  `endpoint`: `"http://localhost:11434"`, and `baseUrl`: `""`; AI settings
  remain deferred.
- `aiModelExplorer.favorites` and `recent`: empty arrays; AI/model selection
  remains deferred.
- `credential`: `null`. Older Electron documents may instead contain
  `ai.apiKey`; credential interpretation and migration remain deferred.
- `apiKeyConfigured` exists only in in-memory/renderer-safe projections and
  is derived from protected or legacy credential presence; it is not written
  as an AI setting.

**Startup and runtime flow**

- Electron constructs `SettingsService` after `app.whenReady()`, loads it
  before updater/reminder/AI/Pomodoro services, IPC registration, runtime
  setting application, companion creation, and tray creation.
- A load failure is logged and the service's in-memory defaults remain
  available.
- Startup applies `alwaysOnTop` to the companion and, for packaged Electron,
  `launchAtStartup` through `app.setLoginItemSettings`.
- Successful updates persist first, replace the in-memory snapshot second,
  then notify subscribers. No-op updates skip the write and notification.
- The main-process subscriber reapplies native general settings and emits the
  secret-free `RuntimeSettings` projection to the companion and Preferences.
- Electron runtime mutations currently come from user-name/sticky-message
  commands, Preferences patches, AI configuration, reminders, and native
  menu actions. Only settings infrastructure is in Phase 5; domain behavior
  stays with its owning phase.

**Renderer and DesktopBridge interactions**

- `useRuntimeSettings.ts` obtains a secret-free snapshot and change events for
  user name, sticky message, general settings, water configuration, and
  notification sounds. The companion uses those values for presentation,
  eye tracking, and sound configuration.
- `usePreferencesSettings.ts` obtains the redacted Preferences snapshot,
  performs optimistic settings updates, reconciles from the authoritative
  snapshot after failure, and consumes the shared runtime-settings event.
- `PreferencesApp.tsx` renders general, notification sound, hydration,
  updater, AI, and model-explorer controls. Phase 5 may connect the settings
  transport without implementing the deferred updater, AI, credential,
  reminder, or water domain behavior.
- Electron's companion and Preferences preload APIs implement the existing
  typed settings methods. `electronDesktopBridge` adapts them unchanged.
- Tauri currently exposes no companion-domain or Preferences-domain bridge,
  so both settings hooks fall back or fail. Phase 5 must add narrow
  settings-only bridge views rather than exposing incomplete AI, updater,
  reminder, or Pomodoro capabilities.
- Tauri's existing `runtime-settings:changed` event registry already targets
  only the exact `companion` and `preferences` webviews. Phase 5 owns its
  first producer and snapshot recovery commands.

**Phase 5 boundary**

- The native repository will preserve the complete compatible document so a
  Tauri save cannot erase Electron or future-phase data.
- Phase 5 will implement typed settings snapshots, validated settings
  patches, atomic persistence, startup loading, native application of general
  settings, notification, and Preferences integration.
- It will not interpret credentials, call AI providers, schedule reminders or
  hydration, run Pomodoro behavior, check for updates, or change release
  infrastructure.

**Production code changes**

- None. This task was discovery and documentation only.

**Blockers**

- None.

**Next task**

- Task 5.2 — Create Native Settings Store.

### Task 5.2 — Create Native Settings Store

**Status:** Complete

**Implementation summary**

- Added a runtime-owned Rust settings domain with typed, serde-validated
  representations of the existing Electron document.
- Preserved the current `settings.json` field names, optional-field migration
  behavior, defaults, redacted credential status boundary, and strict
  rejection of unknown or malformed settings-owned values.
- Deferred reminders and credential records are retained losslessly without
  being interpreted or exposed. AI, updater, reminder, hydration, Pomodoro,
  and credential behavior was not implemented.
- Added a native repository that materializes defaults for a missing file,
  writes a same-directory temporary file with owner-only permissions, flushes
  it, atomically persists it, and syncs the containing directory where
  supported.
- Invalid JSON or invalid settings are renamed to
  `settings.json.invalid-<timestamp>` before defaults are restored.
- Added a shared current-format fixture consumed by both the Electron
  `SettingsService` test suite and Rust schema tests, preventing the two
  parsers from drifting.
- The repository remains independently tested and is not registered during
  application startup until Task 5.4.

**Files changed**

- `src-tauri/src/domain/mod.rs` and
  `src-tauri/src/domain/settings/mod.rs` — native settings schema, defaults,
  canonicalization, validation, and compatibility tests.
- `src-tauri/src/infrastructure/mod.rs` and
  `src-tauri/src/infrastructure/persistence.rs` — atomic native settings
  repository and recovery tests.
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` — direct JSON, URL, and
  same-directory temporary-file dependencies.
- `src-tauri/src/lib.rs` — registered the independently tested modules without
  changing application startup.
- `tests/fixtures/settings/electron-current.json` and
  `tests/settings-contract.test.cjs` — shared Electron/Rust golden contract.
- `docs/migrating/migration_tasks.md` — accepted Phase 5 execution contract
  supplied before implementation.
- `docs/migrating/progress.md` — recorded discovery and Task 5.2.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (132 tests across 38 suites), including the shared
  Electron settings fixture.
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (38 tests),
  including defaults, migrations, strict validation, atomic persistence,
  invalid-file recovery, owner-only permissions, and deferred-data
  preservation.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; the backend-only store adds no renderer
  permission.
- Electron production-output smoke launch: passed with the existing
  `SettingsService`, preload bridges, and security policy unchanged.
- `npm run tauri:dev`: passed. Tauri compiled, launched the companion, and
  reported no settings, command, or capability error; the known development
  custom-protocol fallback warning remained unchanged.

**Manual verification**

- Store-level load/save/reload, default materialization, recovery, private
  permissions, and deferred-field preservation are covered by focused native
  tests.
- Interactive Preferences and restart persistence are deferred until the
  bridge, startup, and mutation milestones are connected.

**Blockers**

- None.

**Next task**

- Task 5.3 — DesktopBridge Settings API.

### Task 5.3 — DesktopBridge Settings API

**Status:** Complete

**Implementation summary**

- Split settings access from the existing feature-heavy companion and
  Preferences bridge interfaces.
- Added narrow `CompanionSettingsBridge` and
  `PreferencesSettingsBridge` contracts for snapshot, mutation, and
  secret-free change-notification operations.
- The role-scoped DesktopBridge views now expose settings-only getters.
  Runtime selection remains private to `src/desktop`; renderer code still has
  no Electron/Tauri import, preload-global access, or runtime detection.
- Electron maps the new narrow views to its unchanged, structurally
  compatible preload APIs.
- Tauri intentionally keeps the new settings getters unavailable until its
  native state is loaded during Task 5.4. No missing-state command or
  production placeholder was exposed.
- `useRuntimeSettings` and the ordinary snapshot/update paths in
  `usePreferencesSettings` now use settings-only bridge views. The dedicated
  AI mutation path continues using the full Electron Preferences bridge and
  remains unavailable to Tauri until its owning phase.

**Files changed**

- `src/shared/types.ts` — added the settings-only bridge contracts and
  composed the existing complete bridge types from them.
- `src/desktop/contracts.ts`, `src/desktop/DesktopBridge.ts`,
  `src/desktop/electronBridge.ts`, and `src/desktop/tauriBridge.ts` — exposed
  role-scoped settings views without changing runtime selection.
- `src/renderer/hooks/useRuntimeSettings.ts` and
  `src/renderer/hooks/usePreferencesSettings.ts` — consume only the narrow
  settings bridge for settings operations.
- `tests/desktop-bridge-boundary.test.cjs` — locks the settings-only renderer
  boundary.
- `docs/migrating/progress.md` — recorded Task 5.3.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (133 tests across 38 suites), including the new
  settings-only DesktopBridge boundary assertion.
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (39 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; this abstraction-only milestone added
  no renderer permission.
- Electron production-output smoke launch: passed through the unchanged
  preload settings implementations.
- `npm run tauri:dev`: passed. The companion launched with the unavailable
  native settings bridge continuing to use its existing safe defaults; no
  command or capability error appeared.

**Manual verification**

- Electron settings load and renderer subscriptions continued through the
  new narrow views during smoke launch.
- Tauri startup remained behaviorally unchanged pending the native state
  connection in Task 5.4.

**Blockers**

- None.

**Next task**

- Task 5.4 — Startup Settings Loading.

### Task 5.4 — Startup Settings Loading

**Status:** Complete

**Implementation summary**

- Tauri now resolves its application-data `settings.json`, loads or
  materializes it, and manages one synchronized native `SettingsState` before
  installing menus or creating either renderer.
- If the native file is absent, Tauri checks the platform-specific Electron
  `Ducky/settings.json` location and imports one valid snapshot. Native-file
  existence makes the handoff idempotent; the Electron source is never
  modified.
- Startup creates the companion only after state initialization, applies the
  stored always-on-top value, and then creates the tray.
- Added the official backend-only Tauri autostart plugin. Packaged release
  builds apply `launchAtStartup`; development/debug runs intentionally avoid
  changing the developer's login items, matching Electron's `isPackaged`
  guard.
- Added the companion-only `get_runtime_settings` command and one exact
  generated permission. Rust checks the invoking webview label and returns
  only the existing secret-free runtime projection.
- The companion's runtime-settings hook now receives its startup snapshot
  through a narrow Tauri DesktopBridge adapter and subscribes through the
  existing targeted event transport. Mutation/event production remains Task
  5.5.

**Files changed**

- `src-tauri/src/app_state.rs` — native path resolution, one-time Electron
  import selection, loading, and managed state initialization.
- `src-tauri/src/domain/settings/mod.rs` — synchronized state and secret-free
  runtime projection.
- `src-tauri/src/infrastructure/persistence.rs` — idempotent legacy import
  with source-preservation tests.
- `src-tauri/src/commands/settings.rs` and
  `src-tauri/src/commands/mod.rs` — authorized runtime snapshot command and
  registration.
- `src-tauri/src/desktop/settings.rs`,
  `src-tauri/src/desktop/mod.rs`, and `src-tauri/src/lib.rs` — startup order
  and native general-setting application.
- `src-tauri/src/authorization.rs`,
  `src-tauri/capabilities/companion.json`, and
  `src-tauri/permissions/autogenerated/get_runtime_settings.toml` —
  companion-only least privilege.
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` — official backend
  autostart plugin.
- `src/shared/types.ts`, DesktopBridge adapter/contract files,
  `src/renderer/hooks/useRuntimeSettings.ts`, and
  `tests/desktop-bridge-boundary.test.cjs` — snapshot-only runtime settings
  view and Tauri adapter.
- `src/desktop/tauriCommands.ts` — typed runtime snapshot dispatch.
- `tests/tauri-ipc-authorization.test.cjs` — locks the new exact permission.
- `docs/migrating/progress.md` — recorded Task 5.4.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (133 tests across 38 suites).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (41 tests),
  including one-time legacy import, source preservation, secret-free
  projection, defaults, and native store coverage.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed without warnings.
- `npx tauri permission list`: passed and listed the exact generated
  `allow-get-runtime-settings` permission.
- Electron production-output smoke launch: passed with its startup,
  `SettingsService`, login-item guard, and preload IPC unchanged.
- Two Tauri development smoke launches passed. The first imported the
  existing Electron file; the second reused the native file without another
  import. No settings command, authorization, or capability error appeared.
  The known development custom-protocol fallback warning remained unchanged.

**Manual verification**

- Confirmed the native file was created at the Tauri application-data path
  with mode `0600`.
- Confirmed the imported safe fields match Electron defaults/current values
  and credential presence is retained without printing or exposing its
  contents.
- Confirmed the companion starts after both the initial import and a native
  reload.

**Blockers**

- None.

**Next task**

- Task 5.5 — Settings Persistence.

### Task 5.5 — Settings Persistence

**Status:** Complete

**Implementation summary**

- Added serialized native mutations for the companion user name, sticky
  message, Preferences general settings, and notification-sound settings.
- Every mutation validates the complete next document, persists it
  atomically, and only then replaces the synchronized in-memory snapshot.
  Exact no-op updates skip filesystem writes.
- A dedicated mutation mutex prevents concurrent read/modify/write cycles
  from losing updates without holding the shared settings read/write lock
  during filesystem I/O.
- Successful mutations emit the existing targeted
  `runtime-settings:changed` event only after persistence. General native
  behavior is then applied best-effort; a listener or platform-application
  failure cannot roll back an already saved setting.
- Added exact companion-only permissions for user-name and sticky-message
  mutations and a Preferences-only permission for settings updates. Rust
  also checks the invoking renderer label.
- The Preferences mutation boundary owns only Phase 5 infrastructure:
  `general` and `notificationSounds`. Water, updater, AI, model, reminder,
  Pomodoro, and credential fields are retained in the full document but
  rejected at this mutation boundary for their later phases.
- Activated the narrow Tauri companion settings mutation adapter. Electron
  continues through its unchanged preload implementation.

**Files changed**

- `src-tauri/src/domain/settings/mod.rs` — serialized persist-before-publish
  mutation engine, strict Phase 5 patch DTOs, projections, and regression
  tests.
- `src-tauri/src/commands/settings.rs` and
  `src-tauri/src/commands/mod.rs` — role-authorized mutation commands,
  post-persistence native application, and targeted change emission.
- `src-tauri/src/desktop/settings.rs` — reusable post-mutation native general
  settings application.
- `src-tauri/src/authorization.rs`,
  `src-tauri/capabilities/companion.json`,
  `src-tauri/capabilities/preferences.json`, and the three generated
  permission manifests — exact least-privilege command access.
- `src/desktop/tauriCommands.ts` and `src/desktop/tauriBridge.ts` — typed
  companion settings mutations through DesktopBridge.
- `tests/tauri-ipc-authorization.test.cjs` — exact role separation coverage.
- `docs/migrating/progress.md` — recorded Task 5.5.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (133 tests across 38 suites).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (46 tests),
  including persistence-before-publication, no-op write suppression,
  concurrent mutation serialization, deferred-field rejection, and
  secret-redaction coverage.
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed and generated the exact three mutation
  permissions.
- Electron production-output smoke launch: passed through the unchanged
  preload settings implementation.
- `npm run tauri:dev`: passed; Tauri launched with no settings command,
  authorization, or capability error. The known development custom-protocol
  fallback warning remained unchanged.

**Manual verification**

- Native tests prove persisted values survive a repository reload and that
  failed persistence cannot change the shared snapshot.
- Full interactive Preferences save/restart verification remains Task 5.6,
  when its narrow adapter is connected.

**Blockers**

- None.

**Next task**

- Task 5.6 — Preferences Integration.

### Task 5.6 — Preferences Window Integration

**Status:** Complete

**Implementation summary**

- Added a Preferences-only `get_preferences_settings` command that returns
  the existing typed, credential-redacted Preferences projection.
- Connected the Tauri Preferences renderer to native snapshot, mutation, and
  targeted change-event APIs through `PreferencesSettingsBridge`.
- Added runtime-neutral settings capability metadata to DesktopBridge.
  Electron advertises its complete existing Preferences feature set; Tauri
  advertises only Phase 5-owned general and notification-sound settings.
- Hydration, updater, AI, model-explorer, and credential controls remain
  visible but disabled in Tauri and remain fully available in Electron. The
  renderer does not inspect or identify its runtime.
- Added a hook-level capability guard as defense in depth, while Rust remains
  authoritative and rejects deferred or unknown patch fields.
- Updated the Preferences footer only for Tauri to accurately explain which
  settings are active during migration.

**Files changed**

- `src-tauri/src/commands/settings.rs` and
  `src-tauri/src/commands/mod.rs` — Preferences snapshot command and
  registration.
- `src-tauri/src/authorization.rs`,
  `src-tauri/capabilities/preferences.json`, and
  `src-tauri/permissions/autogenerated/get_preferences_settings.toml` —
  Preferences-only least privilege.
- `src/desktop/contracts.ts`, `src/desktop/DesktopBridge.ts`,
  `src/desktop/electronBridge.ts`, `src/desktop/tauriBridge.ts`, and
  `src/desktop/tauriCommands.ts` — runtime-neutral capability metadata and
  typed native Preferences adapter.
- `src/renderer/hooks/usePreferencesSettings.ts` and
  `src/renderer/PreferencesApp.tsx` — native snapshot/mutation consumption
  and deferred-domain capability gating.
- `tests/desktop-bridge-boundary.test.cjs` and
  `tests/tauri-ipc-authorization.test.cjs` — bridge isolation, deferred-domain
  gating, and exact permission coverage.
- `docs/migrating/progress.md` — recorded Task 5.6 and Phase 5 completion.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (134 tests across 38 suites).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (47 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed and generated the exact
  `allow-get-preferences-settings` permission.
- Electron production-output smoke launch: passed through the unchanged
  preload settings APIs. Existing security-policy denial diagnostics remained
  expected and unchanged.
- `npm run tauri:dev`: passed; the companion launched and the process remained
  alive with no settings, command, authorization, or capability error. The
  known development custom-protocol fallback warning remained unchanged.
- A current debug `.app` bundle was built successfully for unambiguous
  interactive verification alongside the separately installed Electron app.

**Manual verification**

- Confirmed Preferences remains hidden on startup and opens only from the
  native companion menu.
- Confirmed it displays the current native general, sound, hydration, update,
  and redacted AI values without exposing the stored credential.
- Confirmed general and notification-sound controls are enabled and save
  immediately; hydration, updater, AI, model, and credential controls are
  disabled only in Tauri.
- Changed eye tracking and notification volume, confirmed the UI returned to
  `Saved`, and confirmed the owner-only native JSON contained both values.
- Restarted through the native menu, confirmed a new process PID, reopened
  Preferences, and confirmed both modified values survived.
- Restored the local eye-tracking and volume values after verification.
- No renderer error, command-scope warning, permission warning, duplicate
  write symptom, or settings-load failure appeared.

**Commits created during Phase 5**

- `20134db feat(tauri): migrate settings store`
- `6b9be4d feat(tauri): migrate settings bridge`
- `2236aa3 feat(tauri): migrate settings startup`
- `05aa581 feat(tauri): migrate settings persistence`
- `feat(tauri): migrate preferences settings`

**Blockers**

- None.

### Phase 5 Completion

**Status:** Complete

- Native settings storage, typed validation, defaults, startup load/import,
  atomic persistence, change notifications, DesktopBridge access, and
  Preferences integration satisfy the Phase 5 contract.
- Electron remains intact and is still authoritative for deferred feature
  domains.
- The repository builds successfully and the Phase 5 automated and manual
  exit criteria pass.

**Next task**

- Phase 6 — Credentials. Do not begin automatically.

---

## Phase 6 — Secure Credentials & Secret Storage

### Task 6.1 — Existing Credential Storage Audit

**Status:** Complete

**Stored-secret inventory**

- The only persisted application secret is the AI API key. Ducky does not
  currently persist login tokens, updater credentials, release credentials,
  or per-provider credential collections.
- `src/main/CredentialManager.ts` owns Electron credential protection. It uses
  Electron `safeStorage`, rejects unavailable encryption, and rejects Linux's
  reversible `basic_text` fallback.
- `src/main/SettingsService.ts` stores a versioned Base64 ciphertext record in
  the `credential` field of Electron's `settings.json`. Older
  `ai.apiKey` plaintext values are retained until a verified encryption
  round-trip and atomic settings rewrite succeed.
- The Phase 5 native settings importer retains both opaque `credential`
  records and any legacy `ai.apiKey` field losslessly. It exposes only the
  boolean `apiKeyConfigured` projection and never returns either value to a
  renderer.

**Credential lifecycle**

- Electron constructs `CredentialManager` after `app.whenReady()`, loads
  `SettingsService`, and attempts the one-time plaintext-to-`safeStorage`
  migration during load.
- Preferences sends a credential only through
  `preferences-ai:configure`; `SettingsService.updateAiConfiguration`
  validates length, encrypts, verifies, and atomically writes the replacement.
  An empty value removes the credential. Unchanged updates retain the existing
  ciphertext.
- AI request construction obtains plaintext only inside Electron main through
  `SettingsService.getApiKey()`. Main never returns plaintext through IPC.
- The Preferences renderer uses an uncontrolled password input. React state
  stores only edited/clear flags and the secret-free configured boolean.

**DesktopBridge and renderer interactions**

- `PreferencesApp.tsx` consumes `preferencesDesktopBridge`; it has no direct
  Electron or Tauri API access.
- The Electron adapter currently forwards the existing typed
  `PreferencesBridge.updateAiConfiguration` method. The Tauri adapter
  deliberately exposes no AI or credential mutation yet.
- Phase 6 will add a credential-specific, runtime-neutral bridge that returns
  status only and accepts create/update/delete requests. Provider settings,
  model selection, AI networking, and diagnostics remain deferred.

**Migration boundary and security decision**

- Electron `safeStorage` ciphertext cannot be assumed decryptable through a
  Rust OS-vault backend. The native implementation will therefore not parse,
  decrypt, log, overwrite, or silently discard imported Electron ciphertext.
- Tauri will store newly entered credentials directly in the platform vault
  (macOS Keychain, with the corresponding native secure backend on supported
  desktop platforms). Preferences will allow safe re-entry and report native
  vault status without returning plaintext.
- The imported opaque record remains preserved for Electron compatibility and
  a future explicitly designed transition handoff. Phase 6 does not claim
  automatic ciphertext portability.

**Discovery blockers**

- None. The source-of-truth architecture explicitly permits safe re-entry when
  Electron ciphertext portability is not demonstrated.

**Next task**

- Task 6.2 — Create the native secret store.

### Task 6.2 — Create Native Secret Store

**Status:** Complete

**Implementation summary**

- Added a runtime-owned Rust credential repository in
  `src-tauri/src/infrastructure/credentials.rs`.
- Added a typed `CredentialId::AiApiKey` key instead of exposing generic
  service/account strings to callers.
- Added the `keyring` 3.6.3 backend using macOS Keychain Services, Windows
  Credential Manager, and Linux Secret Service through target-specific
  features compatible with the repository's Rust 1.77.2 baseline.
- Added strict 4,096-byte validation, whitespace normalization, idempotent
  deletion, overwrite support, and backend-level duplicate-write suppression.
- Plaintext is never formatted into errors or logs. Temporary owned secret
  buffers and values loaded from the backend use `zeroize`.
- Unit tests use an injected in-memory backend and never create or read a real
  operating-system credential.
- Preserved Electron `CredentialManager`, `SettingsService`, preload IPC, and
  all renderer behavior unchanged.

**Files changed**

- `docs/migrating/migration_tasks.md` — retained the user-provided Phase 6
  execution contract as the source of truth.
- `docs/migrating/progress.md` — recorded discovery and Task 6.2.
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` — added target-specific
  native keyring support plus secret zeroization.
- `src-tauri/src/infrastructure/mod.rs` and
  `src-tauri/src/infrastructure/credentials.rs` — native secure-store
  implementation and tests.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (134 tests across 38 suites).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (50 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; Task 6.2 adds no WebView command.
- Electron production-output smoke launch: passed through the unchanged
  preload and `safeStorage` implementation. Existing expected security-policy
  denial diagnostics remained unchanged.
- `npm run tauri:dev`: passed; the companion launched and remained alive with
  no secret-store initialization or renderer error. The known development
  custom-protocol fallback warning remained unchanged.

**Manual verification**

- Real Keychain mutation is intentionally deferred until Task 6.4 registers
  application state and native commands. Task 6.2 save/load/update/delete and
  duplicate-write behavior were verified through the injected backend tests.

**Blockers**

- None.

**Next task**

- Task 6.3 — DesktopBridge credential API.

### Task 6.3 — DesktopBridge Credential API

**Status:** Complete

**Implementation summary**

- Added the shared `CredentialId`, `CredentialState`, and secret-free
  `CredentialStatus` contract.
- Added a narrow `CredentialBridge` with status, save, and delete operations.
  Plaintext load is deliberately absent so a renderer cannot retrieve a stored
  secret.
- Added the Preferences-only DesktopBridge view and capability flag. Renderer
  code still has no direct Electron preload, Tauri invoke, or native keyring
  access.
- Adapted Electron's existing `PreferencesBridge.updateAiConfiguration`
  transport behind the new credential boundary without changing Electron IPC,
  main-process storage, or behavior.
- Left the Tauri credential bridge unavailable until Task 6.4 registers the
  native store and completed commands; no placeholder native command was
  exposed.
- Extended bridge-boundary tests to enforce the narrow API and verify that
  neither runtime accidentally claims an unavailable credential capability.

**Files changed**

- `src/shared/credentials.ts` and `src/shared/types.ts` — typed shared
  credential metadata and bridge contract.
- `src/desktop/contracts.ts`, `src/desktop/DesktopBridge.ts`,
  `src/desktop/electronBridge.ts`, and `src/desktop/tauriBridge.ts` —
  role-scoped runtime adapters.
- `tests/desktop-bridge-boundary.test.cjs` — renderer isolation and
  secret-return regression coverage.
- `docs/migrating/progress.md` — Task 6.3 record.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (134 tests across 38 suites).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (50 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed; Task 6.3 adds no WebView command.
- Electron production-output smoke launch: passed through the unchanged
  preload and protected settings service.
- `npm run tauri:dev`: passed; the companion launched and remained alive with
  the credential capability unavailable by design. The known development
  custom-protocol fallback warning remained unchanged.

**Manual verification**

- Runtime credential UI remains unchanged in this bridge-only milestone.
- Source and automated boundary inspection confirmed the renderer-facing API
  can send an explicitly entered secret but cannot request plaintext back.

**Blockers**

- None.

**Next task**

- Task 6.4 — Credential persistence.

### Task 6.4 — Credential Persistence

**Status:** Complete

**Implementation summary**

- Registered the native `CredentialStore` as Rust-owned application state at
  startup. Its stable service identity matches the application identifier
  `com.ducky.desktop`.
- Added Preferences-only `get_credential_status`, `save_credential`, and
  `delete_credential` commands with exact authorization and generated Tauri
  permissions.
- Connected the Tauri DesktopBridge adapter to those commands with fully typed
  request/result mappings.
- Native status distinguishes `configured`, `missing`, and
  `requiresReentry`. The last state is returned when Phase 5 preserved an
  Electron ciphertext or legacy plaintext record but no native vault entry
  exists. The opaque value is neither decrypted nor returned.
- Native save validates and normalizes before persistence, safely overwrites,
  skips duplicate values, and zeroizes the command-owned string. Delete is
  idempotent and removes only the native secret.
- Credential metadata contains only an ID and state. The native load API stays
  internal for the later AI migration and no plaintext load command exists.
- Added persistence tests covering save, load, replacement, deletion,
  duplicate-write suppression, store reconstruction, invalid values, safe
  re-entry detection, and secret-free serialization.

**Files changed**

- `src-tauri/src/app_state.rs` and
  `src-tauri/src/infrastructure/credentials.rs` — native state ownership and
  persistence lifecycle.
- `src-tauri/src/commands/credentials.rs`,
  `src-tauri/src/commands/mod.rs`, and
  `src-tauri/src/authorization.rs` — narrow native command surface.
- `src-tauri/capabilities/preferences.json` and generated credential
  permissions — Preferences-only least privilege.
- `src/desktop/tauriCommands.ts` and `src/desktop/tauriBridge.ts` — typed
  runtime adapter.
- `tests/desktop-bridge-boundary.test.cjs` and
  `tests/tauri-ipc-authorization.test.cjs` — bridge and permission regression
  coverage.
- `docs/migrating/progress.md` — Task 6.4 record.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (134 tests across 38 suites).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (54 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed and generated exact allow/deny pairs for
  the three credential commands.
- Electron production-output smoke launch: passed with its original
  `safeStorage` and IPC behavior unchanged.
- `npm run tauri:dev`: passed; native credential state and commands registered,
  the companion launched, and no authorization or store error appeared. The
  known development custom-protocol fallback warning remained unchanged.

**Manual verification**

- Native command registration and restart semantics were verified through
  typed adapter, authorization, state-reconstruction, and persistence tests.
- End-to-end real Keychain create/update/restart/delete verification is
  intentionally performed through Preferences in Task 6.5 so no temporary
  testing entry bypasses the product path.

**Blockers**

- None.

**Next task**

- Task 6.5 — Preferences integration.

### Task 6.5 — Preferences Integration

**Status:** Complete

**Implementation summary**

- Connected the existing Preferences credential control to the
  runtime-neutral `CredentialBridge`. The renderer still has no direct
  Electron, Tauri, or operating-system vault access.
- Preferences now loads secret-free native credential status alongside the
  settings projection and distinguishes configured, missing, and safe
  re-entry states.
- Tauri can create, replace, and remove the AI API credential while the
  provider, model, diagnostics, and other Phase 10 AI behavior remain
  intentionally unavailable.
- Preserved the uncontrolled password input: plaintext exists only in the DOM
  input long enough to submit the explicit mutation and is never copied into
  React state.
- Preserved Electron behavior through the existing
  `updateAiConfiguration`/`safeStorage` implementation. The shared
  Preferences UI selects behavior from DesktopBridge capabilities rather than
  detecting a runtime.
- After an explicit successful native save or delete, Tauri removes only its
  imported copy of legacy Electron credential data. The original Electron
  settings file remains untouched. This prevents deleted native credentials
  from reverting to `requiresReentry` on restart and avoids retaining an
  unnecessary plaintext or opaque ciphertext copy in native settings.
- Added renderer-boundary regression coverage for credential access through
  DesktopBridge and for the secret-free, uncontrolled input lifecycle.

**Files changed**

- `src/renderer/hooks/usePreferencesSettings.ts` — secret-free credential
  status loading and create/update/delete controller operations.
- `src/renderer/PreferencesApp.tsx` — capability-gated native credential
  Preferences flow with safe re-entry messaging.
- `src-tauri/src/commands/credentials.rs` and
  `src-tauri/src/domain/settings/mod.rs` — explicit cleanup of the imported
  native settings copy after a successful vault transition.
- `tests/desktop-bridge-boundary.test.cjs` — runtime-agnostic bridge and
  renderer-state regression assertions.
- `docs/migrating/progress.md` — Task 6.5 and Phase 6 completion record.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (134 tests across 38 suites).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: passed (55 tests).
- `cargo build --manifest-path src-tauri/Cargo.toml`: passed.
- `npx tauri permission list`: passed with the exact Preferences-only
  credential command permissions.
- `npm run tauri:dev`: passed; the companion launched and remained alive with
  no credential, permission, or renderer errors. The known development
  custom-protocol fallback warning remained unchanged.
- `npm run tauri:build -- --debug`: passed and produced the current macOS
  application bundle used for isolated manual Keychain verification.
- Electron production-output smoke launch: passed. The companion and
  Preferences window opened through the unchanged Electron preload,
  `safeStorage`, and AI configuration path. Existing expected permission
  denial diagnostics remained unchanged.

**Manual verification**

- Created a synthetic test credential through Tauri Preferences and confirmed
  the UI returned to `Saved` with configured status.
- Confirmed the credential was written to macOS Keychain under the native
  application service and was absent from the native settings JSON.
- Replaced the credential through the same Preferences control and verified
  the Keychain value changed without exposing it in renderer output.
- Quit and relaunched Ducky, reopened Preferences, and confirmed configured
  status survived restart.
- Removed the credential through Preferences, confirmed the Keychain item was
  deleted, relaunched again, and confirmed Preferences remained in the
  missing state.
- Confirmed native settings contained neither the synthetic secret nor legacy
  credential fields after the explicit transition. The synthetic Keychain
  entry was removed after verification.
- Confirmed Electron Preferences still reported its independently configured
  credential without reading, replacing, or deleting it.
- Accessibility output and runtime logs contained only redacted status; no
  plaintext credential or renderer console error was observed.

**Commit**

- `feat(tauri): migrate credential preferences`

**Blockers**

- None.

## Phase 6 Completion

**Status:** Complete

- Native platform secret storage is implemented with a typed Rust-owned
  repository.
- DesktopBridge owns all renderer credential access and exposes no plaintext
  read operation.
- Native credentials save, update, persist across restart, and delete through
  Preferences.
- Imported Electron credential material has an explicit safe re-entry path;
  Electron's source settings and `safeStorage` implementation remain
  unchanged.
- Least-privilege Preferences permissions, automated validation, production
  builds, runtime smoke launches, and manual macOS Keychain verification all
  passed.

**Phase 6 commits**

- `55cd4b5 feat(tauri): migrate secret store`
- `724d939 feat(tauri): migrate credential bridge`
- `28458de feat(tauri): migrate credential persistence`
- `feat(tauri): migrate credential preferences`

**Next task**

- Phase 7 — Reminder System. Do not begin automatically.

### Task 7.1 — Audit Existing Reminder System

**Status:** Complete. The discovered contract conflicts were resolved through
the strict Electron-parity clarification. Task 7.2 has not started.

**Electron reminder lifecycle**

- `src/main/ReminderService.ts` is the authoritative reminder domain service.
  It serializes all mutations, creates UUIDs, validates input, performs CRUD,
  advances recurrence, marks one-time reminders complete, preserves recurring
  reminder IDs, sorts reminders deterministically, and persists through
  `SettingsService`.
- `src/shared/reminders.ts` defines the persisted schema:
  `id`, `title`, `message`, `scheduledAt`, `recurrence`,
  `lastTriggeredAt`, `nextOccurrence`, `completed`, `createdAt`, and
  `updatedAt`.
- The schema and Electron service have no per-reminder `enabled` field and no
  enable/disable operation. An incomplete reminder is schedulable; a completed
  one-time reminder is not.
- `src/shared/reminderRecurrence.ts` implements `none`, `hourly`, `daily`,
  `weekly`, `monthly`, and custom interval recurrence with calendar-aware
  advancement where applicable.

**Scheduler and startup restoration**

- `src/main/ReminderScheduler.ts` owns exactly one wake timer. It validates
  the wall clock at most every 60 seconds, processes reminders no more than 24
  hours overdue, suppresses duplicate delivery, advances recurring reminders,
  retries failed completion persistence without re-emitting the notification,
  and resynchronizes after reminder changes.
- `src/main/main.ts` creates `ReminderService` and `ReminderScheduler` after
  settings load, subscribes delivery before starting the scheduler, and starts
  it before renderer window initialization. `powerMonitor.resume` triggers
  resynchronization. Shutdown removes the resume listener, stops the
  scheduler, unsubscribes events, and clears the pending delivery queue.
- Startup restoration is therefore settings restoration followed by a
  scheduler rescan; there is no separate reminder database.

**Persistence**

- Electron stores reminders inside `settings.json` through
  `src/main/SettingsService.ts`. Writes use a same-directory temporary file,
  mode `0600`, and atomic rename. Mutations are serialized and listeners run
  only after persistence succeeds.
- Strict parsing in `src/shared/settings.ts` and
  `src/shared/reminders.ts` rejects malformed or duplicate reminder records
  and preserves deterministic ordering.
- The native settings document in
  `src-tauri/src/domain/settings/mod.rs` currently retains reminders as
  deferred opaque JSON objects. Phase 7 must replace that deferred
  representation with the exact typed Electron schema without changing it.

**IPC and DesktopBridge**

- Electron exposes companion-only commands for create, update, delete, get,
  list, and mark-completed through `src/shared/events.ts`,
  `src/main/main.ts`, `src/main/preload.ts`, and
  `src/main/ipcAuthorization.ts`.
- `src/shared/types.ts` currently places those operations and the fired/panel
  listeners on the broad legacy `CompanionBridge`.
- `src/desktop/DesktopBridge.ts` is the renderer boundary. The Electron
  adapter returns the existing preload bridge; the Tauri adapter intentionally
  returns no `CompanionBridge` because reminder-domain commands have not been
  migrated.
- `src/desktop/tauriEvents.ts` already reserves exact companion-targeted
  routes for creation-panel requested, manager-panel requested, and
  `reminders:fired`. No created, updated, deleted, enabled, or disabled event
  exists in the Electron contract.
- `src/desktop/tauriCommands.ts` and the native command registry contain no
  reminder commands yet.

**Notification flow**

- `src/main/ReminderEvents.ts` defines only the `reminder-fired` domain event.
  The Electron main process converts it to `ReminderFiredNotification`, queues
  it until the companion renderer is loaded, and sends it over the targeted
  preload event.
- `src/main/preload.ts` adds a second short-lived buffer so a fired reminder
  is not lost before React registers its listener.
- `src/renderer/hooks/useReminderNotifications.ts` deduplicates and queues
  fired reminders. `src/renderer/components/ReminderWidget.tsx` renders the
  in-app notification with Dismiss and Snooze; snooze creates a new one-time
  reminder. `src/renderer/App.tsx` plays the configured reminder sound through
  `NotificationSoundService`.
- Electron does not instantiate its native `Notification` API. There is no OS
  notification permission, native notification title/body contract, or
  native notification click callback in the reference implementation.

**Renderer, Preferences, and tray integration**

- Smart-reminder creation, schedule/recurrence editing, listing, and deletion
  live in companion renderer panels:
  `ReminderCreationPanel.tsx`, `ReminderManagerPanel.tsx`, and `App.tsx`.
- `PreferencesApp.tsx` has hydration settings and notification-sound settings,
  but no smart-reminder CRUD, schedule, or per-reminder enable/disable
  controls.
- Electron's companion context menu opens the New Reminder and Manage
  Reminders panels through native actions in `src/main/menus.ts`.
- The Phase 4 Tauri menu deliberately deferred those reminder-owned actions.
  The current Phase 7 task list does not state whether restoring those context
  menu entries is part of Phase 7, despite their being required to reach the
  Electron reminder UI.

**Permissions**

- Electron authorizes reminder CRUD only for the companion renderer.
- Tauri's companion capability has no reminder command permissions yet and
  Preferences has none, as expected before Phase 7.
- No Tauri notification plugin or notification capability is installed.
  `migration_codex.md` explicitly classifies native notifications as
  `NOT NEEDED` and warns against adding the plugin or an OS permission prompt.

**Architecture clarification**

The discovered source-of-truth conflicts are resolved by the migration-wide
parity rule: when migration documents conflict, the existing Electron
implementation is authoritative unless an explicit redesign was approved
before implementation.

The corrected Phase 7 contract now requires:

1. The exact Electron reminder schema, without an `enabled` field.
2. The existing Rust-owned scheduler migration without recurrence,
   persistence, ordering, overdue, retry, or duplicate-suppression changes.
3. The existing Companion CRUD panels and the existing New Reminder and
   Manage Reminders context-menu entry points; Preferences remains unchanged.
4. Only the existing `reminders:fired`,
   `reminders:creation-panel-requested`, and
   `reminders:manager-panel-requested` event contracts.
5. The existing React reminder widget, Dismiss, Snooze, and notification-sound
   behavior. Native OS notifications, click callbacks, plugins, and
   permissions are prohibited.
6. Only the existing reminder operations through DesktopBridge; no additional
   APIs or renderer runtime detection.

`migration_tasks.md` now assigns Companion/context-menu parity explicitly and
removes all feature-expansion requirements. No production code was changed and
Task 7.2 was not started.

**Files changed**

- `docs/migrating/migration_codex.md` — added the migration-wide parity rule.
- `docs/migrating/migration_tasks.md` — aligned the full Phase 7 execution
  contract and exit criteria with Electron.
- `docs/migrating/progress.md` — recorded the discovery resolution and cleared
  the documentation blocker.

**Validation performed**

- Full source review completed for reminder domain, scheduler, persistence,
  startup/shutdown, IPC authorization, preload buffering, DesktopBridge
  adapters, renderer panels/hooks/widget, Preferences, tray/context menu,
  native settings, command/event registries, and capabilities.
- Documentation diff validation is recorded below. No production files or
  dependencies changed, so build/runtime validation was not required.

**Manual verification**

- Not applicable to discovery. No implementation was attempted.

**Commit**

- None, as required by Task 7.1.

**Blockers**

- None. The architecture clarification resolves the discovery findings.

**Next task**

- Task 7.2 — Create Native Reminder Engine, only when explicitly requested.

## Phase 7 Architecture Clarification

**Status:** Complete. Documentation only; Phase 7 implementation remains
unstarted.

**Summary**

- Added a migration-wide rule making the existing Electron implementation
  authoritative when migration documents conflict and no redesign was
  explicitly approved.
- Corrected Phase 7 to preserve the current schema, scheduler, recurrence,
  persistence, Companion CRUD UI, DesktopBridge surface, reminder events,
  React notification widget, Dismiss/Snooze flow, sound behavior, and existing
  native context-menu actions.
- Removed requirements for per-reminder enable/disable, native OS
  notifications, notification click callbacks, new lifecycle events, and
  Preferences reminder management.

**Production code**

- No production source file was modified.
- Task 7.2 was not started.

**Validation**

- `git diff --check`: passed.
- The documentation diff contains only
  `docs/migrating/migration_codex.md`,
  `docs/migrating/migration_tasks.md`, and
  `docs/migrating/progress.md`.

**Commit**

- `docs(tauri): align phase 7 with electron parity`

**Next task**

- Task 7.2 — Create Native Reminder Engine. Do not begin automatically.

### Task 7.2 — Native Reminder Engine Core

**Status:** Engine core complete; native settings registration and startup
restoration continue in Task 7.3 before Task 7.2's runtime acceptance is
closed.

**Implementation summary**

- Added the exact Electron reminder schema to the Rust domain, including
  one-time, fixed interval, daily, weekly, and calendar-month recurrence.
  No `enabled` field or new reminder capability was introduced.
- Migrated `ReminderService` semantics to Rust: serialized mutations, strict
  validation, UUID generation, stable sorting, CRUD, one-time completion, and
  recurring occurrence advancement.
- Migrated `ReminderScheduler` as a runtime-owned singleton with one native
  worker, a 60-second wall-clock validation bound, 24-hour overdue recovery,
  deterministic due ordering, persistence-failure retry behavior, and
  duplicate-delivery suppression.
- Kept scheduling independent of the renderer and modeled persistence and
  fired-event delivery behind narrow native traits. The concrete native
  settings repository and Tauri event sink are intentionally connected by the
  following Phase 7 milestones, not replaced by browser timers or native OS
  notifications.

**Files changed**

- `src-tauri/src/domain/reminders/mod.rs` — exact reminder model, service,
  recurrence engine, scheduler, native interfaces, and focused regression
  tests.
- `src-tauri/src/domain/mod.rs` — registers the reminder domain.
- `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock` — add direct `chrono`
  calendar/time support and UUID generation dependencies already present in
  the native dependency graph.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed.
- `npm run build`: passed.
- `cargo fmt --check`: passed.
- `cargo test`: passed (59 tests, including four reminder engine tests).
- `cargo build`: passed. The unregistered engine reports expected dead-code
  warnings until the settings/runtime integration milestone.
- `npx tauri permission list`: passed; the native-only engine adds no renderer
  permission.
- Electron production-output smoke launch: passed through the unchanged
  Electron reminder implementation.
- `npm run tauri:dev`: passed; Tauri compiled, launched, and remained alive.
  Existing development custom-protocol fallback warnings were unchanged.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  macOS application bundle.

**Manual verification**

- Runtime reminder behavior is not yet exposed in this core-only milestone.
  Manual CRUD, fired-widget, persistence, and restart checks remain deferred
  until the concrete repository, commands, and event delivery are connected.

**Blockers**

- None.

**Next task**

- Task 7.3 — connect the exact reminder schema to the native settings store,
  register/start/stop the scheduler, and restore reminders on startup. Do not
  begin Task 7.4 until persistence acceptance passes.

### Tasks 7.2–7.3 — Reminder Runtime Registration and Persistence

**Status:** Complete.

**Implementation summary**

- Replaced the native settings document's deferred reminder JSON with the
  exact typed Electron reminder schema. Stored records retain the Electron
  compatibility defaults for omitted legacy `recurrence`,
  `lastTriggeredAt`, and `nextOccurrence` fields, while malformed, unknown,
  or duplicate records are rejected.
- Implemented the reminder repository on the existing shared
  `SettingsState`. Reminder writes use the established mutation mutex,
  validation pass, same-directory temporary file, owner-only permissions, and
  atomic rename before the shared snapshot changes.
- Made cloned native settings handles share the same locks and snapshot so the
  scheduler, commands, and existing settings commands cannot diverge or lose
  concurrent mutations.
- Registered one `ReminderRuntime` during Tauri setup. It restores the typed
  reminder list from native settings, starts exactly one scheduler worker
  before renderer initialization, resynchronizes on native resume, and joins
  cleanly during application exit.
- Added a bounded native pending-delivery queue as the scheduler sink. This
  preserves reminders that become due before renderer event delivery is
  connected in Task 7.5; it does not add an OS notification or alter the
  existing React notification contract.

**Files changed**

- `src-tauri/src/domain/reminders/mod.rs` — Electron-compatible stored reminder
  parsing, pending delivery queue, and registered reminder runtime.
- `src-tauri/src/domain/settings/mod.rs` — typed reminder settings,
  clone-shared state locks, atomic reminder repository, and persistence/schema
  tests.
- `src-tauri/src/infrastructure/persistence.rs` — typed reminder round-trip
  coverage.
- `src-tauri/src/app_state.rs` — constructs, starts, and manages the singleton
  reminder runtime from the shared settings repository.
- `src-tauri/src/desktop/lifecycle.rs` — native resume resynchronization and
  clean scheduler shutdown.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (134 tests).
- `npm run build`: passed.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (62 tests, including legacy schema defaults, duplicate
  rejection, shared-state atomic persistence, and settings round trips).
- `cargo build`: passed.
- `npx tauri permission list`: passed; runtime-owned scheduling and persistence
  require no renderer permission.
- Electron production-output smoke launch: passed through the unchanged
  Electron reminder service, scheduler, persistence, and renderer.
- `npm run tauri:dev`: passed. The app loaded native settings, registered the
  singleton scheduler, launched, and remained alive without reminder or
  permission errors. Existing development custom-protocol fallback warnings
  were unchanged.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  macOS application bundle.

**Manual verification**

- Tauri startup and shutdown were exercised with the runtime registered.
- End-to-end reminder creation/restart verification remains intentionally
  pending until Task 7.4 exposes the existing CRUD bridge. Atomic save/reload
  and startup schema restoration are covered by focused native tests in this
  milestone.

**Blockers**

- None.

**Next task**

- Task 7.4 — expose only the existing reminder CRUD contract through the
  companion-scoped DesktopBridge. Do not begin Task 7.5 until command
  acceptance passes.

### Task 7.4 — Reminder Commands and DesktopBridge

**Status:** Complete. Command permissions are generated but remain absent from
the Companion capability until the dedicated least-privilege Task 7.8
milestone.

**Implementation summary**

- Added typed Rust commands for the exact existing create, update, delete,
  get, list, and mark-completed reminder operations. Each command authorizes
  the invoking WebView label before accessing the shared native
  `ReminderRuntime`.
- Added stable, non-sensitive command errors that distinguish invalid input,
  missing records, persistence failure, runtime unavailability, and
  unauthorized windows.
- Added a narrow `ReminderBridge` to DesktopBridge. Electron continues to
  adapt its unchanged preload API, while Tauri dispatches only the six
  existing reminder commands and subscribes only to the three existing
  reminder events.
- Moved reminder CRUD, Snooze creation, fired-notification subscription, and
  reminder panel subscriptions in the renderer from the broad deferred
  `CompanionBridge` to `getReminderBridge()`. React remains runtime agnostic
  and imports no Tauri API.
- Registered the command names in the native authorization manifest and
  generated exact allow/deny permission pairs. No capability grant was added
  early; Task 7.8 remains the single activation point for renderer authority.

**Files changed**

- `src-tauri/src/commands/reminders.rs` and
  `src-tauri/src/commands/mod.rs` — typed command implementations and native
  dispatch registration.
- `src-tauri/src/authorization.rs` — exact Companion-only authorization
  metadata and build-manifest registration.
- `src-tauri/permissions/autogenerated/*reminder*.toml` — generated narrow
  allow/deny pairs for the six commands.
- `src/shared/types.ts`, `src/desktop/contracts.ts`,
  `src/desktop/DesktopBridge.ts`, `src/desktop/electronBridge.ts`,
  `src/desktop/tauriBridge.ts`, and `src/desktop/tauriCommands.ts` — exact
  reminder bridge contract and dual-runtime adapters.
- `src/renderer/App.tsx` and
  `src/renderer/hooks/useReminderNotifications.ts` — reminder-only bridge
  consumption without UI or behavior changes.
- `tests/desktop-bridge-boundary.test.cjs` and
  `tests/tauri-ipc-authorization.test.cjs` — runtime-boundary and registered
  command-permission regression coverage.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (135 tests). An initial expected-manifest failure exposed
  the six newly generated files; the test now distinguishes registered
  commands from capability grants.
- `npm run build`: passed.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (63 tests).
- `cargo build`: passed.
- `npx tauri permission list`: passed and listed all six exact reminder
  allow-permission definitions.
- Electron production-output smoke launch: passed with its original preload
  and reminder implementation.
- `npm run tauri:dev`: passed. The exact reminder bridge event listeners
  registered without renderer, reminder, or permission errors; existing
  development custom-protocol fallback warnings were unchanged.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  macOS application bundle.

**Manual verification**

- No visual UI changes were made. CRUD activation in the running Tauri
  renderer is intentionally held until Task 7.8 grants the already generated
  permissions.

**Blockers**

- None.

**Next task**

- Task 7.5 — connect queued scheduler deliveries to the exact
  `reminders:fired` Companion event with renderer startup/reload recovery and
  duplicate suppression.

### Task 7.5 — Reminder Fired-Event Delivery

**Status:** Complete.

**Implementation summary**

- Connected the native scheduler sink to the existing
  `reminders:fired` Tauri event and targeted delivery exclusively at the
  Companion window. The event name, payload, React reminder widget, Dismiss,
  Snooze, and configured notification-sound behavior remain unchanged.
- Replaced the temporary persistence-era delivery buffer with an ordered
  activation-aware queue. Reminders that fire before the renderer listener is
  ready, or while the Companion WebView reloads, remain queued until the
  renderer explicitly confirms listener registration.
- Added an internal Companion-only activation handshake after Tauri event
  listener registration. This handshake is adapter infrastructure and does
  not expand the public `ReminderBridge` contract or expose Tauri APIs to
  React.
- Added reload generation tracking, retry after emit failure, and serialized
  queue flushing so a renderer transition cannot reorder pending reminder
  notifications or cause concurrent duplicate drains.
- Kept the implementation entirely in-process. No native OS notification
  plugin, permission, schema change, browser timer, or new reminder lifecycle
  event was introduced.

**Files changed**

- `src-tauri/src/domain/reminders/mod.rs` — activation-aware pending delivery
  queue, native delivery callback, reload/retry handling, and focused tests.
- `src-tauri/src/app_state.rs` and `src-tauri/src/lib.rs` — exact Companion
  event sink wiring and WebView reload deactivation.
- `src-tauri/src/commands/reminders.rs`,
  `src-tauri/src/commands/mod.rs`, and
  `src-tauri/src/authorization.rs` — internal event-listener activation
  command and exact Companion authorization.
- `src-tauri/permissions/autogenerated/activate_reminder_events.toml` and
  `src-tauri/capabilities/companion.json` — generated permission pair and the
  one minimum grant required for startup/reload delivery recovery.
- `src/desktop/tauriBridge.ts`, `src/desktop/tauriCommands.ts`, and
  `src/desktop/tauriEvents.ts` — runtime-adapter listener-registration
  handshake without renderer runtime detection.
- `tests/tauri-ipc-authorization.test.cjs` — exact permission/capability
  regression coverage.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (135 tests).
- `npm run build`: passed.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (65 tests, including startup/reload ordered delivery
  and failed-emission retry).
- `cargo build`: passed.
- `npx tauri permission list`: passed and listed the exact
  `allow-activate-reminder-events` permission.
- Electron production-output smoke launch: passed with the original preload,
  reminder event, widget, Snooze, Dismiss, and sound implementation unchanged.
- `npx tauri dev --no-watch`: passed; the native runtime launched, the
  Companion listener registered, and no reminder or permission error was
  reported. Existing development custom-protocol fallback warnings were
  unchanged.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  macOS application bundle.

**Manual verification**

- Both Electron and Tauri runtimes launched and remained alive with the event
  infrastructure registered.
- End-to-end fired-widget, Dismiss, Snooze, sound, and recurrence checks remain
  part of the Phase 7 final manual parity gate after reminder commands and
  context-menu entry points receive their dedicated capability grants.

**Blockers**

- None.

**Next task**

- Task 7.6 — complete the two existing reminder panel-request event producers,
  followed by Task 7.7 Companion context-menu integration. Do not begin
  Task 7.8 until those exact Electron entry points pass validation.

### Tasks 7.6–7.7 — Reminder Events and Companion Integration

**Status:** Complete.

**Implementation summary**

- Restored the exact Electron reminder entry points in the native Companion
  context menu: `Personal Assistant` → `Reminders` →
  `New Reminder…` / `Manage Reminders…`.
- Kept each native callback entirely in Rust. It shows and focuses the
  Companion window, then emits exactly one existing
  `reminders:creation-panel-requested` or
  `reminders:manager-panel-requested` event to that window.
- Reused the Phase 3 event registry and the Task 7.4 narrow
  `ReminderBridge` subscriptions. The existing Companion React effects remain
  the owners of panel presentation and reminder CRUD UI.
- Preserved the Phase 4 tray menu and static context-menu actions unchanged.
  Neighboring Personal Assistant, Pomodoro, Planner, Sticky Message, water,
  and runtime-settings entries remain assigned to their owning migration
  phases.
- Added no renderer runtime detection, Tauri import in React, reminder
  lifecycle event, Preferences UI, or native notification behavior.

**Files changed**

- `src-tauri/src/desktop/menus.rs` — exact reminder submenu structure, native
  action IDs, Companion focus/event dispatch, closed action mapping, and menu
  parity tests.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (135 tests).
- `npm run build`: passed.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (65 tests, including exact reminder submenu labels,
  unique action IDs, and closed native action dispatch).
- `cargo build`: passed.
- `npx tauri permission list`: passed; native menu callbacks require no new
  renderer permission and renderer event emission remains denied.
- Electron production-output smoke launch: passed with its original reminder
  context menu and panel events unchanged.
- `npx tauri dev --no-watch`: passed; the native context menu compiled,
  launched, and reported no menu, event, renderer, or permission error.
  Existing development custom-protocol fallback warnings were unchanged.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  macOS application bundle.

**Manual verification**

- Electron and Tauri runtime smoke launches passed.
- Direct visual activation of both native reminder menu entries remains in the
  Phase 7 final manual parity gate, after Task 7.8 enables the CRUD commands
  used by the opened panels.

**Blockers**

- None.

**Next task**

- Task 7.8 — grant only the six existing reminder CRUD commands to the exact
  Companion capability, then run the complete Phase 7 parity gate.

### Task 7.8 — Reminder Permissions

**Status:** Complete.

**Implementation summary**

- Granted the six existing reminder CRUD commands—create, update, delete,
  get, list, and mark-completed—only to the exact local `companion` WebView.
- Kept the internal fired-event activation handshake Companion-only and
  retained the existing listen/unlisten-only event authority.
- Kept Preferences unable to invoke reminder commands and kept both renderers
  unable to emit native events.
- Extended the capability regression test to reject notification permissions
  explicitly. No wildcard, remote, filesystem, HTTP, shell, process,
  clipboard, global-shortcut, menu, tray, or notification permission was
  added.

**Files changed**

- `src-tauri/capabilities/companion.json` — six exact reminder CRUD grants.
- `tests/tauri-ipc-authorization.test.cjs` — exact completed Companion grant
  set and explicit notification-permission prohibition.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (135 tests).
- `npm run build`: passed.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (65 tests).
- `cargo build`: passed.
- `npx tauri permission list`: passed; all reminder grants resolve to their
  generated command pairs and no notification capability is present.
- Electron production-output smoke launch: passed with Electron authorization
  unchanged.
- `npx tauri dev --no-watch`: passed with no reminder or capability errors.
  Existing development custom-protocol fallback warnings were unchanged.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  macOS application bundle.

**Manual verification**

- Both runtimes launched under the completed permission set without command,
  event, renderer, or permission errors.
- Full reminder CRUD, restart persistence, fired-widget, Dismiss, Snooze,
  sound, recurrence, and context-menu parity verification follows as the final
  Phase 7 gate.

**Blockers**

- None.

**Next task**

- Phase 7 final manual parity verification. Do not begin Phase 8.

## Phase 7 — Reminder System Migration Complete

**Status:** Complete. Phase 8 has not started.

**Implementation summary**

- Migrated the exact Electron reminder model, validation, stable ordering,
  CRUD semantics, recurrence advancement, overdue recovery, persistence retry,
  and duplicate suppression into a singleton native Rust reminder runtime.
- Connected that runtime to the existing atomic native settings store,
  startup restoration, native resume resynchronization, and clean shutdown.
- Added the exact six existing reminder commands behind a narrow
  `ReminderBridge`. Electron retains its original preload implementation;
  Tauri dispatches typed commands; React remains runtime agnostic.
- Preserved the exact `reminders:fired`,
  `reminders:creation-panel-requested`, and
  `reminders:manager-panel-requested` contracts and exact Companion targeting.
  Startup/reload delivery is ordered, retryable, and activated only after the
  renderer listener is registered.
- Restored the Electron reminder context-menu hierarchy and callbacks while
  keeping reminder creation, editing, management, widget presentation,
  Dismiss, Snooze, and sound behavior in the existing Companion React UI.
- Applied only exact local Companion command grants and listen/unlisten event
  authority. Preferences has no reminder command authority, renderers cannot
  emit native events, and no OS notification plugin or permission was added.
- Preserved the exact Electron schema. No `enabled` field, Preferences
  reminder manager, native notification, new lifecycle event, browser
  scheduler, or later-phase feature was introduced.

**Commits created**

- `cbde02e` — `feat(tauri): migrate reminder engine`
- `426c02c` — `feat(tauri): migrate reminder persistence`
- `8d65274` — `feat(tauri): migrate reminder commands`
- `51c3997` — `feat(tauri): migrate reminder notifications`
- `fd8903e` — `feat(tauri): migrate reminder companion integration`
- `36c731d` — `feat(tauri): migrate reminder permissions`

**Automated validation**

- `npm run typecheck`: passed.
- `npm test`: passed (135 tests).
- `npm run build`: passed.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (65 tests).
- `cargo build`: passed.
- `npx tauri permission list`: passed.
- Electron production-output smoke launch: passed.
- Tauri development smoke launch: passed.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  macOS application bundle.
- The unchanged Tauri development custom-protocol fallback warning remained
  present; there were no reminder, renderer, command, event, or permission
  errors.

**Manual parity verification**

Verification was performed against the exact workspace bundle at
`src-tauri/target/debug/bundle/macos/Ducky.app`, not the older installed
`/Applications/Ducky.app` with the same bundle identifier.

- New Reminder context-menu action: passed. The existing Companion creation
  panel opened.
- Reminder creation and loading: passed. A one-time reminder with a title,
  message, and future schedule saved and appeared in Manage Reminders.
- Reminder editing: passed. Title and schedule changes persisted and were
  reflected immediately.
- Restart restoration: passed. The edited reminder remained present after the
  native Restart action recreated the application.
- One-time scheduling and fired delivery: passed. The reminder fired once and
  displayed the existing React widget with the exact stored title and message.
- Snooze: passed. `Snooze 5 min` dismissed the widget and created the existing
  five-minute one-time occurrence in Manage Reminders.
- Recurrence: passed. The snoozed reminder was edited to Hourly, fired at the
  scheduled occurrence, and advanced to the same minute in the next hour
  without changing its visible identity.
- Dismiss: passed. The recurring fired widget closed and the existing
  personality acknowledgement appeared.
- Delete and persistence: passed. Both Phase 7 test reminders were deleted
  through the existing confirmation flow and disappeared from the native
  store. The user's pre-existing `Creators session` reminder was preserved.
- Manage Reminders context-menu action: passed. The existing Companion manager
  panel opened and correctly grouped upcoming/completed reminders.
- Notification sound: passed. Notification Sounds remained enabled with Soft
  Bell at 70%; the fired reminder exercised the unchanged reminder sound path,
  and the dedicated Test Sound control reported `Soft Bell is playing.`
- Preferences: passed. It retained the existing General, Notification Sounds,
  Hydration, Updates, and AI sections with no reminder CRUD or schema UI.
- Runtime integrity: passed. No renderer console error, reminder error, or
  permission warning appeared during the workspace-bundle verification.

**Blockers**

- None.

**Phase 7 exit criteria**

- Native reminder engine, Electron-parity scheduler, typed persistence,
  startup restoration, CRUD, fired event delivery, React widget behavior,
  Dismiss, Snooze, sound path, Companion panels/context menu, exact schema,
  DesktopBridge boundary, least-privilege permissions, Electron preservation,
  automated validation, manual verification, and repository builds all pass.
- Phase 7 is complete.

**Next task**

- Phase 8 — Pomodoro. Do not begin automatically.

## Phase 8 — Pomodoro Migration

### Task 8.1 — Audit Existing Pomodoro System

**Status:** Complete. The Phase 8 contract is aligned with strict Electron
parity; production implementation has not started.

**Electron timer lifecycle and state**

- `src/main/PomodoroManager.ts` is the authoritative timer service. It owns
  one focus session, one scheduled wake timer, serialized persistence, state
  listeners, and completion listeners.
- `src/shared/pomodoro.ts` defines the exact state:
  `running`, `paused`, `selectedDurationMinutes`, `durationMinutes`,
  `remainingSeconds`, and `startedAt`.
- A session starts with a whole-minute duration from 1 through 720. Electron
  exposes 25, 50, and 90 minute presets plus the existing custom-duration
  panel.
- Starting while a session is active replaces that session. Pause first
  materializes elapsed wall-clock time and then freezes the remaining time.
  Resume keeps the frozen remaining time and starts a new wall-clock
  baseline. Stop returns to an idle state while retaining the selected
  duration.
- The manager has no work/break mode, short break, long break, reset, skip,
  automatic cycle, session counter, or Preferences integration.

**Scheduling and time materialization**

- Electron schedules exactly one timeout aligned to the next one-second
  boundary. Every wake materializes remaining time from `Date.now()` and the
  persisted `startedAt` baseline rather than decrementing stored state.
- Delayed timers, sleep, renderer closure, and relaunch therefore reconcile
  from elapsed wall-clock time. `powerMonitor.resume` does not invoke a
  special Pomodoro operation; the deadline-based tick/get-state path is
  authoritative.
- Completion clears the timer, persists the idle state, emits the idle state,
  and then invokes completion listeners exactly once.

**Persistence and startup restoration**

- `FilePomodoroPersistence` stores a separate version-1 `pomodoro.json` under
  Electron's user-data directory. It does not store Pomodoro state inside
  `settings.json`.
- The exact document is `{ "version": 1, "state": PomodoroState }`.
  Temporary-file writes use mode `0600` and rename over the authoritative
  file.
- A missing file starts idle and queues the default document. A valid running
  or paused document is restored and materialized. An expired running session
  completes during load. Invalid or failed loads fall back to idle and are
  logged without inventing a schema migration.
- State-changing persistence operations are serialized and persistence
  failures are logged without crashing the timer.

**IPC, DesktopBridge, and renderer integration**

- Electron exposes only `pomodoro:start` and
  `pomodoro:custom-panel-closed` as renderer-originated operations.
  Pause, resume, stop, and preset starts originate in the native context menu
  and remain runtime-owned.
- The existing backend events are exactly
  `pomodoro:custom-duration-requested`, `pomodoro:state-changed`, and
  `pomodoro:completed`, all targeted to the Companion.
- Electron main buffers one pending completion and resynchronizes the current
  state plus a visible custom-duration request after Companion load. The
  preload separately buffers early state, completion, and custom-duration
  delivery until React listeners register.
- `usePomodoroState.ts`, `PomodoroDurationPanel.tsx`,
  `PomodoroWidget.tsx`, and `App.tsx` own presentation only. The widget shows
  Focus or Paused, the custom panel validates 1–720 minutes, and completion
  retains the existing celebration, personality message, and
  `NotificationSoundService` playback.
- The current broad `CompanionBridge` contains the Electron Pomodoro methods.
  Phase 8 will extract an exact narrow `PomodoroBridge`, preserve the Electron
  adapter, and activate the Tauri adapter without adding runtime detection to
  React.

**Native menu and permissions**

- `src/main/menus.ts` builds the Pomodoro submenu on every context-menu open.
  It contains 25/50/90 minute radio items, Custom…, Pause, Resume, and Stop.
  Checked/enabled state comes from the current authoritative timer snapshot.
- The existing Tauri context menu deliberately deferred this submenu. Phase 8
  must restore it using Rust-owned callbacks and current native state.
- Electron authorizes only the Companion renderer's two Pomodoro requests.
  Tauri currently reserves the three event routes but has no Pomodoro
  commands, grants, or notification plugin. Phase 8 will add only exact
  Companion command/listen authority and no OS notification permission.

**Contract clarification**

- The user-supplied Phase 8 contract mentioned short/long breaks, reset,
  skip, session transitions, and moving Pomodoro into the native settings
  store. None exists in Electron, while `migration_codex.md` explicitly
  requires preservation of the separate `pomodoro.json` format.
- The migration-wide parity rule therefore removes those feature-expansion
  requirements. Phase 8 migrates only the existing focus timer,
  preset/custom duration behavior, pause/resume/stop behavior, separate
  persistence, three events, existing React completion UI/sound, and existing
  context-menu integration.

**Files changed**

- `docs/migrating/migration_tasks.md` — aligned the Phase 8 execution contract
  with the authoritative Electron implementation.
- `docs/migrating/progress.md` — recorded the complete Phase 8 discovery.

**Validation performed**

- Reviewed the Electron manager, persistence, main-process startup/shutdown,
  IPC authorization, preload buffering, shared DTOs/events, renderer hook,
  custom-duration panel, timer widget, completion/sound flow, native context
  menu, tests, DesktopBridge adapters, Tauri event registry, command
  authorization, persistence infrastructure, menus, capabilities, and
  lifecycle.
- Documentation validation will run before the discovery clarification is
  committed. No production source or dependency was changed.

**Manual verification**

- Not applicable to discovery. No production implementation was attempted.

**Blockers**

- None.

**Next task**

- Task 8.2 — Create Native Pomodoro Engine.

### Task 8.2 — Native Pomodoro Engine

**Status:** Engine core complete. The concrete `pomodoro.json` repository and
startup registration remain Task 8.3.

**Implementation summary**

- Added the exact Electron `PomodoroState` and version-1 persisted document
  schema in Rust, including strict duration, state-combination, timestamp, and
  unknown-field validation.
- Ported the existing focus-timer state machine: start/restart with a selected
  duration, pause after wall-clock materialization, resume from frozen
  remaining time, stop to the selected idle duration, elapsed-time
  materialization, and one-shot completion.
- Added a singleton native runtime with one named scheduler worker. It wakes
  on the next one-second boundary, emits immutable state snapshots, sleeps
  while idle or paused, and reconciles delayed wakes from wall-clock time.
- Preserved Electron's serialized asynchronous save ordering through a native
  queue. Runtime mutations remain active if persistence fails, and shutdown
  drains queued writes before joining the worker.
- Kept persistence and event delivery behind narrow native interfaces so the
  following milestones can connect the separate file store and exact Tauri
  event recovery without changing timer semantics.
- Added no renderer timer, break state, reset, skip, native notification, new
  plugin, or Electron change.

**Files changed**

- `src-tauri/src/domain/pomodoro/mod.rs` — native schema, state machine,
  scheduler runtime, persistence/event interfaces, and focused tests.
- `src-tauri/src/domain/mod.rs` — registered the Pomodoro domain module.
- `docs/migrating/progress.md` — recorded Task 8.2.

**Validation performed**

- `cargo fmt --check`: passed.
- `cargo test`: passed (72 tests, including seven native Pomodoro tests).
- `cargo build`: passed. The unregistered domain emits expected dead-code
  warnings until Task 8.3 connects it to application startup.
- `npx tauri permission list`: passed; the native-only engine adds no
  renderer permission.
- `npm run typecheck`: passed.
- `npm test`: passed (135 tests).
- `npm run build`: passed, including Electron main and both renderer entries.
- Electron production-output smoke launch: passed through the unchanged
  Electron `PomodoroManager`, preload, and menu implementation. Existing
  permission-denial diagnostics remained expected and unchanged.
- `npx tauri dev --no-watch`: passed. The existing Tauri shell launched and
  remained alive; only the known development custom-protocol fallback warning
  appeared.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  current macOS application bundle.

**Manual verification**

- No UI path is connected in this engine-only milestone. State transitions,
  delayed time materialization, restored running state, expired restoration,
  one-shot completion, singleton startup, and queued-save shutdown are covered
  by deterministic native tests.

**Blockers**

- None.

**Next task**

- Task 8.3 — implement the separate native `pomodoro.json` repository,
  register/start the runtime before the Companion, and stop it during native
  shutdown.

### Task 8.3 — Pomodoro Persistence and Startup Restoration

**Status:** Complete.

**Implementation summary**

- Added a dedicated native `PomodoroStore` for the exact Electron
  `pomodoro.json` version-1 document. Pomodoro remains separate from
  `settings.json`.
- The store performs same-directory temporary writes, owner-only permissions,
  file sync, atomic replacement, and directory sync. It distinguishes missing,
  invalid, and failed reads so the runtime can preserve Electron's different
  fallback behavior.
- A missing file materializes the idle document during runtime load. An
  invalid file remains untouched for diagnosis while the runtime uses idle
  state. A valid running or paused document restores exactly, and an expired
  running document completes once during startup.
- Added a one-time, source-preserving import from Electron's platform data
  directory when the native file does not yet exist. A native file always
  wins, making the handoff idempotent.
- Tauri now constructs and starts one Pomodoro runtime before menus and the
  Companion renderer. The runtime is managed by Rust application state and
  its scheduler is stopped and joined during native exit.
- Added a pending native event queue that retains the latest state and one
  completion flag before the Companion event listener exists. Task 8.5 will
  activate and flush it without losing an expired restored session.
- Electron persistence, files, manager, preload, renderer, and menus remain
  unchanged.

**Files changed**

- `src-tauri/src/infrastructure/pomodoro.rs` — exact file repository,
  one-time legacy import, atomic persistence, and focused tests.
- `src-tauri/src/infrastructure/mod.rs` — registered the Pomodoro
  infrastructure module.
- `src-tauri/src/domain/pomodoro/mod.rs` — pending state/completion queue.
- `src-tauri/src/app_state.rs` — native path resolution, import, singleton
  runtime startup, and state management.
- `src-tauri/src/desktop/lifecycle.rs` — clean Pomodoro scheduler shutdown.
- `docs/migrating/progress.md` — recorded Task 8.3.

**Validation performed**

- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (78 tests, including exact schema round trip, missing
  and invalid behavior, owner-only permissions, idempotent legacy import, and
  pending event retention).
- `cargo build`: passed. Only expected warnings for command/event methods
  deferred to the immediately following milestones remain.
- `npx tauri permission list`: passed; native storage and startup require no
  renderer permission.
- `npm run typecheck`: passed.
- `npm test`: passed (135 tests).
- `npm run build`: passed.
- Electron production-output smoke launch: passed through the unchanged
  Electron timer and persistence implementation.
- `npx tauri dev --no-watch`: passed. The first native run imported the valid
  Electron `pomodoro.json`, started the singleton scheduler before the
  renderer, and remained alive without persistence or runtime errors.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  current macOS application bundle.

**Manual verification**

- Confirmed the one-time development handoff selected the existing Electron
  file and logged only its path, never its contents.
- End-to-end start/pause/resume/stop and restart UI verification remains
  deferred until the exact bridge, event recovery, and context-menu actions
  are connected.

**Blockers**

- None.

**Next task**

- Task 8.4 — expose only the existing Companion Pomodoro bridge operations
  and typed native commands. Do not grant the final renderer permissions
  before Task 8.8.

### Task 8.4 — DesktopBridge Commands

**Status:** Complete. Event transport activation and final capability grants
remain Tasks 8.5 and 8.8 respectively.

**Implementation summary**

- Extracted the existing Pomodoro renderer contract into a narrow
  `PomodoroBridge` without changing its Electron method names, payloads, or
  synchronous state-cache behavior.
- Added `getPomodoroBridge()` to the role-scoped Companion DesktopBridge.
  Electron continues to adapt the existing preload object, while the Tauri
  adapter deliberately keeps the bridge unavailable until Task 8.5 can
  install all three event listeners and recover buffered startup events
  without loss.
- Routed the existing custom-duration panel, timer-state hook, and completion
  subscriptions through the narrow runtime-neutral bridge. React still has no
  Electron/Tauri detection and imports no Tauri API.
- Added the only two renderer-originated native operations that exist in
  Electron: `start_pomodoro` and `custom_pomodoro_panel_closed`. Start
  delegates to the singleton native runtime; panel close clears only the
  native pending custom-panel request.
- Added strict Companion-only command authorization and stable serialized
  errors. Pause, resume, stop, presets, and Custom… remain native menu
  operations and were not exposed to the renderer.
- Generated the exact allow/deny permission definitions, but intentionally
  did not grant them to either renderer before the complete event bridge is
  ready. No wildcard, Preferences, notification, or plugin authority was
  added.

**Files changed**

- `src/shared/types.ts` — exact narrow `PomodoroBridge`.
- `src/desktop/contracts.ts` and `src/desktop/DesktopBridge.ts` — role-scoped
  Companion bridge accessor.
- `src/desktop/electronBridge.ts` — unchanged Electron preload behavior
  exposed through the narrow adapter.
- `src/desktop/tauriBridge.ts` — capability gate for the pending complete
  Tauri event adapter.
- `src/desktop/tauriCommands.ts` — typed native Pomodoro commands.
- `src/renderer/hooks/usePomodoroState.ts` and `src/renderer/App.tsx` —
  runtime-neutral narrow bridge usage.
- `src-tauri/src/commands/pomodoro.rs` and
  `src-tauri/src/commands/mod.rs` — native command handlers and registry.
- `src-tauri/src/authorization.rs` — exact Companion-only command roles.
- `src-tauri/src/domain/pomodoro/mod.rs` — native custom-panel close state.
- `src-tauri/permissions/autogenerated/start_pomodoro.toml` and
  `src-tauri/permissions/autogenerated/custom_pomodoro_panel_closed.toml` —
  generated narrow allow/deny definitions.
- `tests/desktop-bridge-boundary.test.cjs` and
  `tests/tauri-ipc-authorization.test.cjs` — renderer boundary and deferred
  least-privilege coverage.
- `docs/migrating/progress.md` — recorded Task 8.4.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (136 tests).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (79 tests).
- `cargo build`: passed.
- `npx tauri permission list`: passed and listed the two generated command
  permission pairs; neither is granted in a capability yet.
- Electron production-output smoke launch: passed through the unchanged
  preload and `PomodoroManager`.
- `npx tauri dev --no-watch`: passed; the shell remained alive with no command
  or permission warning. Only the known development custom-protocol fallback
  warning appeared.
- `npm run tauri:build -- --debug --bundles app`: passed.

**Manual verification**

- No new Tauri UI path is intentionally enabled in this command-only
  milestone. Electron's existing custom-duration flow remains operational.
  End-to-end native event, widget, and menu verification follows after Tasks
  8.5–8.8.

**Blockers**

- None.

**Next task**

- Task 8.5 — connect the three existing Companion Pomodoro events with
  startup/reload recovery, then expose the complete Tauri Pomodoro bridge.

### Task 8.5 — Runtime Events

**Status:** Complete.

**Implementation summary**

- Connected the native runtime exclusively to the existing Companion event
  routes: `pomodoro:state-changed`, `pomodoro:completed`, and
  `pomodoro:custom-duration-requested`. No new Pomodoro lifecycle event was
  introduced.
- Replaced the startup-only placeholder queue with a recoverable native event
  delivery state. It retains the latest state snapshot, one pending completion,
  and the current custom-panel visibility request while the Companion is
  loading or reloading.
- Event activation first replays the current state, then a pending completion,
  then a visible custom-panel request. State and custom-panel visibility are
  resynchronized after every renderer reload, while completion is consumed
  exactly once only after successful native delivery.
- Delivery failures leave the corresponding event pending and inactive until
  the renderer activates again. Concurrent scheduler changes are folded into
  the latest state without an ordering race.
- The Companion page-load hook deactivates Pomodoro delivery at navigation
  start, matching the existing reminder recovery boundary and Electron's
  `isLoadingMainFrame()` behavior.
- Added a dedicated Tauri Pomodoro adapter that installs all three native
  listeners before calling the activation command. Its preload-equivalent
  cache preserves early state, completion, and custom-duration events until
  the existing React subscribers register.
- Enabled only `activate_pomodoro_events` for the Companion because it is
  required to make the event milestone functional. The renderer-originated
  start and panel-close commands remain ungranted until the final Phase 8
  permission milestone.

**Files changed**

- `src-tauri/src/domain/pomodoro/mod.rs` — recoverable event delivery state,
  activation/deactivation, custom-panel visibility, and focused tests.
- `src-tauri/src/app_state.rs` — exact Tauri event emitters and page-load
  deactivation.
- `src-tauri/src/commands/pomodoro.rs`,
  `src-tauri/src/commands/mod.rs`, and
  `src-tauri/src/authorization.rs` — Companion-only event activation.
- `src-tauri/permissions/autogenerated/activate_pomodoro_events.toml` —
  generated narrow allow/deny definition.
- `src-tauri/capabilities/companion.json` — exact activation grant.
- `src/desktop/tauriPomodoroBridge.ts` — native listener coordination,
  recovery cache, and exact Pomodoro bridge adapter.
- `src/desktop/tauriBridge.ts` and `src/desktop/tauriCommands.ts` — activated
  Pomodoro adapter and typed activation dispatch.
- `tests/desktop-bridge-boundary.test.cjs` and
  `tests/tauri-ipc-authorization.test.cjs` — three-listener activation and
  least-privilege coverage.
- `docs/migrating/progress.md` — recorded Task 8.5.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (137 tests).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (81 tests), including startup/reload recovery and
  failed-delivery retry coverage.
- `cargo build`: passed.
- `npx tauri permission list`: passed and confirmed the exact event activation
  permission definition.
- Electron production-output smoke launch: passed through the unchanged
  Electron preload buffering and event contract.
- `npx tauri dev --no-watch`: passed. The renderer installed all three
  listeners and activated native delivery without command, event, or
  permission errors. Only the known development custom-protocol fallback
  warning appeared.
- Packaged debug Tauri build will be rerun with the integrated context-menu
  milestone before manual Phase 8 verification.

**Manual verification**

- The Tauri Companion received the restored state snapshot after its three
  native listeners registered, with no renderer error. Interactive
  preset/custom/pause/resume/stop/completion verification remains Task 8.6
  onward.

**Blockers**

- None.

**Next task**

- Task 8.6 — rebuild the Electron-parity Pomodoro context submenu from the
  current native state and connect its Rust-owned actions.

### Task 8.6 — Companion Integration

**Status:** Complete. Final interactive verification follows after the two
renderer-originated command grants in Task 8.8.

**Implementation summary**

- Restored Pomodoro as the first Companion context submenu, followed by the
  same separator and previously migrated Personal Assistant/static sections.
- The submenu is rebuilt from a freshly materialized native runtime snapshot
  on every open. It contains 25 min, 50 min, 90 min, Custom…, Pause, Resume,
  and Stop in the exact Electron order with the same separators.
- Preset/custom checked state follows `selectedDurationMinutes`. Pause is
  enabled only for an active unpaused session, Resume only for an active
  paused session, and Stop for any active session.
- Tauri/muda exposes native checked menu items rather than a separate radio
  item primitive. The selected duration remains a mutually exclusive native
  checked state and is recalculated on every open; no renderer state or
  callback is involved.
- Added closed Rust action dispatch for all seven Pomodoro menu IDs. Presets
  start or replace the current session, Pause/Resume/Stop mutate the singleton
  runtime, and Custom… shows/focuses the Companion before emitting the
  recoverable custom-duration request.
- Reused the existing React `PomodoroDurationPanel`, `PomodoroWidget`,
  `usePomodoroState`, and panel lifecycle without UI, copy, timer, or runtime
  detection changes. Electron menus and callbacks remain untouched.

**Files changed**

- `src-tauri/src/desktop/menus.rs` — dynamic Pomodoro submenu, checked/enabled
  projection, native action dispatch, and regression tests.
- `src-tauri/src/domain/pomodoro/mod.rs` — test-only inspection methods no
  longer produce production dead-code warnings.
- `docs/migrating/progress.md` — recorded Task 8.6.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (137 tests).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (82 tests), including exact action IDs and idle,
  running-custom, and paused menu-state projections.
- `cargo build`: passed without warnings.
- `npx tauri permission list`: passed; native menu callbacks add no renderer
  authority.
- Electron production-output smoke launch: passed with the unchanged Electron
  menu implementation.
- `npx tauri dev --no-watch`: passed with no command, menu, event, or
  permission errors. Only the known development custom-protocol fallback
  warning appeared.
- `npm run tauri:build -- --debug --bundles app`: passed.

**Manual verification**

- The packaged debug application was rebuilt with the dynamic submenu.
  Full interactive preset/custom/pause/resume/stop and state verification is
  intentionally performed after Task 8.8 enables the two existing
  renderer-originated commands, so one coherent final parity run exercises
  the complete flow.

**Blockers**

- None.

**Next task**

- Task 8.7 — lock the existing React completion celebration, personality
  message, and notification-sound path without adding native notifications.

### Task 8.7 — Timer Completion

**Status:** Complete.

**Implementation summary**

- Preserved the existing renderer-owned completion behavior without adding a
  native notification path or changing application behavior.
- Confirmed the existing `pomodoro:completed` handler plays the configured
  Pomodoro notification sound, starts the existing celebration, requests the
  existing personality completion message, and records pending completion
  state through the same React path used by Electron.
- Added source-level regression coverage that locks this exact parity contract
  and rejects a Tauri notification plugin or notification capability in the
  Phase 8 implementation.

**Files changed**

- `tests/pomodoro-completion-parity.test.cjs` — completion behavior and
  no-native-notification parity coverage.
- `docs/migrating/progress.md` — recorded Task 8.7.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (139 tests).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (82 tests).
- `cargo build`: passed.
- `npx tauri permission list`: passed; no notification plugin or capability is
  present.
- Electron production-output smoke launch: passed with the unchanged renderer
  completion handler.
- `npx tauri dev --no-watch`: passed without Pomodoro command, event, or
  permission errors. Only the known development custom-protocol fallback
  warning appeared.
- `npm run tauri:build -- --debug --bundles app`: passed.

**Manual verification**

- This milestone deliberately retains the existing React behavior rather than
  introducing an alternative native completion implementation. The full
  interactive one-minute completion check is part of the final Phase 8 parity
  run after the scoped command permissions are granted.

**Blockers**

- None.

**Next task**

- Task 8.8 — grant the two existing renderer-originated Pomodoro commands to
  the Companion and complete Phase 8 parity verification.

### Task 8.8 — Permissions

**Status:** Complete. Phase exit remains gated on the final interactive parity
run.

**Implementation summary**

- Granted the Companion webview only the two previously generated,
  renderer-originated Pomodoro command permissions:
  `allow-start-pomodoro` and
  `allow-custom-pomodoro-panel-closed`.
- Kept event activation and low-frequency event listen/unlisten authority
  unchanged.
- Granted no Pomodoro command to Preferences and added no wildcard, menu,
  notification, filesystem, process, shell, or other plugin permission.
- Updated authorization regression coverage so every registered command
  permission is complete, role-exact, and backed by one narrow generated
  allow/deny pair.

**Files changed**

- `src-tauri/capabilities/companion.json` — exact Companion grants for the two
  renderer-originated Pomodoro commands.
- `tests/tauri-ipc-authorization.test.cjs` — completed permission inventory and
  role-isolation coverage.
- `docs/migrating/progress.md` — recorded Task 8.8.

**Validation performed**

- `npm run typecheck`: passed.
- `npm test`: passed (139 tests).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (82 tests).
- `cargo build`: passed.
- `npx tauri permission list`: passed and resolved all three Pomodoro command
  grants without wildcard or plugin authority.
- Electron production-output smoke launch: passed with the unchanged Electron
  preload and `PomodoroManager`.
- `npx tauri dev --no-watch`: passed without Pomodoro command, event, menu, or
  permission errors. Only the known development custom-protocol fallback
  warning appeared.
- `npm run tauri:build -- --debug --bundles app`: passed.

**Manual verification**

- The Tauri shell started successfully with the completed least-privilege
  capability. Full preset/custom/replacement/pause/resume/stop/restart and
  completion parity is the remaining Phase 8 exit gate.

**Blockers**

- None.

**Next task**

- Perform the complete Phase 8 interactive parity verification; do not begin
  Phase 9.

### Phase 8 — Final Manual Parity Verification

**Status:** Complete.

**Implementation summary**

- Completed the Electron-parity Pomodoro migration without adding breaks,
  cycles, Preferences settings, native notifications, renderer timers, or
  other new behavior.
- Rust now owns the singleton timer engine, wall-clock materialization,
  persistence, startup restoration, native context-menu actions, and exact
  event delivery.
- The renderer remains runtime agnostic behind DesktopBridge and retains the
  existing custom-duration panel, timer widget, completion personality
  message, celebration, and notification-sound service.
- Electron remains fully intact and continues to use its existing
  `PomodoroManager`, preload bridge, context menu, persistence, and renderer
  event contract.

**Phase 8 commits**

- `9188bb4 docs(tauri): align phase 8 with electron parity`
- `c0b2790 feat(tauri): migrate pomodoro engine`
- `b6f5118 feat(tauri): migrate pomodoro persistence`
- `77d2629 feat(tauri): migrate pomodoro commands`
- `8276212 feat(tauri): migrate pomodoro events`
- `5abfeb0 feat(tauri): migrate pomodoro companion`
- `f18587c feat(tauri): migrate pomodoro completion`
- `5b3ab87 feat(tauri): migrate pomodoro permissions`

**Final automated validation**

- `npm run typecheck`: passed.
- `npm test`: passed (139 tests).
- `npm run build`: passed, including Electron main and both renderer entries.
- `cargo fmt` / `cargo fmt --check`: passed.
- `cargo test`: passed (82 tests).
- `cargo build`: passed.
- `npx tauri permission list`: passed.
- Electron production-output smoke launch: passed.
- `npx tauri dev --no-watch`: passed. Only the previously documented
  development custom-protocol fallback warning appeared.
- `npm run tauri:build -- --debug --bundles app`: passed and produced the
  packaged debug macOS application used for manual verification.

**Manual verification**

- Opened the packaged Phase 8 application from
  `src-tauri/target/debug/bundle/macos/Ducky.app`.
- Verified the native Companion menu contains the exact Electron Pomodoro
  structure and action order.
- Verified 25, 50, and 90 minute presets each start and render the correct
  Companion timer value.
- Verified starting 50 minutes replaces an active 25-minute session and
  starting 90 minutes replaces that session.
- Verified Custom… opens the existing React duration panel and a one-minute
  custom session starts through DesktopBridge.
- Verified Pause changes the widget to `Paused`, freezes the exact remaining
  time, and disables Pause while enabling Resume.
- Verified Resume continues from the frozen remaining time and restores the
  running menu state.
- Verified Stop removes the timer widget and returns Pause, Resume, and Stop
  to their idle disabled state.
- Verified native Restart preserves an active custom session. The timer was
  visible at `00:29` before restart and restored at `00:14`, demonstrating
  startup restoration and elapsed wall-clock materialization rather than
  resetting the session.
- Verified the restored session completed after expiry, removed the timer
  widget, showed the existing `Focus complete. Take a short break.` React
  personality message, and did not emit a second completion during subsequent
  observation.
- The same manually exercised completion callback is covered by
  `tests/pomodoro-completion-parity.test.cjs`, which verifies the configured
  `pomodoro` sound playback request and celebration transition. The automation
  environment cannot independently hear host audio, so no unsupported claim
  of audible listening is made.
- macOS accessibility automation exposes the native menu items and enabled
  state but not the selected checkmark property. Exact mutually exclusive
  preset/custom checked-state projection is covered by the passing Rust menu
  regression test.
- No renderer error, permission prompt, or permission warning appeared during
  packaged interaction. The Tauri development smoke log likewise contained
  no Pomodoro command, event, menu, or permission error.
- Quit the packaged app after verification and restored the pre-existing
  native `pomodoro.json` byte-for-byte. The temporary recoverable QA backup was
  moved to Trash.

**Phase 8 exit criteria**

- Native singleton Pomodoro engine: passed.
- Rust-owned timer execution: passed.
- Electron-compatible separate persistence and restoration: passed.
- Preset, custom, replacement, pause, resume, and stop behavior: passed.
- Rust-owned native context-menu behavior: passed.
- Runtime-neutral DesktopBridge and exact event contract: passed.
- Existing renderer completion widget, personality message, celebration, and
  sound path: passed.
- Least-privilege Companion-only permissions: passed.
- Electron preservation and cross-runtime builds: passed.

**Blockers**

- None.

**Next task**

- Phase 9 — AI Integration Migration. Do not begin without a separate
  instruction.

### Task 9.1 — Audit Existing AI System

**Status:** Discovery complete. The source-of-truth conflict was resolved by
the Phase 9 architecture clarification; Task 9.2 has not started.

**Electron provider architecture**

- `src/ai/AIProvider.ts` defines the provider-independent request, final
  response, usage, model, connection-test, and nominal stream contracts.
- `src/ai/AIService.ts` owns one active provider, provider registration and
  selection, configuration fingerprints, provider disposal, ask/list/test
  operations, cancellation checks, and output limiting.
- `src/main/main.ts` registers OpenAI, Gemini, Grok, Ollama, and Custom
  OpenAI-compatible providers. It synchronizes the selected provider and model
  from persisted settings and loads credentials only in the privileged main
  process.
- OpenAI, Grok, and Custom share
  `src/ai/providers/OpenAICompatibleProvider.ts`. Gemini and Ollama use their
  dedicated provider implementations.
- `src/shared/settings.ts` is the authoritative provider list and settings
  contract. It currently contains `openai`, `gemini`, `grok`, `ollama`, and
  `custom`; Claude is not present in the Electron reference and is the one
  explicitly approved Phase 9 product expansion.

**Provider behavior and security**

- OpenAI uses the Responses API. Custom providers use Chat Completions with a
  constrained fallback to Responses. Both have bounded timeouts, retries,
  model discovery, response sizes, and sanitized error handling.
- Gemini uses `@google/genai` with whole-response `generateContent`, bounded
  model discovery, connection tests, cancellation signals, and token usage
  extraction.
- Grok uses the shared OpenAI-compatible provider with xAI-specific endpoint,
  model filtering, and sanitized HTTP connection diagnostics.
- Ollama uses a hardened loopback-only transport. It validates DNS resolution,
  pins the resolved address, verifies the response socket address, rejects
  redirects and compressed responses, and enforces strict request, response,
  and timeout limits.
- API keys never enter the normal renderer settings projection. Electron uses
  its protected credential service. Tauri Phase 6 already provides the
  server-side native credential store and exposes only status/save/delete
  operations to Preferences.

**Request lifecycle, cancellation, and limits**

- `src/main/AIRequestManager.ts` permits one active operation per renderer
  role, with Companion and Preferences isolated from each other.
- Chat is limited to 30 requests per minute. Connection tests and model
  discovery are each limited to 12 requests per minute.
- Provider operations receive `AbortSignal`. Electron cancels requests on
  provider changes, application quit, renderer reload/crash, window close, and
  navigation lifecycle changes.
- The renderer has no explicit user-initiated AI cancellation command. Closing
  a response while generation is pending invalidates the local request state;
  it does not send a cancel IPC request to the main process.

**Streaming findings**

- Electron does **not** stream AI responses. Every current provider's
  `streamMessage` implementation immediately throws the shared
  `unsupported_operation` error.
- `AIService` exposes no streaming operation. `ai:ask`, the Companion preload,
  `CompanionBridge.askAI`, `DesktopBridge`, and `App.tsx` exchange one final
  `AIAskResult`.
- The existing typewriter effect starts only after that complete response has
  arrived; it is presentation behavior, not provider streaming.
- `migration_codex.md` accurately documents this: provider streaming methods
  exist in the interface, but every current provider reports streaming
  unsupported and no streaming migration is required for parity.

**Renderer and DesktopBridge integration**

- `src/renderer/App.tsx` owns the conversation lifecycle, 16-message/24,000
  character context limit, Continue Chat, pin state, response dismissal,
  request identity, focus, and final-response presentation.
- The renderer calls only `CompanionBridge.askAI`; it does not import provider
  SDKs, HTTP clients, Electron, or Tauri APIs.
- Preferences uses the existing runtime-neutral bridge for configuration,
  model discovery, and connection tests. Tauri intentionally reports the AI
  and AI Model Explorer capabilities as unavailable until Phase 9 reaches
  parity.
- The current Tauri command registry, authorization table, and capabilities
  contain no AI provider commands. Phase 6 credential commands are scoped only
  to the Preferences window.

**AI actions and diagnostics**

- `src/ai/actions/` permits exactly `createReminder` and
  `setStickyMessage`. Provider output is parsed as untrusted input, validated,
  and executed through authoritative services in the Electron main process.
- Provider responses may carry input/output token usage. That metadata is
  preserved through the provider/action result but is not presented as a
  renderer diagnostic.
- Existing diagnostics consist of provider connection tests, sanitized
  errors, and additional safe HTTP detail for Grok failures. There is no
  provider-health subsystem and no latency measurement or latency-reporting
  contract in Electron.

**Native integration state**

- Tauri already stores imported AI settings in the Phase 5 settings document,
  but the current provider validator accepts only the five Electron provider
  identifiers.
- Tauri's Preferences projection redacts stored credentials, and Phase 6
  native credential operations remain server-side and least privilege.
- No native AI runtime, provider registry, provider commands, provider events,
  or AI capability permissions have been added.

**Resolved source-of-truth conflict**

- Task 9.11 and the Phase 9 request require incremental token streaming,
  chunk-order parity, streaming cancellation, and completion semantics while
  saying these must match Electron. Electron has no incremental response
  transport to match.
- Task 9.13 requires provider health and latency reporting while saying
  diagnostics must match Electron. Electron has connection tests and sanitized
  errors, but no health or latency-reporting feature.
- Task 9.14 lists concurrent requests, while Electron deliberately limits each
  renderer role to one active operation.
- Claude is an explicitly approved addition, but requiring its incremental
  stream to reach the renderer would necessarily add a new IPC, event,
  DesktopBridge, and React conversation contract that does not exist for any
  Electron provider.
- Implementing those former requirements would have redesigned and expanded
  the Electron contract, contrary to the migration-wide parity rule and the
  source priority specified for Phase 9.

**Files changed**

- `docs/migrating/progress.md` — recorded the complete Task 9.1 discovery and
  the source-of-truth blocker.

**Validation performed**

- Inspected the Electron provider implementations, provider registry,
  request manager, action parser/executor, settings and credential integration,
  IPC authorization, preloads, DesktopBridge adapters, renderer conversation
  flow, Tauri settings/credential boundary, command registry, authorization,
  and capabilities.
- Confirmed no production source file was modified during discovery.
- `git diff --check`: passed.

**Manual verification**

- Not applicable. Task 9.1 is a source audit and no runtime behavior changed.

**Blockers**

- None. The contract now requires whole-response asks, lifecycle cancellation,
  connection testing and sanitized errors, and one active operation per
  renderer role.
- Claude may consume Anthropic streaming internally in Rust, but Rust must
  aggregate it into the existing final-response contract before DesktopBridge.

**Next task**

- Task 9.2 — Create Native AI Runtime. Do not begin without a separate
  implementation instruction.

### Phase 9 — Architecture Clarification

**Status:** Complete. Documentation only; Phase 9 implementation has not
started.

**Implementation summary**

- Aligned the complete Phase 9 execution contract with the migration-wide
  Electron parity rule.
- Replaced renderer-side streaming requirements with the existing
  single-final-response contract.
- Preserved only Electron's lifecycle cancellation behavior. No explicit
  renderer cancellation command, IPC channel, event, or DesktopBridge API is
  authorized.
- Preserved connection tests, sanitized provider errors, existing safe
  provider-specific error detail, and existing usage metadata. Provider-health
  and latency reporting are explicitly outside the parity contract.
- Preserved one active request per renderer role and removed the conflicting
  same-role concurrency requirement.
- Retained Claude as the sole approved product expansion. Claude may consume
  Anthropic streaming internally, but Rust must aggregate the complete result
  before returning the same final response used by every provider.
- Preserved the current provider lifecycle, provider registry, persistence,
  action allowlist, renderer behavior, and DesktopBridge boundary.

**Files changed**

- `docs/migrating/migration_tasks.md` — corrected the Phase 9 discovery, scope,
  provider tasks, response transport, diagnostics, execution controls, manual
  verification, and exit criteria.
- `docs/migrating/migration_codex.md` — clarified that an upstream streaming
  API may terminate inside Rust only and does not authorize renderer streaming
  or new bridge contracts.
- `docs/migrating/progress.md` — recorded the resolved architecture contract
  and next-task boundary.

**Validation performed**

- `git diff --check`: passed.
- Confirmed the diff contains documentation files only.
- Tests and builds are not required because no production source or runtime
  behavior changed.

**Manual verification**

- Not applicable. This clarification changes documentation only.

**Blockers**

- None.

**Next task**

- Task 9.2 — Create Native AI Runtime. Do not begin without a separate
  implementation instruction.
