# Ducky Electron to Tauri v2 Architecture Discovery and Migration Plan

**Review date:** 27 July 2026  
**Scope:** Architecture discovery and migration analysis only  
**Current desktop version:** 1.1.0  
**Target:** Tauri v2 on macOS, Windows, and Linux

## Executive Summary

Ducky is a two-window Electron desktop application:

- a transparent, frameless, always-on-top companion window; and
- a conventional Preferences window.

The renderer is React 19 and Vite 8. Most of its UI, CSS, sprite animation, eye rendering, speech bubbles, forms, and notification audio can remain. The privileged Electron main process cannot remain in a normal Tauri application: its lifecycle, persistence, credential protection, scheduling, AI networking, native window operations, menus, tray, and updater must move to Rust or Tauri plugins.

The migration is feasible, but it is not a shell replacement. The overall complexity is **high** because three areas are not drop-in compatible:

1. **Credential continuity:** Electron `safeStorage` ciphertext cannot be assumed to be decryptable by a Tauri credential backend.
2. **AI providers:** the current provider stack relies on Node libraries and a security-hardened Node HTTP implementation, especially for Ollama.
3. **Updates and releases:** Electron Builder update manifests, blockmaps, and packages are incompatible with the signed artifact format required by the Tauri updater.

The recommended approach is a parallel, strangler-style migration: add `src-tauri/` and a renderer-side desktop bridge while the Electron application remains releasable. Port and prove one domain at a time, perform cross-platform parity testing, release a final Electron transition build if credentials or installer handoff require it, then remove Electron.

### Migration outcome by area

| Area | Outcome |
| --- | --- |
| React renderers, CSS, components, hooks | Keep with a thin bridge adaptation |
| Sprite animation, behavior timers, eye visuals, drag gesture recognition | Keep; replace native window/cursor calls |
| Shared TypeScript parsing and display helpers | Keep where renderer-only; mirror authoritative validation in Rust |
| Electron main lifecycle and native integrations | Rewrite in Rust/Tauri |
| Settings, reminders, Pomodoro, daily planner | Port to Rust while preserving on-disk formats |
| AI service and providers | Rewrite in Rust; do not move secrets into the WebView |
| Preloads and `contextBridge` | Delete after the Tauri bridge is complete |
| Electron Builder and `electron-updater` | Replace with Tauri bundling/updater and a rewritten release workflow |
| Website | Remains a separate Next.js app; update release resolution and Electron copy only at cutover |

### Estimated engineering effort

For one engineer already familiar with the codebase, a responsible estimate is **10–16 engineer-weeks**, excluding delays for Apple/Windows signing accounts and external release review. The broad range reflects platform-specific transparent-window, installer-upgrade, secure-credential, and updater testing. A proof of concept can be much faster, but it is not equivalent to production parity.

## Review Method and Source of Truth

### Parity Rule

When migration documents conflict with the existing Electron implementation,
the existing Electron implementation is the authoritative behavior unless an
explicit redesign is documented and approved before implementation. This
migration preserves implemented product behavior; it does not use migration
tasks to introduce new features.

For AI providers, an implementation may consume a provider's streaming API
inside the privileged Rust runtime only when useful. Rust must aggregate that
stream into the single final response used by Electron before crossing
DesktopBridge. Internal provider transport does not authorize incremental
renderer streaming, streaming IPC/events, or a new DesktopBridge contract.

For updates, Electron's renderer-visible contract is status, manual checking,
the automatic-startup-check preference, and targeted status events. Internal
Electron download code that has no IPC, DesktopBridge, renderer, menu, or
notification caller does not authorize new user-facing download, install,
restart, menu, or notification behavior. The one-time manual Electron → Tauri
migration dialog is the only approved updater expansion.

The review covered all tracked top-level areas and all desktop runtime source:

- `.github/`, `assets/`, `character/`, `docs/`, `scripts/`, `src/`, `supabase/`, `tests/`, and `website/`;
- root manifests, TypeScript configurations, Vite configuration, Electron Builder configuration, and release workflow;
- every import from `electron` and `electron-updater`;
- every channel in `src/shared/events.ts`;
- both preloads and both window constructors;
- all main-process services, renderer components/hooks/services, AI providers, shared models, tests, and release verification.

Generated dependency/build directories such as `node_modules/`, `dist/`, `release/`, and `website/.next/` were treated as outputs rather than source. Package-lock contents and the installed dependency graph were inspected for Electron-specific transitive packages.

The implementation is the source of truth. Several documents describe an earlier or aspirational architecture. In particular, `docs/TECHNICAL.md` mentions PixiJS, Zustand, a physics engine, global activity adapters, Electron Store, settings import/export, and animation states that do not exist in the current application. Feature specifications under `docs/features/` also contain unchecked future work. Those planned features are not “must migrate” items.

## Repository and Folder Architecture

| Path | Current responsibility | Migration treatment |
| --- | --- | --- |
| `.github/workflows/release.yml` | Tag validation, tests, three-platform Electron builds, release publication | Rewrite for Rust/Tauri artifacts and signatures |
| `assets/icons/` | Application and tray icons | Reuse; generate any Tauri-required icon sizes |
| `assets/sounds/` | Four built-in WAV notification sounds | Reuse unchanged |
| `character/` | Official mascot and idle/blink/look-left frames | Reuse unchanged |
| `docs/` | Product, implementation, feature, security, release, and analytics documents | Update relevant release/architecture docs during migration |
| `scripts/generate-notification-sounds.mjs` | Build-time WAV generator | Keep as a Node development script |
| `scripts/verify-release-artifacts.mjs` | Verifies Electron artifacts and update metadata | Rewrite for Tauri bundles, signatures, and updater JSON |
| `src/ai/` | Node-side provider abstraction, providers, request/action processing | Port privileged behavior to Rust |
| `src/engine/` | Browser-side animation, eye, drag, and timer utilities | Mostly keep |
| `src/main/` | Electron lifecycle, native integrations, persistence, scheduling, IPC | Replace with `src-tauri/` Rust modules |
| `src/personality/` | Pure TypeScript message selection | Keep, except authoritative action responses may move to Rust |
| `src/renderer/` | React companion and Preferences renderers | Keep and adapt bridge calls |
| `src/shared/` | DTOs, parsers, reducers, settings/reminder/update contracts | Keep renderer-facing contracts; create matching Rust DTOs |
| `supabase/` | Website download analytics database migrations | No desktop runtime relationship |
| `tests/` | Node tests for desktop domain and provider behavior | Keep during transition; add Rust and Tauri integration tests |
| `website/` | Independent native Next.js/Vercel marketing and download application | Keep separate; it imports no desktop source |
| `electron-builder.yml` | Electron bundle/update configuration | Delete only after Tauri release parity |
| `vite.config.ts` | Two renderer entry points and renderer bundle | Keep, then point Tauri at its dev server/output |
| `tsconfig.main.json` | Compiles Electron main, AI, personality, shared code | Retire after Rust ports are complete |
| `tsconfig.renderer.json` | Type-checks renderer, engine, personality, shared code | Keep |

There is no current `apps/`, `electron/`, Cargo workspace, Tauri configuration, npm workspace, pnpm workspace, or Turborepo. Electron code lives in `src/main/`; the root package and `website/` are separate npm applications.

## Current Runtime Architecture

```text
Electron application
├── Main process (Node.js)
│   ├── application/window lifecycle
│   ├── settings + encrypted credential persistence
│   ├── reminders + Pomodoro + daily planner
│   ├── AI providers and assistant action execution
│   ├── tray, menus, cursor sampling, auto-launch
│   └── electron-updater
├── Companion preload
│   └── window.psyduck (fixed, typed IPC facade)
├── Companion renderer
│   ├── React UI
│   ├── sprite/eye/drag engines
│   └── audio + short-lived hydration timer
├── Preferences preload
│   └── window.psyduckPreferences (fixed, typed IPC facade)
└── Preferences renderer
    └── settings, AI model explorer, sounds, update check
```

### Startup sequence

`src/main/main.ts` is the executable entry point. Its effective sequence is:

1. Initialize application name/Windows application ID and install the macOS application menu.
2. Wait for `app.whenReady()`.
3. Configure macOS branding/About panel and dock icon.
4. Construct `CredentialManager` around Electron `safeStorage`.
5. Load `settings.json` through `SettingsService` from Electron `userData`.
6. Construct `UpdateService`, `ReminderService`, `DailyPlannerService`, and assistant-action processing.
7. Start `ReminderScheduler` and register `powerMonitor.resume` reconciliation.
8. Construct `AIService` with OpenAI, Gemini, Grok, Ollama, and custom OpenAI-compatible providers.
9. Construct/load `PomodoroManager` from `pomodoro.json`.
10. Apply AI configuration and register all IPC handlers.
11. Apply always-on-top, startup, and updater settings.
12. Create the companion window and system tray.
13. Subscribe to settings, reminder, Pomodoro, and updater state changes.
14. Check for updates.

