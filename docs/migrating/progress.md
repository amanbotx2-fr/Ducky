# Ducky Tauri v2 Migration Progress

**Last updated:** 27 July 2026

## Current Status

- Active phase: Phase 1 — Create Tauri Workspace
- Last completed task: Task 1.2 — Configure two windows
- Next task: Task 1.3 — Capabilities
- Blockers: None for Phase 1

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