On `app.activate`, Ducky shows or recreates the companion. It intentionally stays alive through the tray when windows close. On `before-quit`, it removes listeners, stops schedulers, cancels AI work, unregisters IPC, disposes services, and destroys the tray.

### Renderer lifecycle

- `src/renderer/index.html` → `main.tsx` → `App.tsx` mounts the companion under React Strict Mode.
- `src/renderer/preferences.html` → `preferences.tsx` → `PreferencesApp.tsx` mounts Preferences.
- Vite builds both pages into `dist/renderer`.
- The Electron preload is installed before each renderer and buffers important one-shot/state events until React subscribes.
- `CompanionWidgetStack` observes rendered height and asks the main process to resize upward while preserving the desktop bottom edge.
- Renderer navigation, redirects, new windows, and browser/device permissions are denied.

### State ownership

| State | Current owner |
| --- | --- |
| Validated settings and encrypted AI credential | Main process / `SettingsService` |
| Smart reminders | Main process / settings snapshot |
| Smart reminder wake scheduling | Main process / `ReminderScheduler` |
| Pomodoro persisted state and ticking | Main process / `PomodoroManager` |
| AI provider instance and requests | Main process / `AIService` |
| Current conversation, pin state, response dismissal | Companion renderer / `App.tsx` |
| Hydration countdown | Companion renderer / `WaterReminder` |
| Sprite animation queue and eye smoothing | Companion renderer |
| Preferences drafts/model explorer UI | Preferences renderer |

## Main Process Inventory

| File | Current responsibility | Tauri destination |
| --- | --- | --- |
| `src/main/main.ts` | Composition root, lifecycle, IPC, window orchestration, broadcasts | `src-tauri/src/lib.rs`, lifecycle, command, and event modules |
| `src/main/window.ts` | Companion window creation/placement/resizing | Rust `windows/companion.rs` |
| `src/main/preferencesWindow.ts` | Singleton Preferences window | Rust `windows/preferences.rs` |
| `src/main/preload.ts` | Companion bridge and early-event buffers | Renderer bridge adapter + Rust state snapshots; then delete |
| `src/main/preferencesPreload.ts` | Preferences bridge | Renderer bridge adapter; then delete |
| `src/main/ipcAuthorization.ts` | Role/channel/URL/subframe authorization | Tauri capabilities, command permissions, and label checks |
| `src/main/rendererSecurity.ts` | Dev URL validation, route loading, navigation/new-window denial | Tauri build URL/CSP/capabilities and webview navigation policy |
| `src/main/permissionPolicy.ts` | Deny all WebContents permissions | Least-privilege Tauri capabilities; do not grant unused plugins |
| `src/main/SettingsService.ts` | Validation, migrations, atomic settings persistence | Rust settings repository/service |
| `src/main/CredentialManager.ts` | `safeStorage` encryption/decryption and legacy migration | Rust credential vault adapter plus explicit legacy migration |
| `src/main/ReminderService.ts` | Reminder CRUD/recurrence mutation | Rust reminder service |
| `src/main/ReminderScheduler.ts` | Wake timer, overdue policy, resume reconciliation | Rust async scheduler |
| `src/main/ReminderEvents.ts` | Internal reminder event fan-out | Rust typed domain events/broadcast |
| `src/main/DailyPlannerService.ts` | Today's future reminder briefing | Rust query/service |
| `src/main/PomodoroManager.ts` | Timer state, persistence, relaunch materialization | Rust Pomodoro actor/service |
| `src/main/AIRequestManager.ts` | Abort, one-request-per-role, rate limiting | Rust cancellation tokens/semaphores/rate limiter |
| `src/main/AIConnectionDiagnostics.ts` | Safe provider diagnostics | Rust sanitized error/diagnostics model |
| `src/main/UpdateService.ts` | Electron update state machine | Rust updater runtime abstraction; production Tauri signing/feed integration follows in the release phase |
| `src/main/menus.ts` | macOS app menu, companion context menu, tray menu | Rust Tauri menu/tray builders |
| `src/main/tray.ts` | Tray icon, tooltip, context menu | Rust `TrayIconBuilder` |
| `src/main/appBranding.ts` | name, app ID, icons, About UI | Tauri config/Rust dialog/menu |

### Error handling and logging

The current application primarily uses structured prefixes with `console.warn`/`console.error`, validates renderer/provider inputs, sanitizes provider messages, and treats many notification/logging failures as non-fatal. It has no centralized persistent logging service, crash reporter, or telemetry. In Tauri, use Rust `tracing` or `log` with `tauri-plugin-log` if persisted logs are desired, preserve stable subsystem/event names, and continue excluding API keys, prompts, absolute user paths, and provider response bodies.

## Electron API Inventory and Tauri Mapping

Classification means:

- **DIRECT:** a core Tauri API or official plugin covers the behavior.
- **PARTIAL:** an equivalent exists but behavior/platform semantics require adaptation.
- **CUSTOM:** Ducky-specific Rust/platform work is required.
- **NOT NEEDED:** the Electron mechanism or API is unused or disappears in Tauri.

| Electron API/feature | Current implementation and purpose | Files | Class | Tauri v2 equivalent | Difficulty / problems / strategy |
| --- | --- | --- | --- | --- | --- |
| `app` | Ready/activate/quit/relaunch, version/name, resources/userData, login item, dock/About branding | `main.ts`, `appBranding.ts`, `rendererSecurity.ts` | PARTIAL | `Builder::setup`, `AppHandle`, `RunEvent`, path/package APIs, process/autostart plugin | **Medium.** Split composition from lifecycle. Prevent exit while tray is active and recreate/show the companion on activation. |
| `BrowserWindow` | Creates companion and Preferences windows | `window.ts`, `preferencesWindow.ts`, `main.ts`, authorization | PARTIAL | `WebviewWindowBuilder`/configured windows | **High.** Basic flags map directly; transparent always-on-top behavior must be proven in WKWebView, WebView2, and WebKitGTK. |
| `ipcMain` / `ipcRenderer` | 20 request/response invokes, 4 renderer events, 12 backend events | `main.ts`, both preloads | DIRECT | `#[tauri::command]` + `invoke`; Tauri events/channels | **Medium.** Preserve DTO validation and error shapes. Use channels for ordered cursor samples, events for low-frequency notifications, commands for state snapshots. |
| `contextBridge` | Exposes frozen role-specific bridge objects | both preloads | NOT NEEDED | `@tauri-apps/api` behind a local `DesktopBridge` adapter | **Low.** Do not expose raw Tauri APIs throughout components. Keep the current TypeScript bridge interface. |
| Preload scripts | Isolate channels and buffer early events | both preloads | NOT NEEDED | Rust state/queues plus renderer bridge subscription setup | **Medium.** Tauri events can be missed before listeners mount; provide initial snapshot commands and queue one-shot events in Rust. |
| `webContents.send` | Backend-to-renderer state and UI events | `main.ts` | DIRECT | `Emitter::emit_to` or ordered `Channel` | **Low–medium.** Use targeted window labels, not global broadcasts. |
| `webContents` navigation/window hooks | Denies navigation, redirects, popups; detects load/crash/navigation | `rendererSecurity.ts`, `main.ts` | PARTIAL | Tauri local content, CSP, capabilities, webview callbacks/events | **Medium.** Keep no remote content and no remote IPC. Add explicit navigation validation if links are ever introduced. |
| `session` permission handlers | Deny all web/device permissions | `permissionPolicy.ts` | PARTIAL | Capabilities, plugin permission scopes, CSP, no unused plugins | **Medium.** There is no one-for-one Electron session handler. Deny by omission and test camera/mic/geolocation are inaccessible. |
| `Tray` | Persistent tray icon/menu/tooltip | `tray.ts`, `main.ts` | DIRECT | `TrayIconBuilder` and native menu | **Low.** Rebuild menu whenever dynamic state changes if required. Current tray has no double-click behavior. |
| `Menu` | macOS app/edit menu, context menu, tray menu | `menus.ts`, `main.ts` | DIRECT | Tauri native Menu/Submenu/PredefinedMenuItem and popup menu | **Medium.** Preserve macOS edit roles and dynamic checkbox/radio/enabled state. |
| `nativeImage` | Loads/resizes app/tray/About icon | `appBranding.ts`, `tray.ts` | DIRECT | Tauri `Image`, `include_image!`, configured bundle icons | **Low.** Generate platform icon variants once and use a dedicated template tray asset on macOS if needed. |
| `dialog` | About information message | `appBranding.ts` | DIRECT | Rust dialog plugin/message dialog or custom About window | **Low.** Update “Built with Electron” copy at cutover. |
| `safeStorage` | Protects API keys, rejects Linux `basic_text`, migrates legacy plaintext | `CredentialManager.ts`, `main.ts`, `SettingsService.ts` | CUSTOM | Rust OS keyring adapter or Stronghold-backed vault | **High.** Stronghold is not a drop-in OS-vault equivalent and needs key management. Electron ciphertext needs a deliberate transition path or user re-entry. |
| `autoUpdater` / `electron-updater` | Checks GitHub releases, tracks status/progress, contains unexposed download logic | `UpdateService.ts`, `main.ts` | PARTIAL | Rust updater abstraction backed by `tauri-plugin-updater` after release configuration | **High.** Phase 10 preserves status/check/events/settings only. The current UI exposes check only; `downloadUpdate()` has no IPC/UI caller and does not authorize new controls. Phase 11 owns signatures, artifacts, endpoints, and production verification. |
| `powerMonitor` | Reconciles reminders/Pomodoro after system resume | `main.ts` | PARTIAL | `RunEvent::Resumed` plus elapsed-time reconciliation; custom platform validation | **High.** Tauri's event-loop resume is not guaranteed to be identical to Electron system sleep resume on every desktop. Keep deadline-based periodic reconciliation and add OS-specific suspend/resume handling only if tests prove it necessary. |
| `screen` | Cursor position, primary/matching display work areas | `main.ts`, `window.ts` | DIRECT | Tauri app cursor position and monitor/window APIs | **Medium.** Preserve physical/logical coordinate conversions, negative monitor coordinates, scale factors, and bottom-edge anchoring. |
| `nativeTheme` | Chooses initial Preferences background | `preferencesWindow.ts` | DIRECT | Tauri theme and `WindowEvent::ThemeChanged` / CSS media query | **Low.** Prefer CSS `prefers-color-scheme`, with a non-flashing native window background. |
| `setLoginItemSettings` | Start at login in packaged builds | `main.ts` | DIRECT | `tauri-plugin-autostart` | **Medium.** Confirm install/uninstall registration on all package formats. |
| `setAlwaysOnTop` | User-controlled companion z-order | `window.ts`, `main.ts` | DIRECT | window `set_always_on_top` | **Low.** Command must be companion-only. |
| `setVisibleOnAllWorkspaces` | Shows companion across macOS Spaces, excluding full-screen | `window.ts` | PARTIAL | window `set_visible_on_all_workspaces` | **Medium.** API is platform-specific and semantics must be tested on macOS Spaces/full-screen. |
| `skipTaskbar`, hidden buttons/menu | Keeps companion unobtrusive | both window files | PARTIAL | `skip_taskbar`, decorations/menu controls, platform activation policy if needed | **Medium.** Support differs by OS; preserve Preferences as a normal window. |
| `setBounds` / absolute `setPosition` | Dynamic widget height and drag positioning | `window.ts`, `main.ts`, `PsyDuck.tsx` | DIRECT | Tauri set size/position or `start_dragging` | **Medium.** Keep Ducky's click-versus-drag threshold and stable bottom edge. Test DPI/multi-monitor boundaries. |
| Transparent/frameless window | Desktop companion visual surface | `window.ts`, renderer CSS | PARTIAL | Tauri transparent undecorated WebviewWindow | **High.** System WebViews differ from Chromium; test transparency, hit regions, shadows, audio, and GPU behavior on each OS. |
| `shell` | Not used | none | NOT NEEDED | Do not install shell/opener permission for parity | None. |
| Electron `Notification` | Not used; notifications are renderer widgets | none | NOT NEEDED | Keep current React widgets; Tauri notification plugin is unnecessary | Avoid accidentally changing UX or adding an OS permission prompt. |
| `globalShortcut` | Not used | none | NOT NEEDED | Do not install global-shortcut plugin | None. |
| `desktopCapturer` | Not used | none | NOT NEEDED | No replacement | None. |
| `clipboard` | Not used | none | NOT NEEDED | Do not grant clipboard capability | None. |
| `protocol` | Not used | none | NOT NEEDED | Tauri's internal asset protocol is sufficient | Do not add custom schemes unless a real requirement appears. |
| `crashReporter` | Not used | none | NOT NEEDED | Optional future crash reporting, outside parity scope | None for migration. |
| `utilityProcess` | Not used | none | NOT NEEDED | No sidecar required in the recommended architecture | Avoid a Node sidecar except as a temporary, explicitly removable bridge. |
| `app.setAppUserModelId` | Windows identity/notifications/install integration | `appBranding.ts` | PARTIAL | bundle identifier/Windows installer metadata | **Medium.** Preserve `com.ducky.desktop` and verify upgrade/uninstall behavior. |

## Window and Desktop Behavior Inventory

### Companion

Defined in `src/main/window.ts`:

- initial logical content size `220 × 220`;
- bottom-right of the primary display work area with a 24-pixel margin;
- transparent, frameless, hidden title bar, no OS shadow;
- always-on-top according to settings;
- skipped from the taskbar;
- non-resizable, non-minimizable, non-maximizable, and non-fullscreenable;
- all macOS window buttons hidden;
- visible across macOS Spaces but not over full-screen applications;
- dynamically grows upward to fit widgets, clamped to the matching display work area;
- moved by a renderer pointer gesture using absolute screen coordinates.

This is the most platform-sensitive UI in the migration. Build it early as a cross-platform spike, but do not treat that spike as proof of production parity.

### Preferences

Defined in `src/main/preferencesWindow.ts`:

- singleton `640 × 650`, minimum `520 × 480`;
- conventional framed window;
- theme-appropriate native background to avoid a white flash;
- hidden application menu bar;
- separate route and restricted IPC role;
- reopening focuses the existing instance.

This maps cleanly to a named `preferences` Tauri WebviewWindow.

### Cursor and eye tracking

The Electron main process samples `screen.getCursorScreenPoint()` at approximately 30 Hz and sends changed positions. `PsyDuck.tsx` computes the eye origin using window screen position plus DOM bounds. `EyeTracker.ts` smooths and clamps pupil motion via `requestAnimationFrame` and CSS variables.

Recommended Tauri implementation:

1. Sample the native cursor in Rust.
2. Send ordered positions through one Tauri `Channel` scoped to `companion`.
3. Retain the existing TypeScript smoothing and CSS pupil rendering.
4. Normalize coordinates explicitly to one physical/logical coordinate system and test monitors left/above the primary display.

## IPC Inventory

### Current authorization model

`src/main/ipcAuthorization.ts` assigns every renderer-to-main channel to exactly one role. It rejects:

- unregistered channels;
- missing sender frames;
- destroyed senders;
- subframes;
- a BrowserWindow/WebContents mismatch; and
- any sender URL other than the exact pinned page URL.

Tauri capabilities are label-based and should be used, but custom application commands registered normally are available to all windows unless they are included in the Tauri application command manifest/permission system. The migration must therefore:

1. expose commands through a first-party internal plugin or an `AppManifest` command permission list;
2. create separate `companion` and `preferences` capabilities;
3. accept the invoking `WebviewWindow` in privileged commands and reject an unexpected label as defense in depth;
4. avoid wildcard windows, remote capability URLs, generic filesystem, HTTP, shell, and process permissions.

### Renderer to backend

All request payloads remain untrusted and must be parsed again in Rust.

| Channel | Role/direction | Purpose | Tauri mapping |
| --- | --- | --- | --- |
| `psyduck:get-cursor-position` | companion invoke | One cursor snapshot | `get_cursor_position` command |
| `psyduck:move-window` | companion event | Move companion during drag | scoped `move_companion_window` command |
| `psyduck:set-content-height` | companion event | Resize widget stack upward | scoped `set_companion_content_height` command |
| `psyduck:show-context-menu` | companion event | Open native context menu | scoped `show_companion_context_menu` command |
| `runtime-settings:get` | companion invoke | Initial runtime-safe settings | `get_runtime_settings` command |
| `runtime-settings:update-user-name` | companion invoke | Validate/persist name | `update_user_name` command |
| `runtime-settings:update-sticky-message` | companion invoke | Validate/persist sticky text | `update_sticky_message` command |
| `ai:ask` | companion invoke | Run AI request and approved actions | async `ask_ai` command with cancellation |
| `pomodoro:start` | companion invoke | Start selected duration | `start_pomodoro` command |
| `pomodoro:custom-panel-closed` | companion event | Clear pending custom-duration request | command or low-frequency event |
| `reminders:create` | companion invoke | Create validated reminder | `create_reminder` command |
| `reminders:update` | companion invoke | Update validated reminder | `update_reminder` command |
| `reminders:delete` | companion invoke | Delete reminder | `delete_reminder` command |
| `reminders:get` | companion invoke | Read one reminder | `get_reminder` command |
| `reminders:list` | companion invoke | Read sorted reminders | `list_reminders` command |
| `reminders:mark-completed` | companion invoke | Complete/advance recurrence | `mark_reminder_completed` command |
| `daily-planner:get` | companion invoke | Build today's briefing | `get_daily_planner` command |
| `preferences-settings:get` | preferences invoke | Preferences-safe snapshot | `get_preferences_settings` command |
| `preferences-settings:update` | preferences invoke | Validate/persist settings patch | `update_preferences_settings` command |
| `preferences-ai:configure` | preferences invoke | Validate provider/model/credential | `update_ai_configuration` command |
| `ai:list-models` | preferences invoke | Discover provider models | `list_ai_models` command |
| `ai:test-connection` | preferences invoke | Provider connectivity diagnostics | `test_ai_connection` command |
| `updates:status:get` | preferences invoke | Initial updater status | `get_update_status` command |
| `updates:check` | preferences invoke | Manual update check | `check_for_updates` command |

### Backend to renderer

| Channel | Destination | Purpose | Tauri mapping |
| --- | --- | --- | --- |
| `psyduck:cursor-position` | companion | Cursor samples | Ordered `Channel<ScreenPoint>` |
| `runtime-settings:changed` | companion and preferences | Secret-free accepted settings | targeted event; fetch snapshot after mount/recovery |
| `personal-assistant:user-name-requested` | companion | Open name panel from native menu | targeted event with pending flag |
| `personal-assistant:sticky-message-requested` | companion | Open sticky panel | targeted event with pending flag |
| `reminders:creation-panel-requested` | companion | Open creation panel | targeted event with pending flag |
| `reminders:manager-panel-requested` | companion | Open manager panel | targeted event with pending flag |
| `daily-planner:panel-requested` | companion | Open planner | targeted event with pending flag |
| `reminders:fired` | companion | Present due reminder | Rust queue plus targeted event |
| `updates:status-changed` | preferences | Update progress/status | targeted event |
| `pomodoro:custom-duration-requested` | companion | Open duration panel | targeted event with pending flag |
| `pomodoro:state-changed` | companion | Timer state/tick | targeted event plus initial snapshot |
| `pomodoro:completed` | companion | One-shot completion/sound | Rust pending flag/queue plus targeted event |

There are **36 unique IPC channel constants**. `runtime-settings:changed` has two destinations. The current preloads buffer panel requests, reminder events, Pomodoro completion/state, and custom-duration requests; equivalent recovery behavior is required after renderer reloads.

## Renderer Inventory

### Entry points and orchestration

| File | Responsibility | Migration |
| --- | --- | --- |
| `src/renderer/main.tsx` | Companion React bootstrap | Keep |
| `src/renderer/App.tsx` | Companion feature orchestration and local conversation lifecycle | Keep; replace `window.psyduck` with `DesktopBridge` |
| `src/renderer/preferences.tsx` | Preferences bootstrap | Keep |
| `src/renderer/PreferencesApp.tsx` | Settings/model/sound/update UI | Keep; replace `window.psyduckPreferences` adapter |
| `src/renderer/index.html` | Companion CSP/root | Keep; move/align CSP with Tauri config |
| `src/renderer/preferences.html` | Preferences CSP/root | Keep; move/align CSP with Tauri config |
| `src/renderer/styles/global.css` | Companion visuals/transitions | Keep; visually regress system WebViews |
| `src/renderer/styles/preferences.css` | Preferences visuals | Keep |

Both HTML pages currently allow only self-hosted scripts/images/styles plus the local Vite WebSocket in development. The Tauri `app.security.csp` should enforce the production policy centrally. Do not enable remote Tauri IPC access.

### Components

| Component files | Current responsibility | Migration |
| --- | --- | --- |
| `AIModelCard`, `AIModelGroup`, `AIModelSearch`, `AIModelExplorer` | Discover/search/favorite/select models; modal focus management | Keep |
| `ChatInputBubble` | Conversation input | Keep |
| `CompanionWidgetStack` | Orders widgets and reports content height | Keep; bridge height call |
| `FloatingCompanionPanel` | Shared panel positioning, focus, dismissal, transition | Keep |
| `DailyPlannerPanel` | Today's schedule UI | Keep |
| `PomodoroDurationPanel`, `PomodoroWidget` | Custom duration and active timer UI | Keep |
| `PsyDuck` | Sprite rendering, animation priority, eye layer, pointer drag/click/context actions | Keep; bridge cursor/window operations |
| `ReminderCreationPanel`, `ReminderManagerPanel`, `ReminderWidget` | Reminder creation/CRUD/fired presentation | Keep |
| `SpeechBubble`, `SpeechBubbleMarkdown` | Assistant copy and restricted Markdown rendering | Keep |
| `StickyMessagePanel`, `StickyMessageWidget` | Sticky-message edit/display | Keep |
| `UserNamePanel` | Name edit | Keep |

### Hooks and renderer services

| Files | Purpose | Migration |
| --- | --- | --- |
| `useModelExplorer.ts` | Model discovery/filter/favorites/recents | Keep |
| `usePomodoroState.ts` | Snapshot/subscription | Keep with Tauri bridge |
| `usePreferencesSettings.ts` | Preferences snapshot/updates | Keep with Tauri bridge |
| `useReminderNotifications.ts` | Reminder event queue/current item | Keep; validate Rust replay semantics |
| `useRuntimeSettings.ts` | Runtime snapshot/subscription | Keep with Tauri bridge |
| `useSpeechBubble.ts` | External-store speech content | Keep |
| `useTypewriterText.ts` | Text reveal | Keep |
| `useUpdateStatus.ts` | Updater snapshot/subscription/check | Keep against new updater DTO |
| `NotificationSoundService.ts` | Preload/play/stop WAV, independent volume, BroadcastChannel overlap prevention | Keep; QA WebView audio behavior |

### Animation, eye, and drag systems

The actual renderer is DOM/CSS based:

- `AnimationRegistry.ts` uses `import.meta.glob` to register frame files.
- `AnimationEngine.ts` advances frames using `requestAnimationFrame`.
- `BehaviorEngine.ts` schedules ambient look/blink behavior.
- `EyeTracker.ts` smooths pupil movement.
- `DragController.ts` interprets pointer gestures and window positions.
- `WaterReminder.ts` is a renderer-side timeout.
- `PsyDuck.tsx` coordinates the animation queue, eye visibility, dragging, and context menu.

Current authored animations are `idle` (6 frames), `blink` (5), and `look_left` (7); right-looking is achieved by horizontal flipping. `AssetLoader.ts`, `EventBus.ts`, and `StateMachine.ts` are minimal placeholders. There is no implemented physics engine, PixiJS renderer, Zustand store, typing animation, thinking animation, stretch reminder, or agent-done celebration.

The animation layer can remain unchanged. Test frame crispness, transparency, timer throttling, and `requestAnimationFrame` recovery in each system WebView.

### Conversation lifecycle

Conversation history, Continue Chat, pin state, response dismissal timers, and input focus live in `App.tsx`. Closing manually clears the renderer session; pinning disables automatic dismissal. Provider requests receive conversation messages in the validated request. This is renderer-local and can remain, but API keys and provider calls must stay in Rust.

## Settings and Persistence Architecture

### Settings

`src/shared/settings.ts` defines defaults, DTOs, strict parsing, URL policy, and secret-free projections for:

- user name and sticky message;
- smart reminders;
- always-on-top, launch-at-startup, and eye tracking;
- hydration enablement/interval;
- notification sound enablement, selection, and volume;
- automatic update preference;
- AI enabled/provider/model/endpoint/base URL/API-key-configured status;
- model favorites and recents.

`SettingsService.ts`:

- reads one versioned JSON document;
- serializes mutations through an operation queue;
- writes a temporary file with mode `0600` and atomically renames it;
- renames invalid data to a timestamped recovery file and restores defaults;
- stores reminder data and encrypted credential ciphertext in the same snapshot;
- migrates legacy plaintext credentials only after verified secure encryption.

### Pomodoro

`PomodoroManager.ts` stores a separate versioned `pomodoro.json`, also with atomic temporary writes and mode `0600`. It materializes elapsed time from timestamps after pauses, sleep, or relaunch, ticks once per second, and can complete a session after recovery.

### File interactions

Runtime filesystem access is limited to:

- `settings.json` and invalid-file recovery through `SettingsService.ts`;
- `pomodoro.json` through `PomodoroManager.ts`;
- packaged icon/mascot resource paths in native integration code.

Renderer assets and WAV files are bundled by Vite rather than read through a privileged runtime filesystem API. Build scripts write generated WAVs and inspect release artifacts. Tests create isolated temporary files.

### Tauri persistence strategy

Use Rust repositories under Tauri's application data directory and preserve the current JSON schema/file names initially. Do not expose generic filesystem plugin access to either WebView. Implement:

- atomic same-directory temporary write + rename;
- restrictive permissions where supported;
- the existing invalid-file recovery behavior;
- golden fixtures shared by TypeScript and Rust parsers;
- a one-time legacy Electron-data locator/import because Tauri's application data path must not be assumed to match Electron's path.

The official Store plugin is not necessary and would make exact legacy/recovery behavior harder to preserve.

### Credential migration risk

This is a release gate. The new vault must not silently discard a credential and must not weaken Linux behavior.

Recommended sequence:

1. Decide on a Rust OS keyring adapter or Stronghold design and threat model.
2. Build a migration fixture for each OS.
3. Determine whether the new executable can access and decrypt Electron `safeStorage` data. Assume **no** until demonstrated.
4. If not, ship a final Electron transition release that decrypts the credential and writes it through an explicitly designed handoff, or ask the user to re-enter the API key in Tauri.
5. Never write decrypted credentials into logs, command responses, renderer storage, environment files, or ordinary migration JSON.

## Reminder and Time-Based Architecture

### Smart reminders

`ReminderService.ts` owns validated CRUD, UUID generation, recurrence, sorting, completion, and persistence in settings. `ReminderScheduler.ts` owns one wake timer, checks at most every 60 seconds, presents reminders up to 24 hours overdue, deduplicates delivery, completes/advances recurrence, retries safely, and reconciles after system resume.

`DailyPlannerService.ts` derives a local-day briefing from incomplete future reminders.

Port these services together to Rust so timing and persistence remain authoritative when the companion WebView is closed or reloaded.

### Pomodoro

The main process owns Pomodoro state and broadcasts snapshots/completion. The renderer only presents it and plays the selected completion sound. Port the manager as one Rust state machine/actor with serialized commands and deadline materialization.

### Hydration

Unlike smart reminders, `WaterReminder.ts` is currently a renderer timer. It survives settings changes while the companion renderer exists but is not main-process durable and has no Electron `powerMonitor` reconciliation. Preserve this behavior for parity or explicitly schedule an improvement after parity; do not accidentally merge it into smart reminders during the infrastructure migration.

## AI Architecture

### Provider and request flow

`AIService.ts` owns one active provider, configuration fingerprinting, provider disposal, ask/list/test operations, and output limits. `AIRequestManager.ts` provides:

- one active operation per renderer role;
- abort cancellation on navigation/crash/close;
- chat limit of 30 requests/minute;
- connection/model discovery limit of 12 requests/minute.

`main.ts` embeds the current time, time zone, and conversation history into an assistant-action prompt. Provider output is untrusted. The action parser permits only:

- `createReminder`; and
- `setStickyMessage`.

The main process validates and executes those actions through authoritative services.

### Providers

| Provider | Current implementation | Important behavior | Tauri recommendation |
| --- | --- | --- | --- |
| OpenAI | `OpenAIProvider.ts` over `OpenAICompatibleProvider.ts` and `openai` | Responses/Chat Completions fallback, model listing/testing, timeout/retry limits | Rust `reqwest` OpenAI-compatible client |
| Grok | `GrokProvider.ts` over compatible provider | xAI base URL and model endpoint | Same compatible Rust client with provider policy |
| Custom/OpenRouter-compatible | `OpenAICompatibleProvider.ts` | User base URL, model discovery, constrained errors | Rust client with the exact URL policy and response limits |
| Gemini | `GeminiProvider.ts` with dynamic `@google/genai` import | `generateContent`, model list/test, timeout | Gemini REST implementation in Rust |
| Ollama | `OllamaProvider.ts`, `OllamaEndpointPolicy.ts`, `OllamaTransport.ts` | Localhost-only endpoint; DNS loopback verification; pinned resolved IP; remote address verification; no redirects; identity encoding; timeout and size limits | Custom Rust `reqwest`/Hyper transport preserving all SSRF and response-limit controls |

Provider streaming methods exist in the interface but current providers report streaming unsupported. No streaming migration is required for parity.

### Why the AI stack should move to Rust

Moving provider requests into the renderer would expose API keys to WebView memory, create CORS differences, weaken Ollama SSRF controls, and broaden capabilities. A bundled Node sidecar would retain most of the runtime and packaging cost that Tauri is intended to remove. A sidecar is acceptable only as a time-bounded transition spike with a documented deletion milestone.

Port the action parser/executor and provider network boundary to Rust. Keep presentation-only conversation reducers and Markdown/typewriter code in TypeScript.

## Notification and Audio Architecture

Ducky does not use Electron's native `Notification`. A “notification” is a React reminder/Pomodoro widget plus a short WAV played by `NotificationSoundService.ts`.

The sound service:

- maps event types to one selected built-in sound;
- preloads HTML `Audio` elements;
- applies independent volume;
- does not play if disabled or volume is zero;
- stops the previous sound before another starts;
- coordinates windows with `BroadcastChannel`;
- never loops.

No Tauri notification plugin is required for parity. Keep the service and assets. Verify WAV decoding, autoplay policy, hidden-window behavior, and cross-window `BroadcastChannel` support in WKWebView, WebView2, and WebKitGTK.

## Dependency Inventory

Installed direct versions are recorded from the current lock/install state.

### Runtime dependencies

| Package | Version | Current use | Action |
| --- | ---: | --- | --- |
| `react` | 19.2.8 | Both renderers | **Keep** |
| `react-dom` | 19.2.8 | Both renderers/portal | **Keep** |
| `openai` | 6.48.0 | Node OpenAI/Grok/custom clients | **Remove after Rust provider parity** |
| `@google/genai` | 2.13.0 | Node Gemini provider | **Remove after Rust provider parity** |
| `ollama` | 0.6.3 | Ollama types/client integration | **Remove after Rust provider parity** |
| `electron-updater` | 6.8.9 | GitHub release updater | **Replace with Tauri updater** |

### Development/build dependencies

| Package | Version | Action |
| --- | ---: | --- |
| `electron` | 43.2.0 | Remove after final Electron transition build |
| `electron-builder` | 26.15.3 | Replace with Tauri CLI/bundler |
| `vite` | 8.1.5 | Keep |
| `@vitejs/plugin-react` | 6.0.4 | Keep |
| `typescript` | 7.0.2 | Keep |
| React/Node type packages | current locked versions | Keep; Node types remain useful for Vite/scripts/tests |
| `concurrently`, `cross-env`, `nodemon`, `wait-on` | development orchestration | Remove when Tauri `beforeDevCommand` replaces the three-process Electron loop |
| `rimraf` | artifact cleanup | Keep if still useful |

Electron packaging brings transitive `@electron/get`, `@electron/asar`, `@electron/fuses`, `@electron/notarize`, `@electron/osx-sign`, `@electron/rebuild`, `@electron/universal`, `@electron/windows-sign`, `app-builder-lib`, `builder-util`, `dmg-builder`, `electron-publish`, `electron-winstaller`, and related packages. They disappear when Electron/Electron Builder are removed; none are application-level features to port.

### Dependencies to add

Minimum likely additions:

- JavaScript: `@tauri-apps/api`, `@tauri-apps/cli`;
- Rust: `tauri`, `serde`, `serde_json`, async runtime primitives, HTTP/TLS, URL, UUID, and logging dependencies;
- official plugins: updater and autostart;
- optional official plugins only if selected: dialog and logging.

Do not add shell, filesystem, HTTP, notification, clipboard, global-shortcut, Store, window-state, or positioner plugins merely because they exist. Native backend code can perform the narrow operations Ducky needs with a smaller frontend capability surface.

Do not mix major renderer/toolchain upgrades into the migration unless a Tauri compatibility issue requires one. Dependency upgrades and architecture replacement should be separately reviewable.

## Build and Release Pipeline

### Current local build

| Script | Current behavior |
| --- | --- |
| `npm run dev` | Runs Vite, TypeScript main watcher, and Electron under nodemon |
| `npm run typecheck` | Checks main, renderer, and Vite configs |
| `npm run build` | Cleans, type-checks, compiles Node main/preloads, builds two Vite pages |
| `npm test` | Builds main TypeScript and runs 19 Node test files |
| `npm run dist:*` | Builds and invokes Electron Builder for a platform |
| `npm run release:verify` | Verifies expected Electron artifacts/update metadata |

`vite.config.ts` already uses a Tauri-compatible relative base, fixed local dev server, and two HTML inputs. Configure Tauri `beforeDevCommand`, `devUrl`, `beforeBuildCommand`, and `frontendDist` around this existing build.

### Current packaging

`electron-builder.yml` produces:

- macOS universal DMG and ZIP plus Electron update metadata/blockmaps;
- Windows x64 NSIS and MSI plus metadata/blockmap;
- Linux x64 AppImage and DEB plus metadata;
- maximum-compression ASAR with hardened Electron fuses.

The macOS and Windows packages are currently not identity/code signed. macOS hardened runtime is enabled, but `identity: null`; Windows `signExecutable` is false.

### Current GitHub release workflow

`.github/workflows/release.yml`:

1. triggers on `vMAJOR.MINOR.PATCH`;
2. verifies tag/package/lock versions and that the tagged commit is on `main`;
3. installs from lockfile and runs tests;
4. builds independently on `macos-latest`, `windows-latest`, and `ubuntu-latest`;
5. caches npm, Electron downloads, and Electron Builder tools;
6. installs Linux packaging support;
7. verifies/stages each platform's assets;
8. aggregates assets and generates SHA-256 checksums;
9. creates/reuses a draft release, refuses to overwrite a published release, uploads all verified assets, and publishes only after success.

The aggregation/publication architecture is worth preserving. Replace only its toolchain and artifact contract.

### Tauri updater and cutover implications

Tauri updater signatures are mandatory and cannot be disabled. The Tauri release must introduce:

- a public updater key in Tauri configuration;
- `TAURI_SIGNING_PRIVATE_KEY` (and password if used) as GitHub Actions secrets;
- updater `.sig` files;
- Tauri updater JSON/endpoints with platform/architecture URLs and signatures;
- Rust toolchains/targets and Rust build caching;
- Tauri platform prerequisites on Linux.

Electron's `latest-mac.yml`, `latest.yml`, `latest-linux.yml`, ZIP/NSIS blockmaps, and Electron packages are not a Tauri update feed. Installed Electron clients cannot be assumed to self-update to the Tauri application.

Use a deliberate cutover:

1. Keep the Electron release workflow operational during development.
2. Assign unambiguous Tauri artifact names so Electron and Tauri assets cannot collide.
3. Test whether each Tauri installer upgrades/replaces the installed Electron application while preserving data.
4. Publish a final Electron transition release if a manual handoff or credential migration is required.
5. Keep legacy Electron feed assets available for existing clients.
6. Switch website latest-asset selection only when Tauri installers are production-ready; current website patterns such as “first `.exe`” may become ambiguous.
7. Update README/site “Electron” wording only at the cutover release.

### Updater phase ownership

The updater migration is intentionally split across two phases:

- **Phase 10 — runtime parity:** implement the native updater abstraction,
  preserve the existing `UpdateStatus` contract, Preferences-only
  DesktopBridge status/check methods, `updates:status-changed`,
  `updates.automatic`, startup/manual checks, persistence, least-privilege
  authorization, and the approved one-time Electron → Tauri migration dialog.
  Test the runtime through a deterministic backend. Do not add renderer
  download/install/restart controls, updater menus, or updater notifications.
- **Phase 11 — release infrastructure:** configure the stable updater public
  key and endpoints, protect private signing material in CI, generate signed
  artifacts/`.sig`/`latest.json`, host the GitHub release feed, preserve legacy
  Electron metadata, add code signing/notarization, and perform staged
  production updater and transition-release verification.

Phase 10 may define and compile the Tauri adapter boundary, but it does not use
placeholder signing identities or claim live production update verification.
Phase 11 must not use its release responsibilities to expand the
renderer-visible updater behavior established by Phase 10.

### Functional parity closure ownership

**Phase 11.5 — Functional Parity Closure** is a mandatory feature-migration
gate between release infrastructure and Electron removal.

The Phase 12 pre-removal audit found that earlier phase contracts deferred
several working Electron behaviors without assigning all of them to a later
executable phase. Those behaviors cannot be implemented during Phase 12
because Phase 12 is cleanup only, and they cannot be deleted without violating
the migration-wide parity rule.

Phase 11.5 owns:

- Set My Name panel events, settings mutation, persistence, and bridge parity;
- Sticky Message set/clear panel events, persistence, and bridge parity;
- the Rust Daily Planner backend and its existing renderer contract;
- hydration Preferences and runtime-settings parity while retaining the
  renderer-owned timer;
- the complete Electron-equivalent dynamic companion context menu;
- complete Tauri composition of the existing companion DesktopBridge;
- the final disposition and verification record for the Electron-only
  one-time migration dialog; and
- any additional renderer-accessible Electron behavior found during the final
  parity audit.

The migration dialog remains an Electron transition-release obligation rather
than ongoing Tauri functionality. Phase 11.5 verifies that obligation and
records when its repository implementation can be deleted; it does not invent
a Tauri-side prompt.

Phase 12 may begin only after Phase 11.5 proves that no required
renderer-accessible Electron behavior remains without a working Tauri
equivalent or an explicitly completed legacy-transition disposition.

## Tauri Equivalent Summary by Feature

| Ducky feature | Files | Mapping | Difficulty | Key strategy |
| --- | --- | --- | --- | --- |
| Companion window | `window.ts`, `PsyDuck.tsx`, CSS | PARTIAL | High | Early cross-platform window spike; Rust owns bounds and labels |
| Preferences window | `preferencesWindow.ts`, `PreferencesApp.tsx` | DIRECT | Low | Named singleton WebviewWindow |
| IPC/security | authorization, preloads, shared events/types | PARTIAL | High | First-party command permissions + per-label capabilities + label assertions |
| Tray/menus | `tray.ts`, `menus.ts` | DIRECT | Medium | Rust native menus and targeted UI events |
| Settings | settings shared/main/renderer files | CUSTOM | Medium | Rust repository, same JSON schema, generated/mirrored DTOs |
| Credentials | `CredentialManager.ts` | CUSTOM | High | OS vault/Stronghold decision and explicit legacy handoff |
| Smart reminders/planner | reminder/scheduler/planner files | CUSTOM | Medium–high | Rust services and durable deadlines |
| Pomodoro | manager/shared/hook/widget | CUSTOM | Medium | Rust state machine; renderer unchanged |
| Hydration | `WaterReminder.ts` and renderer orchestration | DIRECT | Low | Keep renderer timer for parity |
| Eye tracking | main cursor sampling, `EyeTracker`, `PsyDuck` | PARTIAL | Medium | Rust cursor channel, keep visual smoothing |
| Dragging | `DragController`, `PsyDuck`, move handler | PARTIAL | Medium | Keep gesture semantics; replace native position call |
| Sprite animation | engine, `PsyDuck`, character assets | DIRECT | Low | Keep Vite/DOM/rAF implementation |
| Speech/Markdown/typewriter | shared + renderer components/hooks | DIRECT | Low | Keep |
| AI providers | `src/ai`, request management/action processing | CUSTOM | High | Rust HTTP/provider layer; preserve limits and SSRF controls |
| Notification sounds | sound service/assets | DIRECT | Low | Keep Web Audio/HTML Audio implementation |
| Native notifications | none | NOT NEEDED | None | Do not add |
| Auto-launch | main settings application | DIRECT | Medium | Official autostart plugin, Rust-only use |
| Updates | UpdateService/build/release | PARTIAL | High | Phase 10 preserves status/check/settings/events; Phase 11 supplies signed artifacts/feed and production verification |
| About/branding | appBranding/constants/icons | DIRECT | Low | Tauri bundle metadata and dialog |
| Logging | scattered console logging | PARTIAL | Medium | Rust structured logging, optional log plugin |

## Recommended Tauri Architecture

### Process and trust boundary

```text
System WebViews (untrusted inputs)
├── companion renderer
│   └── companion-only DesktopBridge
└── preferences renderer
    └── preferences-only DesktopBridge
           │
           ▼
Tauri runtime authority
├── exact local origins
├── separate label-based capabilities
├── command permissions
└── command-level label checks
           │
           ▼
Rust application core
├── settings/credentials
├── reminders/Pomodoro/planner
├── AI providers/actions
├── updater
├── windows/tray/menus
└── targeted events/channels
```

### State management

- Manage one `AppState` through Tauri `State`.
- Keep independently synchronized services rather than one giant mutex.
- Use `Arc<RwLock<...>>` for read-heavy snapshots and serialized actors/mutexes for state machines or persistence.
- Never hold a lock across slow provider/network work.
- Keep one cancellation token per active AI request and bounded request concurrency.
- Persist only authoritative state; renderer local UI state remains in React.
- Emit immutable, secret-free DTOs after successful persistence.

### Command design

- Commands should be small domain operations, not generic file/HTTP/shell escape hatches.
- Return typed serializable results with stable, sanitized error codes.
- Validate lengths, enums, URLs, IDs, and dates in Rust even if TypeScript already validates them.
- Accept `WebviewWindow` and verify `companion`/`preferences` labels for privileged operations.
- Use events only for low-frequency facts. Use a channel for cursor samples and possibly updater progress.
- Provide `get_*_snapshot` commands so reload recovery does not depend on event timing.

### Shared code strategy

Keep TypeScript code that is genuinely renderer-only:

- React components/hooks;
- conversation presentation reducer;
- animation/eye smoothing/drag gesture code;
- Markdown/typewriter/personality display helpers;
- model metadata used for display.

Authoritative domain mutation and privileged validation belong in Rust. Avoid manually drifting DTOs by generating TypeScript types from Rust where practical, or at minimum maintain shared JSON fixtures and contract tests for settings, reminders, Pomodoro, AI results, and updater status.

## Recommended Folder Structure

```text
src/
├── desktop/
│   ├── DesktopBridge.ts
│   ├── electronBridge.ts        # temporary
│   ├── tauriBridge.ts
│   └── contracts.ts
├── engine/                      # retained
├── personality/                 # retained where renderer-only
├── renderer/                    # retained
└── shared/                      # retained renderer DTO/helpers

src-tauri/
├── Cargo.toml
├── Cargo.lock
├── build.rs
├── tauri.conf.json
├── capabilities/
│   ├── companion.json
│   └── preferences.json
├── icons/
└── src/
    ├── main.rs
    ├── lib.rs
    ├── app_state.rs
    ├── error.rs
    ├── lifecycle.rs
    ├── commands/
    │   ├── ai.rs
    │   ├── companion.rs
    │   ├── pomodoro.rs
    │   ├── preferences.rs
    │   ├── reminders.rs
    │   └── updates.rs
    ├── domain/
    │   ├── ai/
    │   ├── pomodoro/
    │   ├── reminders/
    │   └── settings/
    ├── infrastructure/
    │   ├── credentials.rs
    │   ├── persistence.rs
    │   ├── providers/
    │   └── updater.rs
    └── desktop/
        ├── branding.rs
        ├── menus.rs
        ├── tray.rs
        └── windows/
            ├── companion.rs
            └── preferences.rs
```

Keep `electronBridge.ts` only while both shells are runnable. The React tree should consume `DesktopBridge`, not import Electron or Tauri APIs directly.

## Migration Risks

| Risk | Severity | Evidence | Mitigation / exit criterion |
| --- | --- | --- | --- |
| Existing encrypted API keys become unreadable | Critical | Electron `safeStorage` ciphertext in settings | Tested OS-specific handoff or explicit safe re-entry UX before cutover |
| Existing Electron app cannot consume Tauri updater feed | Critical | Different manifests/artifacts; Tauri signatures mandatory | Dual release/cutover plan; retain legacy assets |
| Installer upgrade overwrites/uninstalls incorrectly | High | Same app identity but different bundlers/install tech | Clean/install/upgrade/uninstall matrix from every supported Electron package |
| Ollama SSRF protections regress | High | Custom DNS/IP/remote-address checks in current transport | Port every invariant with adversarial Rust tests before enabling |
| Window transparency/drag differs across system WebViews | High | Chromium is replaced by WKWebView/WebView2/WebKitGTK | Physical three-OS visual/input test matrix |
| Capability model becomes broader than Electron role policy | High | Custom Tauri commands are broadly available unless permissioned | Command manifest/internal plugin permissions + separate capabilities + label assertions |
| System suspend is not detected identically | High | Electron explicitly listens to `powerMonitor.resume` | Deadline reconciliation on timer/focus/resume plus suspend tests |
| Data path changes reset settings/reminders | High | Electron `userData` and Tauri app data may differ | One-time legacy locator/import with idempotent marker and fixtures |
| Coordinate scale drift breaks eye/drag on mixed DPI | High | Current use of screen + window/DOM coordinates | Define coordinate contract and test negative/mixed-scale monitors |
| Early events are lost on WebView reload | Medium | Preloads currently buffer several events | Rust pending queues/flags and initial snapshot commands |
| WebView audio/BroadcastChannel behavior differs | Medium | Current notification service is browser-based | Packaged tests on all system WebViews |
| Website downloads wrong artifact | Medium | Generic platform extension matching meets new artifact set | Explicit Tauri asset naming/selection tests before publication |
| Current unsigned installers remain warning-prone | Medium | Electron config disables signing | Treat Tauri updater signing and OS code signing as distinct requirements |
| Old documentation causes scope inflation | Medium | Aspirational systems are not implemented | Maintain implementation-derived parity checklist |

## Recommended Migration Order

1. **Freeze observable contracts.** Capture settings/Pomodoro fixtures, IPC payloads, window behavior, screenshots, release names, provider errors, and cross-platform smoke cases.
2. **Add the frontend bridge abstraction.** Make renderers call `DesktopBridge` while the Electron implementation still backs it; no behavior change.
3. **Scaffold Tauri beside Electron.** Add a minimal `src-tauri/`, two windows, exact capabilities, and Vite integration.
4. **Prove the companion shell.** Transparency, bottom-right placement, dynamic height, drag, eye cursor channel, tray, context menu, and Preferences singleton on all three OSes.
5. **Port persistence and credentials.** Settings schema, atomic files, legacy path import, credential vault, and secret-free projections.
6. **Port time-based domains.** Reminders, scheduler, daily planner, Pomodoro, wake/relaunch reconciliation, and events.
7. **Port AI.** Compatible client, Gemini, Ollama security transport, request manager, action parser/executor, diagnostics, and cancellation.
8. **Port updater and release pipeline.** Signed updater artifacts/feed, platform bundles, verifier, checksums, website selection, and dual-feed cutover.
9. **Close deferred functional parity.** Finish profile, sticky message, Daily Planner, hydration, dynamic context-menu, complete companion bridge, and legacy migration-dialog disposition work.
10. **Run parity/security/performance validation.** Three OSes, multiple displays/DPI, suspend/resume, renderer reload/crash, offline launch, upgrade/uninstall, provider failure, and accessibility.
11. **Cut over, then delete Electron.** Do not remove Electron files/dependencies until the Tauri artifact passes every release and Phase 11.5 functional-parity gate.

## Recommended Milestones

### Milestone 0 — Baseline and contracts

Deliver:

- current behavior matrix and visual baselines;
- golden settings/reminder/Pomodoro fixtures;
- IPC DTO catalog and security expectations;
- installer/update inventory.

Exit: Electron remains fully releasable and baseline tests are reproducible.

### Milestone 1 — Dual-shell frontend bridge

Deliver:

- `DesktopBridge` interface;
- Electron adapter preserving current preloads;
- Tauri adapter skeleton;
- zero component-level Electron/Tauri imports.

Exit: Electron behavior/tests remain unchanged.

### Milestone 2 — Tauri desktop shell

Deliver:

- both windows, capabilities, tray/menus, CSP, app lifecycle;
- cursor channel, dynamic height, drag/window commands;
- macOS Spaces and platform visual tests.

Exit: non-persistent UI shell parity on all target OSes.

### Milestone 3 — Persistence and native settings

Deliver:

- Rust settings repository and migrations;
- legacy data import;
- credential vault/handoff;
- autostart/always-on-top integration.

Exit: no credential loss and fixture parity on all target OSes.

### Milestone 4 — Productivity domains

Deliver:

- Rust reminders/scheduler/planner;
- Rust Pomodoro;
- wake, restart, and WebView recovery;
- preserved sound/widget behavior.

Exit: deterministic fake-clock and packaged suspend/resume tests pass.

### Milestone 5 — AI providers and actions

Deliver:

- all five provider modes;
- model listing/testing;
- cancellation/rate limits;
- Ollama network boundary;
- assistant action validation/execution.

Exit: provider contract, adversarial URL/network, size, timeout, and secret-exposure tests pass.

### Milestone 6 — Distribution and updater

Deliver:

- Tauri bundles;
- updater keys/signatures/feed;
- release workflow/verifier;
- legacy Electron transition and website download plan.

Exit: clean install, Electron-to-Tauri upgrade, Tauri update, rollback policy, and uninstall tested per OS.

### Milestone 6.5 — Functional parity closure

Deliver:

- complete Tauri Set My Name and Sticky Message paths;
- Rust Daily Planner backend and renderer bridge;
- hydration settings parity with the existing renderer timer;
- Electron-equivalent dynamic companion context menu;
- complete Tauri companion DesktopBridge composition;
- verified final Electron migration-dialog disposition;
- final Electron-versus-Tauri behavior matrix.

Exit: no renderer-accessible Electron-only behavior remains, every
Phase 11.5 parity check passes, and Electron is still intact and releasable.

### Milestone 7 — Cutover and cleanup

Deliver:

- Tauri as default dev/build/start path;
- updated accurate documentation;
- removed Electron implementation/dependencies/config;
- performance/security review.

Exit: production release candidate with no required Node runtime/sidecar.

## Everything That Must Be Migrated

- Main application startup, activation, shutdown, and tray-resident lifecycle.
- Companion and Preferences window creation, placement, sizing, security, and singleton behavior.
- All 36 IPC channel contracts and early-event recovery semantics.
- IPC role authorization and deny-by-default permission boundary.
- Cursor sampling and native window movement/resizing.
- Native app/context/tray menus and About/branding behavior.
- Settings persistence, parsing, migrations, recovery files, and broadcasts.
- Credential encryption/decryption and a safe existing-user transition.
- Smart reminder CRUD, recurrence, scheduler, resume behavior, and Daily
  Planner, including the Phase 11.5 parity closure.
- Pomodoro persistence, ticking, pause/resume/completion, and recovery.
- AI configuration, all providers, request limits/cancellation, diagnostics, and assistant actions.
- Always-on-top, start-at-login, theme/background, restart, and quit behavior.
- Update status/check/events/settings runtime parity and the approved manual
  Electron → Tauri migration dialog (Phase 10).
- Release verification, bundling, updater public-key configuration, update
  signing, hosted metadata, GitHub publication, notarization, production
  updater verification, and website asset selection (Phase 11).
- Deferred profile, sticky-message, hydration, dynamic context-menu, complete
  companion bridge, Daily Planner, and legacy transition-disposition work
  discovered by the pre-removal audit (Phase 11.5).
- Main-process tests that assert behavior of code being moved to Rust.

## Everything That Can Remain Unchanged or Nearly Unchanged

- React and ReactDOM renderers.
- Vite's two-page renderer structure and relative production base.
- Companion and Preferences CSS, subject to WebView regression fixes.
- Official mascot, animation frames, icons, and WAV assets.
- React components and accessibility behavior.
- Conversation UI lifecycle and renderer presentation state.
- DOM/rAF sprite animation, visual behavior scheduling, typewriter, Markdown, and eye smoothing.
- Notification sound service, pending platform WebView QA.
- Pure display/model metadata and renderer reducers.
- Build-time WAV generation script.
- Standalone Next.js website, Supabase analytics, and website auth, except release copy/asset selection at cutover.

## Everything That Should Be Rewritten

- All privileged Node main-process services in Rust.
- AI provider networking and Ollama transport in Rust.
- Settings/Pomodoro repositories in Rust.
- Credential backend and migration.
- Electron IPC registration/authorization as Tauri commands/capabilities.
- Native lifecycle/window/menu/tray adapters.
- Updater runtime abstraction in Phase 10 and the release artifact
  verifier/workflow in Phase 11.
- Daily Planner backend and any remaining privileged Personal Assistant or
  dynamic-menu behavior assigned to Phase 11.5.
- Main-process unit tests as Rust tests, while retaining useful black-box TypeScript contract fixtures.

## Everything That Should Be Deleted After Parity

Do not delete these early. Delete them only after Phase 11.5 functional parity
and the applicable transition-release obligation are complete:

- `src/main/preload.ts`;
- `src/main/preferencesPreload.ts`;
- Electron-specific implementations in `src/main/main.ts`, `window.ts`, `preferencesWindow.ts`, `ipcAuthorization.ts`, `rendererSecurity.ts`, `permissionPolicy.ts`, `menus.ts`, `tray.ts`, and `appBranding.ts`;
- TypeScript main-service/provider files only after equivalent Rust modules and tests exist;
- `electron-builder.yml`;
- Electron-specific branches in `scripts/verify-release-artifacts.mjs`;
- `tsconfig.main.json` if no transitional Node main code remains;
- `electron`, `electron-builder`, `electron-updater`, `openai`, `ollama`, and `@google/genai` after Rust replacements;
- Electron dev orchestration packages that have no remaining use;
- generated Electron metadata/blockmap expectations from documentation and CI.

Do not delete `website/`, assets, renderer code, or legacy published GitHub assets.

## Validation and Release Gates

### Functional parity

- Companion starts at the correct screen edge and remains reachable.
- Dynamic widgets never crop and keep a stable bottom edge.
- Click, drag, context menu, pin, Continue Chat, panels, and Preferences work.
- Set My Name, Sticky Message, Daily Planner, hydration settings, and every
  dynamic context-menu state/action match Electron.
- The Tauri companion DesktopBridge implements every renderer-used member;
  no required feature is hidden behind an unavailable runtime adapter.
- Tray lifecycle and all native menu commands work.
- Settings/reminders/Pomodoro survive restart and invalid-file recovery.
- AI ask/list/test works for every provider and actions remain allowlisted.
- Sounds never overlap and disabled/zero-volume behavior is unchanged.

### Platform parity

- macOS Intel and Apple Silicon/universal strategy;
- Windows x64 NSIS and/or MSI upgrade/uninstall;
- Linux x64 AppImage and DEB;
- mixed-DPI and negative-coordinate multi-monitor layouts;
- macOS Spaces/full-screen;
- suspend/resume, lock/unlock, offline startup, and WebView reload/crash.

### Security

- No renderer receives API keys or generic fs/http/shell capability.
- Companion cannot call Preferences/AI-configuration/update commands.
- Preferences cannot call companion movement/reminder/chat commands.
- No remote origin receives a capability.
- Ollama remains loopback-only after DNS resolution and connection.
- Custom provider URL rules and response limits remain enforced.
- Phase 10 grants only the Preferences status/check authority present in
  Electron.
- Phase 11 verifies Tauri updater signatures against the stable production
  public key.

### Release

- Tag/version/Cargo/Tauri/package versions match.
- Every expected package, updater signature, and metadata file exists.
- Release publication remains atomic and avoids published-asset replacement.
- Existing Electron installs have an explicit supported transition.
- Website routes select only intended Tauri installer assets.
- The Phase 11.5 record proves the one-time migration dialog's final Electron
  release obligation before its repository source is removed.

These release gates belong to Phase 11. They are not prerequisites for
implementing or unit-testing the Phase 10 runtime abstraction, and Phase 10
must not use placeholder production signing material to satisfy them early.
The separate Phase 11.5 functional-parity gate is mandatory before Phase 12
regardless of release-infrastructure readiness.

## Complexity Conclusion

| Dimension | Assessment |
| --- | --- |
| Renderer reuse | High reuse / low migration complexity |
| Native shell | Medium–high due transparent multi-window behavior |
| Domain services | Medium–high Rust port |
| AI/security | High |
| Credentials | High and release-critical |
| Updater/distribution | High and release-critical |
| Overall | **High; staged migration strongly recommended** |

Tauri is a sound target for Ducky because the UI is already a Vite web application and the privileged surface is well separated. The safe migration is to preserve that separation, narrow it further through Tauri capabilities, and port the authoritative backend deliberately rather than moving privileged behavior into the renderer.

## Official Tauri v2 References Used

- [Calling Rust from the frontend](https://v2.tauri.app/develop/calling-rust/)
- [Capabilities and command security](https://v2.tauri.app/security/capabilities/)
- [Runtime Authority](https://v2.tauri.app/security/runtime-authority/)
- [Window customization](https://v2.tauri.app/learn/window-customization/)
- [System tray](https://v2.tauri.app/learn/system-tray/)
- [Window menus](https://v2.tauri.app/learn/window-menu/)
- [Autostart plugin](https://v2.tauri.app/plugin/autostart/)
- [Updater plugin](https://v2.tauri.app/plugin/updater/)
- [Stronghold plugin](https://v2.tauri.app/plugin/stronghold/)
- [Logging plugin](https://v2.tauri.app/plugin/logging/)
- [GitHub Actions distribution](https://v2.tauri.app/distribute/pipelines/github/)
- [Tauri Rust API documentation](https://docs.rs/tauri/latest/tauri/)
