# Ducky Electron → Tauri v2 Migration Tasks

> Source of Truth:
> `docs/migrating/migration_codex.md`
>
> This document is the executable migration plan.
>
> Every implementation task MUST follow the architecture defined in
> `migration_codex.md`.
>
> This file defines the order of execution.
>
> Do NOT deviate from it.

---

# GLOBAL RULES

These rules are mandatory.

## Build Integrity

- The repository must remain buildable after every completed task.
- Never intentionally leave the project in a broken state.
- Never disable TypeScript checks to make progress.
- Never comment out failing code as a shortcut.
- Never introduce placeholder implementations into production code.

---

## Scope Rules

Only perform work described in this document.

Do NOT:

- redesign the product
- invent new features
- upgrade unrelated dependencies
- refactor unrelated code
- change UI behavior unless required
- rename files without necessity

---

## Architecture Rules

The architecture defined inside

docs/migrating/migration_codex.md

is the source of truth.

If implementation differs from assumptions:

STOP

Document the discrepancy.

Do not invent a new architecture.

---

## Safety Rules

Never delete Electron code until the corresponding Tauri implementation reaches feature parity.

Never migrate two independent systems simultaneously.

Only one migration task may be active at a time.

---

## Validation Rules

After every completed task run:

- npm install (if dependencies changed)
- npm run typecheck
- npm test (if affected)
- npm run build

If any validation fails:

Fix it before continuing.

---

## Commit Rules

Every completed task must become one commit.

Commit message format:

feat(tauri): <task>

or

refactor(tauri): <task>

No task may span multiple unrelated commits.

---

## Progress Tracking

Maintain:

docs/migrating/progress.md

After every completed task update:

- completed task
- files changed
- validation performed
- blockers
- next task

---

# PHASE 0

Repository Preparation

Goal:

Prepare the repository for migration without changing runtime behavior.

---

Task 0.1

Verify repository state

Acceptance

- clean working tree
- dependencies install
- build succeeds
- tests pass

---

Task 0.2

Create DesktopBridge abstraction

Objective

Create a renderer abstraction that hides Electron/Tauri implementations.

Acceptance

Renderer compiles without importing Electron directly.

---

Task 0.3

Move renderer to DesktopBridge

Acceptance

Application behavior unchanged.

---

PHASE COMPLETE WHEN

Renderer no longer depends directly on Electron APIs.

---

# PHASE 1

Create Tauri Workspace

Task 1.1

Initialize Tauri v2

Acceptance

src-tauri exists

Cargo builds

Vite launches

No Electron code modified

---

Task 1.2

Configure two windows

Acceptance

Companion window

Preferences window

Both launch

---

Task 1.3

Capabilities

Acceptance

Separate capabilities

No wildcard permissions

---

Phase Exit Criteria

Tauri launches successfully.

Electron still builds.

---

# PHASE 2

Desktop Window Migration

Task 2.1

Companion window

Task 2.2

Transparent window

Task 2.3

Always on top

Task 2.4

Positioning

Task 2.5

Dragging

Task 2.6

Dynamic height

Task 2.7

Cursor channel

Exit Criteria

Behavior visually matches Electron.

---

# PHASE 3

IPC Migration

Task 3.1

Commands

Task 3.2

Events

Task 3.3

Cursor streaming

Task 3.4

Authorization

Task 3.5

DesktopBridge → Tauri

Exit Criteria

All 36 IPC channels migrated.

---

# PHASE 4

Tray + Native Menu Migration

Goal

Migrate the native system tray and native application menu to Tauri v2 while
preserving existing Electron behaviour.

Electron remains the reference implementation until Phase 12.

DesktopBridge remains the only renderer abstraction.

Native tray callbacks remain inside the runtime and are never routed through
the renderer.

---

## Discovery

Before implementation:

- Inspect the existing Electron tray implementation.
- Inspect tray lifecycle.
- Inspect tray icon loading.
- Inspect native application menu.
- Inspect tray menu.
- Inspect renderer interactions.
- Inspect DesktopBridge integration points.
- Document findings inside docs/migrating/progress.md.

Do not modify production code until discovery is complete.

---

## Scope

This phase includes:

- native tray creation
- tray lifecycle
- tray destruction
- tray icon loading
- tray startup integration
- static tray menu
- macOS application menu
- native menu callbacks
- show companion
- focus companion
- show preferences
- about
- restart
- quit
- DesktopBridge support for renderer-requested context menus
- least-privilege permissions

This phase does NOT include:

- dynamic companion context menu
- settings
- runtime settings
- reminder system
- water reminders
- pomodoro
- daily planner
- sticky message
- user profile
- credentials
- AI
- updater
- release pipeline
- Electron removal

---

## Architecture Rules

Native tray events belong to the runtime.

Examples:

- tray click
- menu selection
- restart
- quit
- show companion
- show preferences

These actions execute directly inside Rust.

DesktopBridge is only used when the renderer requests native functionality,
such as opening a companion context menu.

Renderer-originated requests:

Renderer
↓

DesktopBridge
↓

Runtime Adapter
↓

Rust

Native-originated callbacks:

Rust
↓

Native Window APIs

Do not route native callbacks back through DesktopBridge.

---

## Task 4.1

Audit Existing Tray Behaviour

Objective

Document the Electron implementation.

Acceptance

- tray lifecycle documented
- menu hierarchy documented
- renderer integration documented
- DesktopBridge touch points documented
- platform differences documented

Commit

No commit.

---

## Task 4.2

Create Native Tray Infrastructure

Objective

Implement the native Tauri tray.

Requirements

- tray owned by Rust
- singleton lifecycle
- preserve Electron implementation
- renderer unchanged

Acceptance

- tray initializes
- tray survives startup
- tray destroyed during shutdown
- Electron unchanged

Commit

feat(tauri): migrate tray infrastructure

---

## Task 4.3

Migrate Tray Icon

Objective

Implement native tray icon loading.

Requirements

- preserve existing icon
- preserve platform sizing
- preserve icon quality
- no duplicate assets

Acceptance

- tray icon visible
- icon matches Electron
- Electron unchanged

Commit

feat(tauri): migrate tray icons

---

## Task 4.4

Migrate Static Native Menus

Objective

Implement the static tray menu and macOS application menu.

This task includes only static menus.

Requirements

Preserve:

- ordering
- labels
- separators
- native roles

Do not migrate:

- dynamic companion context menu
- checked runtime state
- reminder actions
- pomodoro actions
- settings actions

Acceptance

- tray menu matches Electron
- macOS application menu matches Electron
- menu hierarchy preserved

Commit

feat(tauri): migrate native menus

---

## Task 4.5

Migrate Native Menu Actions

Objective

Implement native callbacks.

Includes

- show companion
- focus companion
- show preferences
- about
- restart
- quit

Requirements

- callbacks execute inside Rust
- renderer remains runtime agnostic
- Electron preserved

Acceptance

All existing static menu actions function correctly.

Commit

feat(tauri): migrate menu actions

---

## Task 4.6

Renderer Context Menu Bridge

Objective

Implement only the bridge required for renderer-requested companion context
menus.

This task does NOT migrate the dynamic menu itself.

Requirements

- renderer requests native context menu
- DesktopBridge used only for request dispatch
- no menu state implemented
- no Settings integration
- no Pomodoro integration
- no Reminder integration

Acceptance

Renderer can request a native companion context menu.

Dynamic menu content remains deferred.

Commit

feat(tauri): migrate context menu bridge

---

## Task 4.7

Permissions

Objective

Grant minimum required permissions.

Requirements

- exact window labels
- no wildcard permissions
- no filesystem permissions
- no shell permissions
- no process permissions
- no unnecessary plugins

Acceptance

Permission validation passes.

Commit

feat(tauri): migrate tray permissions

---

## Deferred

The following functionality is intentionally excluded from Phase 4 and will be
implemented during their owning feature phases.

Settings

- runtime settings
- eye tracking
- always on top

Pomodoro

- timer menu
- pause
- resume
- stop
- custom duration

Reminder System

- reminder creation
- reminder management
- water reminders

Daily Planner

- planner actions

Profile

- user name

Sticky Message

- set
- clear

These features remain Electron-only until their migration phase.

---

## Validation

After every completed task:

- npm install (only if dependencies changed)
- npm run typecheck
- npm test
- npm run build
- cargo fmt
- cargo test
- cargo build
- Tauri permission validation
- Electron production build
- Electron smoke launch
- Tauri development smoke launch

Fix all failures before continuing.

---

## Manual Verification

Verify:

- tray icon visible
- tray survives startup
- tray survives restart
- tray menu opens
- menu labels correct
- menu ordering correct
- separators correct
- macOS application menu correct
- Show Companion works
- Show Preferences works
- About works
- Restart works
- Quit works
- no permission warnings
- no renderer console errors

Dynamic companion context menu is intentionally excluded from this verification.

---

## Progress Tracking

Update docs/migrating/progress.md with:

- completed task
- implementation summary
- files changed
- validation performed
- manual verification
- blockers
- next task

---

## Phase Exit Criteria

Phase 4 is complete only if:

✓ Native tray created

✓ Tray lifecycle matches Electron

✓ Tray icon matches Electron

✓ Static tray menu matches Electron

✓ macOS application menu matches Electron

✓ Show Companion works

✓ Show Preferences works

✓ About works

✓ Restart works

✓ Quit works

✓ Native callbacks remain inside Rust

✓ DesktopBridge used only for renderer-originated requests

✓ Dynamic companion context menu explicitly deferred

✓ Electron implementation preserved

✓ All validation passes

✓ Manual verification passes

✓ Repository builds successfully

Only then may Phase 5 begin.

# PHASE 5

Settings Migration

Goal

Migrate the settings infrastructure from Electron to Tauri v2 while preserving
existing behaviour.

This phase migrates only the settings system itself.

Feature-specific settings remain deferred to their owning migration phases.

Electron remains the reference implementation until Phase 12.

DesktopBridge remains the only renderer abstraction.

---

## Discovery

Before implementation:

- Audit the existing settings implementation.
- Identify every persisted setting.
- Identify default values.
- Identify storage backend.
- Identify settings read during startup.
- Identify settings modified at runtime.
- Identify renderer interactions.
- Identify DesktopBridge integration.
- Document findings inside docs/migrating/progress.md.

Do not modify production code until discovery is complete.

---

## Scope

This phase includes:

- settings storage
- settings loading
- settings persistence
- default values
- settings validation
- DesktopBridge settings API
- startup loading
- settings change notifications
- Preferences window integration

This phase does NOT include:

- reminder behaviour
- water reminder logic
- pomodoro logic
- AI settings
- updater settings
- credential storage
- release pipeline
- Electron removal

---

## Architecture Rules

Renderer never accesses runtime storage directly.

Renderer

↓

DesktopBridge

↓

Runtime Adapter

↓

Native Settings Store

All validation occurs inside the runtime.

Renderer receives typed settings objects only.

---

## Task 5.1

Audit Existing Settings

Objective

Document the current Electron settings implementation.

Acceptance

- storage backend documented
- settings list documented
- startup flow documented
- renderer interactions documented
- DesktopBridge touch points documented

Commit

No commit.

---

## Task 5.2

Create Native Settings Store

Objective

Implement the Tauri settings backend.

Requirements

- runtime owned
- typed storage
- validation
- preserve Electron implementation

Acceptance

- settings load successfully
- settings save successfully
- Electron unchanged

Commit

feat(tauri): migrate settings store

---

## Task 5.3

DesktopBridge Settings API

Objective

Expose typed settings APIs through DesktopBridge.

Requirements

- renderer runtime agnostic
- no direct Tauri APIs
- preserve Electron bridge

Acceptance

Renderer accesses settings only through DesktopBridge.

Commit

feat(tauri): migrate settings bridge

---

## Task 5.4

Startup Settings Loading

Objective

Initialize settings during application startup.

Requirements

- load before renderer requires them
- preserve defaults
- preserve existing startup behaviour

Acceptance

Application starts with identical settings behaviour.

Commit

feat(tauri): migrate settings startup

---

## Task 5.5

Settings Persistence

Objective

Persist modified settings.

Requirements

- validate before save
- avoid duplicate writes
- preserve defaults

Acceptance

Modified settings survive restart.

Commit

feat(tauri): migrate settings persistence

---

## Task 5.6

Preferences Window Integration

Objective

Connect the Preferences UI to the new settings backend.

Requirements

- no renderer runtime detection
- DesktopBridge only
- preserve Electron implementation

Acceptance

Changing settings updates the native store.

Commit

feat(tauri): migrate preferences settings

---

## Deferred

The following settings remain owned by later phases:

Reminder System

- reminder schedules
- water reminder configuration

Pomodoro

- durations
- auto-start
- break configuration

AI

- providers
- API keys
- model selection

Updater

- update preferences

Credentials

- authentication
- secrets

These remain Electron-only until their migration phases.

---

## Validation

After every completed task:

- npm install (only if dependencies changed)
- npm run typecheck
- npm test
- npm run build
- cargo fmt
- cargo test
- cargo build
- Tauri permission validation
- Electron production build
- Electron smoke launch
- Tauri development smoke launch

Fix all failures before continuing.

---

## Manual Verification

Verify:

- settings load on startup
- default values correct
- settings persist after restart
- Preferences displays current values
- changing settings updates immediately
- no duplicate writes
- no renderer console errors
- Electron behaviour unchanged

---

## Progress Tracking

Update docs/migrating/progress.md with:

- completed task
- implementation summary
- files changed
- validation performed
- manual verification
- blockers
- next task

---

## Phase Exit Criteria

Phase 5 is complete only if:

✓ Native settings store implemented

✓ DesktopBridge handles all settings access

✓ Preferences window uses the native backend

✓ Settings persist across restarts

✓ Default values preserved

✓ Electron implementation preserved

✓ All validation passes

✓ Manual verification passes

✓ Repository builds successfully

Only then may Phase 6 begin.

---

# PHASE 6

Secure Credentials & Secret Storage

Goal

Migrate secure credential storage from Electron to Tauri v2.

This phase establishes the native secret storage layer used by later phases.

It does NOT migrate AI providers, authentication flows, updater logic,
or feature-specific integrations.

Electron remains the reference implementation until Phase 12.

DesktopBridge remains the only renderer abstraction.

---

## Discovery

Before implementation:

- Audit the existing credential implementation.
- Identify every stored secret.
- Identify storage backend.
- Identify credential lifecycle.
- Identify startup loading.
- Identify runtime updates.
- Identify DesktopBridge integration.
- Identify renderer interactions.
- Document findings inside docs/migrating/progress.md.

Do not modify production code until discovery is complete.

---

## Scope

This phase includes:

- secure credential store
- native Keychain integration (macOS)
- typed credential APIs
- credential validation
- credential persistence
- DesktopBridge credential APIs
- credential loading
- credential deletion
- least-privilege permissions

This phase does NOT include:

- AI provider logic
- model selection
- authentication flows
- reminders
- pomodoro
- updater
- release pipeline
- Electron removal

---

## Architecture Rules

Renderer never accesses native secure storage directly.

Renderer

↓

DesktopBridge

↓

Runtime Adapter

↓

Native Secret Store

Secrets never travel through renderer state unless explicitly requested.

Validation occurs inside the runtime.

---

## Task 6.1

Audit Existing Credential Storage

Objective

Document the Electron implementation.

Acceptance

- storage documented
- credential lifecycle documented
- DesktopBridge interactions documented
- renderer interactions documented

Commit

No commit.

---

## Task 6.2

Create Native Secret Store

Objective

Implement the native secure credential backend.

Requirements

- runtime owned
- typed API
- native secure storage
- preserve Electron implementation

Acceptance

Credentials save successfully.

Credentials load successfully.

Commit

feat(tauri): migrate secret store

---

## Task 6.3

DesktopBridge Credential API

Objective

Expose typed credential APIs.

Requirements

- renderer runtime agnostic
- DesktopBridge only
- preserve Electron bridge

Acceptance

Renderer accesses credentials only through DesktopBridge.

Commit

feat(tauri): migrate credential bridge

---

## Task 6.4

Credential Persistence

Objective

Persist credentials securely.

Requirements

- validate before save
- overwrite existing values safely
- support deletion
- prevent duplicate writes

Acceptance

Credentials survive restart.

Deletion removes stored secrets.

Commit

feat(tauri): migrate credential persistence

---

## Task 6.5

Preferences Integration

Objective

Connect secure credential management to Preferences.

Requirements

- DesktopBridge only
- no runtime detection
- preserve Electron implementation

Acceptance

Credentials can be created, updated and removed through Preferences.

Commit

feat(tauri): migrate credential preferences

---

## Deferred

The following remain owned by later phases:

AI

- OpenAI
- Gemini
- Grok
- Ollama
- provider selection
- model selection
- AI diagnostics

Updater

- update authentication
- release credentials

Authentication

- login
- account management

These remain Electron-only until their migration phases.

---

## Validation

After every completed task:

- npm install (only if dependencies changed)
- npm run typecheck
- npm test
- npm run build
- cargo fmt
- cargo test
- cargo build
- Tauri permission validation
- Electron production build
- Electron smoke launch
- Tauri development smoke launch

Fix all failures before continuing.

---

## Manual Verification

Verify:

- credentials save
- credentials load
- credentials update
- credentials delete
- credentials survive restart
- Preferences reflects changes
- no secrets stored in renderer state
- no renderer console errors
- Electron behaviour unchanged

---

## Progress Tracking

Update docs/migrating/progress.md with:

- completed task
- implementation summary
- files changed
- validation performed
- manual verification
- blockers
- next task

---

## Phase Exit Criteria

Phase 6 is complete only if:

✓ Native secret storage implemented

✓ DesktopBridge owns all credential access

✓ Credentials persist securely

✓ Credentials survive restart

✓ Preferences integrates correctly

✓ Electron implementation preserved

✓ All validation passes

✓ Manual verification passes

✓ Repository builds successfully

Only then may Phase 7 begin.

---

# PHASE 7

Reminder System Migration

Goal

Migrate the complete Reminder System from Electron to Tauri v2 while preserving
existing reminder behaviour, scheduling, persistence, notifications, and
DesktopBridge architecture.

Electron remains the authoritative reference implementation until Phase 12.
When this contract or another migration document conflicts with Electron,
Electron behavior takes precedence unless an explicit redesign was approved
before implementation.

DesktopBridge remains the only renderer abstraction.

Reminder scheduling, timers, recurrence advancement, persistence, and fired
event production execute inside Rust.

The renderer remains runtime agnostic.

The existing React reminder widget and notification-sound service remain the
user-facing notification implementation. Phase 7 does not add native OS
notifications or new reminder capabilities.

---

## Discovery

Before implementation:

- Inspect the existing ReminderService.
- Inspect reminder scheduling.
- Inspect reminder persistence.
- Inspect reminder lifecycle.
- Inspect reminder notification flow.
- Inspect runtime startup order.
- Inspect reminder IPC.
- Inspect DesktopBridge reminder interfaces.
- Confirm reminder CRUD remains in the Companion renderer.
- Inspect tray interactions.
- Confirm no native notification plugin or permission is required.
- Document findings inside docs/migrating/progress.md.

Do not modify production code until discovery is complete.

---

## Scope

This phase includes:

- reminder engine
- reminder scheduler
- reminder persistence integration
- reminder startup restoration
- reminder creation
- reminder editing
- reminder deletion
- reminder lookup and completion
- recurring reminders
- one-time reminders
- existing `reminders:fired` notification dispatch
- existing React reminder widget, Dismiss, and Snooze behavior
- reminder DesktopBridge
- existing reminder runtime events
- existing New Reminder and Manage Reminders context-menu actions
- least-privilege permissions

This phase does NOT include:

- Pomodoro
- AI
- updater
- release pipeline
- credentials
- sticky messages
- planner
- Electron removal
- per-reminder enable/disable
- native OS notifications
- native notification click callbacks
- new reminder lifecycle events
- reminder management in Preferences
- reminder schema changes

---

## Architecture Rules

Reminder scheduling belongs to Rust.

Examples:

- timer creation
- timer cancellation
- recurring scheduling
- fired-event production
- startup restoration

DesktopBridge is only used when the renderer requests reminder operations.

Renderer

↓

DesktopBridge

↓

Runtime Adapter

↓

Rust

The existing `reminders:fired` event originates inside Rust and is delivered
to the Companion renderer through the established event architecture. The
existing React widget presents the notification and the renderer-owned sound
service plays the configured sound.

Do not execute reminder timing inside the renderer.

Do not implement browser timers as the production scheduler.

Do not install a Tauri notification plugin or request OS notification
permission.

Preserve the exact Electron reminder schema. Do not add an `enabled` field.

---

## Task 7.1

Audit Existing Reminder System

Objective

Document the Electron reminder implementation.

Acceptance

- reminder lifecycle documented
- scheduler documented
- persistence documented
- DesktopBridge documented
- notification flow documented
- startup restoration documented

Commit

No commit.

---

## Task 7.2

Create Native Reminder Engine

Objective

Implement the Rust reminder scheduler.

Requirements

- runtime owned
- singleton
- survives startup
- survives restart
- scheduler independent of renderer

Acceptance

- engine starts
- engine stops cleanly
- scheduler registered
- Electron unchanged

Commit

feat(tauri): migrate reminder engine

---

## Task 7.3

Reminder Persistence

Objective

Connect reminders to the existing native settings store.

Requirements

- preserve current schema
- preserve reminder IDs
- preserve ordering
- atomic persistence
- startup restoration

Acceptance

- reminders persist
- reminders reload
- restart restores reminders

Commit

feat(tauri): migrate reminder persistence

---

## Task 7.4

Reminder Commands

Objective

Expose reminder management through DesktopBridge.

Includes

- list reminders
- create reminder
- update reminder
- delete reminder
- get reminder
- mark reminder completed

Requirements

- typed commands
- existing request and response contracts only
- no renderer runtime detection
- Electron preserved

Acceptance

Renderer manages reminders through DesktopBridge.

Commit

feat(tauri): migrate reminder commands

---

## Task 7.5

Reminder Notifications

Objective

Migrate the existing fired-reminder delivery path.

Requirements

- existing `reminders:fired` event name and payload
- exact Companion window targeting
- pending delivery survives renderer startup/reload
- duplicate suppression
- existing React reminder widget remains unchanged
- existing Dismiss and Snooze behavior remains unchanged
- existing notification sound behavior remains unchanged
- no native OS notification plugin
- no OS notification permission

Acceptance

The React reminder notification matches Electron behavior and no fired
reminder is lost or delivered twice.

Commit

feat(tauri): migrate reminder notifications

---

## Task 7.6

Reminder Events

Objective

Implement reminder runtime events.

Includes

- `reminders:fired`
- `reminders:creation-panel-requested`
- `reminders:manager-panel-requested`

Requirements

- existing event names and payloads only
- exact window targeting
- DesktopBridge abstraction
- no renderer polling
- no additional reminder lifecycle events

Acceptance

The Companion renderer receives the existing Electron reminder events
correctly.

Commit

feat(tauri): migrate reminder events

---

## Task 7.7

Companion Integration

Objective

Connect the existing Companion reminder panels and native context-menu
actions.

Requirements

- runtime agnostic
- DesktopBridge only
- renderer unchanged
- reminder creation and schedule/recurrence editing remain in the Companion
  renderer
- reminder management remains in the Companion renderer
- New Reminder and Manage Reminders context-menu actions match Electron
- Preferences remains unchanged

Acceptance

Reminder CRUD and the existing context-menu entry points work in the Companion
window exactly as they do in Electron.

Commit

feat(tauri): migrate reminder companion integration

---

## Task 7.8

Permissions

Objective

Grant least-privilege permissions.

Requirements

- existing companion reminder commands only
- existing companion event-listen permission only
- exact window labels
- no wildcard permissions
- no unnecessary plugins
- no notification permission

Acceptance

Permission validation passes.

Commit

feat(tauri): migrate reminder permissions

---

## Deferred

The following functionality is intentionally excluded.

Pomodoro

- work timer
- break timer
- pause
- resume

AI

- reminder generation
- AI scheduling
- smart reminders

Planner

- planner reminders
- planner integration

Updater

- update reminders

Electron Removal

- deferred to Phase 12

---

## Validation

After every completed task:

- npm install (only if dependencies changed)
- npm run typecheck
- npm test
- npm run build
- cargo fmt
- cargo test
- cargo build
- Tauri permission validation
- Electron production build
- Electron smoke launch
- Tauri development smoke launch

Fix every failure before continuing.

---

## Manual Verification

Verify:

- reminders load
- reminders save
- reminder edits save
- reminder deletes persist
- reminders survive restart
- React reminder widgets appear
- reminder widget title and message match Electron
- Dismiss works
- Snooze creates the existing five-minute one-time reminder
- configured reminder sound plays once
- recurring reminders trigger
- one-time reminders trigger
- New Reminder context-menu action opens the existing Companion panel
- Manage Reminders context-menu action opens the existing Companion panel
- Preferences remains unchanged
- no renderer console errors
- no permission warnings

---

## Progress Tracking

Update docs/migrating/progress.md with:

- completed task
- implementation summary
- files changed
- validation performed
- manual verification
- blockers
- next task

---

## Phase Exit Criteria

Phase 7 is complete only if:

✓ Native reminder engine implemented

✓ Scheduler matches Electron

✓ Reminder persistence matches Electron

✓ Startup restoration works

✓ Reminder CRUD works

✓ Existing `reminders:fired` event contract works

✓ React reminder widget, Dismiss, Snooze, and notification sound match
Electron

✓ Existing Companion reminder panels and context-menu actions work

✓ Reminder schema is unchanged

✓ No native OS notification capability was added

✓ DesktopBridge used for renderer requests only

✓ Native scheduling remains inside Rust

✓ Least-privilege permissions implemented

✓ Electron implementation preserved

✓ All automated validation passes

✓ Manual verification passes

✓ Repository builds successfully

Only then may Phase 8 begin.

---

# PHASE 8

Pomodoro Migration

Goal

Migrate the existing Pomodoro focus timer from Electron to Tauri v2 while
preserving timer accuracy, duration selection, pause/resume/stop semantics,
completion behavior, DesktopBridge architecture, and renderer behavior.

Electron remains the reference implementation.

DesktopBridge remains the only renderer abstraction.

Timer execution belongs entirely to Rust.

The renderer remains runtime agnostic.

---

## Discovery

Before implementation:

- Inspect the existing `PomodoroManager`.
- Inspect timer lifecycle.
- Inspect preset and custom duration selection.
- Inspect pause/resume behavior.
- Inspect stop behavior.
- Inspect completion flow.
- Inspect notification flow.
- Inspect startup restoration.
- Inspect persistence.
- Inspect IPC.
- Inspect DesktopBridge contracts.
- Inspect renderer hooks.
- Inspect Preferences integration.
- Inspect tray/context-menu interactions.
- Document findings inside `docs/migrating/progress.md`.

Do not modify production code until discovery is complete.

---

## Scope

This phase includes:

- native Pomodoro engine
- timer scheduler
- timer persistence
- startup restoration
- focus sessions
- preset and custom durations
- pause
- resume
- stop
- timer completion
- DesktopBridge
- runtime events
- Companion integration
- existing notification widget
- existing sounds
- existing Pomodoro context-menu actions
- least-privilege permissions

This phase does NOT include:

- AI
- updater
- release pipeline
- Electron removal
- planner
- reminder redesign

---

## Parity Rule

This phase follows strict Electron parity.

If `migration_tasks.md` conflicts with either
`migration_codex.md` or the existing Electron implementation,
the Electron implementation is authoritative unless an explicit redesign has
been approved beforehand.

Do not redesign Pomodoro behavior. In particular, do not add short-break,
long-break, reset, skip, auto-start, session-cycle, or Preferences behavior
that does not exist in Electron.

---

## Architecture Rules

Timer execution belongs to Rust.

Renderer

↓

DesktopBridge

↓

Runtime Adapter

↓

Rust

The renderer must never own production timer execution.

Do not use browser timers as the production timer.

DesktopBridge remains the only renderer abstraction.

Do not expose Tauri APIs directly to React.

---

## Task 8.1

Audit Existing Pomodoro System

Objective

Document the existing Electron implementation.

Acceptance

- timer lifecycle documented
- session transitions documented
- persistence documented
- DesktopBridge documented
- startup restoration documented
- renderer integration documented

Commit

No commit.

---

## Task 8.2

Create Native Pomodoro Engine

Objective

Implement the native Rust Pomodoro engine.

Requirements

- singleton
- runtime owned
- survives restart
- deterministic timer state
- renderer independent

Acceptance

- engine starts
- engine stops
- state restored
- Electron unchanged

Commit

feat(tauri): migrate pomodoro engine

---

## Task 8.3

Pomodoro Persistence

Objective

Implement the existing separate native `pomodoro.json` store.

Requirements

- preserve Electron schema
- preserve the separate file name and document version
- atomic persistence
- startup restoration
- owner-only file permissions where supported
- preserve missing, invalid, and failed-load behavior

Acceptance

- state persists
- restart restores state
- Electron parity maintained

Commit

feat(tauri): migrate pomodoro persistence

---

## Task 8.4

DesktopBridge Commands

Objective

Expose Pomodoro commands through DesktopBridge.

Includes

- start
- custom-duration panel closed
- state snapshot/event activation required for startup and reload recovery

Requirements

- typed commands
- DesktopBridge only
- renderer unchanged
- pause, resume, and stop remain Rust-owned native menu actions
- no additional renderer command surface

Acceptance

Renderer controls Pomodoro exclusively through DesktopBridge.

Commit

feat(tauri): migrate pomodoro commands

---

## Task 8.5

Runtime Events

Objective

Implement existing Pomodoro runtime events.

Requirements

- preserve Electron event contract
- preserve only `pomodoro:custom-duration-requested`,
  `pomodoro:state-changed`, and `pomodoro:completed`
- exact window targeting
- DesktopBridge abstraction
- no renderer polling
- preserve startup/reload buffering for state, completion, and custom-duration
  requests

Acceptance

Renderer receives timer updates exactly as Electron.

Commit

feat(tauri): migrate pomodoro events

---

## Task 8.6

Companion Integration

Objective

Reconnect the existing Companion UI.

Requirements

- preserve UI
- preserve UX
- preserve the existing custom-duration panel
- preserve the existing running/paused timer widget
- preserve the existing native context-menu presets, checked state,
  Pause/Resume enabled state, Stop enabled state, and Custom… entry point
- keep all native context-menu callbacks inside Rust

Acceptance

Companion behaves identically to Electron.

Commit

feat(tauri): migrate pomodoro companion

---

## Task 8.7

Timer Completion

Objective

Preserve completion behavior.

Requirements

- existing React completion widget
- existing sounds
- existing transitions
- existing completion flow

Do not introduce:

- native OS notifications
- notification plugins
- notification permissions

Acceptance

Completion behavior matches Electron exactly.

Commit

feat(tauri): migrate pomodoro completion

---

## Task 8.8

Permissions

Objective

Grant least-privilege permissions.

Requirements

- Pomodoro commands only
- exact window labels
- no wildcard capabilities
- no unnecessary plugins

Acceptance

Permission validation passes.

Commit

feat(tauri): migrate pomodoro permissions

---

## Deferred

The following functionality is intentionally excluded.

AI

- AI coaching
- AI productivity suggestions
- AI session planning

Updater

- update reminders
- release prompts

Planner

- planner integration

Electron Removal

- deferred to Phase 12

---

## Validation

After every completed task:

- npm install (only if dependencies changed)
- npm run typecheck
- npm test
- npm run build
- cargo fmt
- cargo test
- cargo build
- Tauri permission validation
- Electron production build
- Electron smoke launch
- Tauri development smoke launch

Fix every failure before continuing.

---

## Manual Verification

Verify:

- 25, 50, and 90 minute focus sessions start
- custom focus duration starts
- starting a duration replaces the current session exactly as Electron
- selected-duration radio state matches Electron
- pause works
- resume works
- stop works
- timer survives restart
- startup restoration works
- elapsed time is materialized correctly after restart
- expired restored sessions complete exactly once
- completion widget appears
- sounds play
- Companion controls work
- DesktopBridge works
- no renderer console errors
- no permission warnings

---

## Progress Tracking

Update `docs/migrating/progress.md` after every milestone.

Include:

- completed task
- implementation summary
- files changed
- validation performed
- manual verification
- blockers
- next task

---

## Phase Exit Criteria

Phase 8 is complete only if:

✓ Native Pomodoro engine implemented

✓ Timer execution owned by Rust

✓ Startup restoration works

✓ Persistence matches Electron

✓ Pause/Resume matches Electron

✓ Stop matches Electron

✓ Preset/custom duration behavior matches Electron

✓ Native Pomodoro context-menu behavior matches Electron

✓ Companion integration preserved

✓ DesktopBridge fully migrated

✓ Existing completion widget preserved

✓ Existing sounds preserved

✓ Least-privilege permissions implemented

✓ Electron implementation preserved

✓ All automated validation passes

✓ Manual verification passes

✓ Repository builds successfully

Only then may Phase 9 begin.

---

# PHASE 9

AI System Migration

Goal

Migrate the complete AI System from Electron to Tauri v2 while preserving
existing AI behavior, provider architecture, final-response semantics,
DesktopBridge abstraction, and renderer behavior.

Electron remains the reference implementation.

DesktopBridge remains the only renderer abstraction.

AI provider execution belongs entirely to Rust.

The renderer remains runtime agnostic.

---

## Discovery

Before implementation:

- Inspect the existing AI provider architecture.
- Inspect AIProvider interfaces.
- Inspect provider registry.
- Inspect provider selection.
- Inspect final-response transport and the intentionally unsupported provider
  streaming methods.
- Inspect conversation lifecycle.
- Inspect lifecycle cancellation.
- Inspect diagnostics.
- Inspect rate limiting.
- Inspect token accounting.
- Inspect secret storage integration.
- Inspect DesktopBridge contracts.
- Inspect renderer hooks.
- Inspect AI actions.
- Inspect Preferences integration.
- Document findings inside `docs/migrating/progress.md`.

Do not modify production code until discovery is complete.

---

## Scope

This phase includes:

- native AI runtime
- provider registry
- OpenAI
- Gemini
- Claude
- Grok
- Ollama
- Custom Provider
- whole-response provider execution
- lifecycle cancellation
- AI actions
- diagnostics
- rate limiting
- DesktopBridge
- startup restoration
- provider persistence
- least-privilege permissions

This phase does NOT include:

- updater
- release pipeline
- Electron removal
- planner redesign
- new AI capabilities other than the explicitly approved Claude provider

---

## Parity Rule

This phase follows strict Electron parity.

If `migration_tasks.md` conflicts with either
`migration_codex.md` or the existing Electron implementation,
the Electron implementation is authoritative unless an explicit redesign has
been approved beforehand.

Do not redesign the AI system.

Electron exchanges exactly one final AI response with the renderer. Phase 9
must not add incremental renderer streaming, streaming IPC, streaming events,
or DesktopBridge streaming APIs.

Providers may consume a streaming upstream API internally, but Rust must
aggregate the complete response before crossing DesktopBridge.

Electron exposes no renderer cancellation command. Preserve only automatic
lifecycle cancellation on provider changes, renderer shutdown/reload,
navigation, and application shutdown.

Preserve one active request per renderer role. Do not add concurrent same-role
request scheduling.

Preserve existing diagnostics: connection testing and sanitized provider
errors. Do not add provider-health or latency-reporting features.

---

## Architecture Rules

AI execution belongs entirely to Rust.

Renderer

↓

DesktopBridge

↓

Runtime Adapter

↓

Rust

DesktopBridge remains the only renderer abstraction.

Do not expose Tauri APIs directly to React.

Do not expose provider SDKs directly to renderer code.

---

## Task 9.1

Audit Existing AI System

Objective

Document the complete Electron AI implementation.

Acceptance

- provider architecture documented
- final-response transport and unsupported streaming documented
- lifecycle cancellation documented
- one-request-per-renderer-role behavior documented
- existing connection diagnostics documented
- registry documented
- DesktopBridge documented
- secret integration documented
- renderer integration documented

Commit

No commit.

---

## Task 9.2

Create Native AI Runtime

Objective

Implement the native Rust AI runtime.

Requirements

- singleton
- runtime owned
- provider independent
- whole-response request execution
- renderer independent

Acceptance

- runtime initializes
- runtime shuts down cleanly
- Electron unchanged

Commit

feat(tauri): migrate ai runtime

---

## Task 9.3

Provider Registry

Objective

Implement the native provider registry.

Requirements

- provider registration
- provider lookup
- provider selection
- runtime switching
- preserve existing architecture

Acceptance

All providers register through one runtime.

Commit

feat(tauri): migrate provider registry

---

## Task 9.4

Secret Store Integration

Objective

Integrate the existing native credential storage.

Requirements

- reuse Phase 6 secret store
- no plaintext persistence
- runtime loading
- provider isolation

Acceptance

All providers obtain credentials through native storage.

Commit

feat(tauri): migrate ai secrets

---

## Task 9.5

OpenAI Provider

Objective

Migrate the existing OpenAI provider.

Requirements

- one final response per request
- lifecycle cancellation
- usage reporting
- model selection
- DesktopBridge integration

Acceptance

OpenAI behaves exactly as Electron.

Commit

feat(tauri): migrate openai provider

---

## Task 9.6

Gemini Provider

Objective

Migrate the existing Gemini provider.

Requirements

- preserve Electron behavior
- one final response per request
- lifecycle cancellation
- usage reporting

Acceptance

Gemini behaves exactly as Electron.

Commit

feat(tauri): migrate gemini provider

---

## Task 9.7

Claude Provider

Objective

Add Claude provider support using the existing AI provider architecture.

Requirements

- Anthropic API compatibility
- provider registration
- one final response per request
- lifecycle cancellation
- usage reporting
- secret store integration
- DesktopBridge integration
- no provider-specific renderer UI
- any Anthropic streaming terminates inside Rust and is aggregated before the
  DesktopBridge response

Acceptance

- Claude provider selectable
- Claude returns one complete response through the existing bridge contract
- lifecycle cancellation works
- provider switching works
- architecture consistent with all other providers

Commit

feat(tauri): migrate claude provider

---

## Task 9.8

Grok Provider

Objective

Migrate the Grok provider.

Requirements

- preserve Electron behavior
- one final response per request
- lifecycle cancellation
- usage reporting

Acceptance

Grok behaves exactly as Electron.

Commit

feat(tauri): migrate grok provider

---

## Task 9.9

Ollama Provider

Objective

Migrate the Ollama provider.

Requirements

- local endpoint support
- one final response per request
- lifecycle cancellation
- DesktopBridge integration

Acceptance

Ollama behaves exactly as Electron.

Commit

feat(tauri): migrate ollama provider

---

## Task 9.10

Custom Provider

Objective

Migrate the custom provider implementation.

Requirements

- existing endpoint configuration
- authentication
- one final response per request
- lifecycle cancellation
- DesktopBridge integration

Acceptance

Custom provider behaves exactly as Electron.

Commit

feat(tauri): migrate custom provider

---

## Task 9.11

Final Response Transport

Objective

Preserve the existing whole-response renderer contract.

Requirements

- exactly one final response per request
- provider-internal streaming may be aggregated only inside Rust
- no incremental renderer tokens
- no streaming IPC or runtime events
- no DesktopBridge streaming API
- error propagation
- provider independence

Acceptance

The renderer receives the same final `AIAskResult` semantics as Electron.

Commit

feat(tauri): migrate ai response transport

---

## Task 9.12

AI Actions

Objective

Migrate existing AI actions.

Requirements

- preserve existing action architecture
- provider independent
- runtime owned

Acceptance

AI actions behave exactly as Electron.

Commit

feat(tauri): migrate ai actions

---

## Task 9.13

Diagnostics

Objective

Implement runtime diagnostics.

Requirements

- provider connection testing
- sanitized provider errors
- existing safe provider-specific diagnostic detail
- existing usage metadata
- no provider-health reporting
- no latency reporting

Acceptance

Diagnostics match Electron.

Commit

feat(tauri): migrate ai diagnostics

---

## Task 9.14

Rate Limiting & Cancellation

Objective

Preserve provider execution controls.

Requirements

- lifecycle cancellation
- request isolation
- one active request per renderer role
- timeout handling
- existing rate limiting
- no explicit renderer cancellation command
- automatic cancellation on provider changes, renderer shutdown/reload,
  navigation, and application shutdown

Acceptance

Execution control matches Electron.

Commit

feat(tauri): migrate ai execution controls

---

## Task 9.15

Permissions

Objective

Grant least-privilege permissions.

Requirements

- provider commands only
- exact window labels
- no wildcard permissions
- no unnecessary plugins

Acceptance

Permission validation passes.

Commit

feat(tauri): migrate ai permissions

---

## Deferred

The following functionality is intentionally excluded.

- MCP
- Agent framework redesign
- Autonomous agents
- RAG
- Vector databases
- Multi-agent orchestration
- Planner redesign
- Updater
- Electron removal

---

## Validation

After every completed task:

- npm install (only if dependencies changed)
- npm run typecheck
- npm test
- npm run build
- cargo fmt
- cargo test
- cargo build
- Tauri permission validation
- Electron production build
- Electron smoke launch
- Tauri development smoke launch

Fix every failure before continuing.

---

## Manual Verification

Verify:

- OpenAI works
- Gemini works
- Claude works
- Grok works
- Ollama works
- Custom provider works
- provider switching works
- each provider returns one final response
- no renderer streaming API or IPC exists
- lifecycle cancellation works
- connection tests and sanitized errors work
- AI actions work
- secret loading works
- DesktopBridge works
- no renderer console errors
- no permission warnings

---

## Progress Tracking

Update `docs/migrating/progress.md` after every milestone.

Include:

- completed task
- implementation summary
- files changed
- validation performed
- manual verification
- blockers
- next task

---

## Phase Exit Criteria

Phase 9 is complete only if:

✓ Native AI runtime implemented

✓ Provider registry implemented

✓ OpenAI provider functional

✓ Gemini provider functional

✓ Claude provider functional

✓ Grok provider functional

✓ Ollama provider functional

✓ Custom provider functional

✓ Final-response transport preserved

✓ AI actions preserved

✓ Secret store integration complete

✓ Lifecycle cancellation preserved

✓ Diagnostics preserved

✓ DesktopBridge fully migrated

✓ Least-privilege permissions implemented

✓ Electron implementation preserved

✓ All automated validation passes

✓ Manual verification passes

✓ Repository builds successfully

Only then may Phase 10 begin.

---

# PHASE 10

Updater Migration

Goal

Migrate the existing Electron updater runtime contract to the Tauri
architecture without expanding the product's update experience.

The renderer-visible Electron contract is authoritative:

- obtain the current update status;
- manually check for updates;
- observe `updates:status-changed`;
- persist `updates.automatic`; and
- perform one automatic check during startup when that setting is enabled.

The one-time manual Electron → Tauri migration dialog is the only approved
functional expansion in this phase.

Phase 10 establishes the runtime abstraction and application integration.
Phase 11 supplies the production signing identity, signed artifacts, hosted
feed, CI/CD, notarization, and live production updater verification.

---

## Discovery

Before implementation:

- inspect the existing Electron update flow
- inspect update preferences
- inspect update settings persistence
- inspect update menus
- inspect update notifications
- inspect startup update checks
- inspect manual update flow
- inspect the exact renderer/DesktopBridge surface
- inspect release packaging
- document findings inside `docs/migrating/progress.md`

Do not modify production code until discovery is complete.

---

## Scope

This phase includes:

- native updater runtime abstraction
- a Tauri updater adapter boundary
- the existing update status model
- the existing update status event
- the existing DesktopBridge update methods
- startup update checks when `updates.automatic` is enabled
- manual update checks
- update-setting persistence
- the approved manual Electron → Tauri migration dialog
- least-privilege permissions

This phase does NOT include:

- renderer download, install, or restart-to-update controls
- updater-specific native menus
- updater-specific native or OS notifications
- automatic downloads
- automatic installation
- automatic framework replacement
- updater signing or signing-key generation
- the updater public key
- `latest.json`
- updater `.sig` generation
- GitHub release feed configuration or hosting
- production release metadata
- CI/CD or release automation
- code signing or notarization
- production updater verification
- Electron removal
- installer redesign

---

## Parity Rule

Follow strict Electron parity.

If migration documents conflict with Electron, Electron is authoritative unless
an explicit redesign has already been approved.

---

## Architecture Rules

Updater logic belongs to Rust.

Renderer

↓

DesktopBridge

↓

Rust updater

The renderer never calls Tauri updater APIs directly.

The Phase 10 runtime must be testable through a deterministic updater backend
without production release infrastructure. The production Tauri updater
provider is configured and verified in Phase 11.

No renderer-visible download, install, restart, menu, or notification contract
may be introduced in Phase 10.

---

## Task 10.1

Audit Existing Update System

Objective

Document the existing Electron updater.

Acceptance

- updater documented
- preferences documented
- menu behaviour documented
- notification behaviour documented

Commit

No commit.

---

## Task 10.2

Native Updater

Objective

Implement the native updater runtime abstraction and Tauri adapter boundary.

Requirements

- preserve the exact shared Electron update status DTO
- preserve Electron check coalescing and error sanitization
- preserve prerelease/downgrade behavior
- expose status snapshots and check operations to the command layer
- support a deterministic backend for tests
- fail safely when production updater configuration is absent
- do not download, install, or restart the application

Acceptance

The native runtime reproduces Electron's status/check behavior in automated
tests without depending on a production feed.

Commit

feat(tauri): migrate updater runtime

---

## Task 10.3

DesktopBridge

Objective

Expose updater functionality through DesktopBridge.

Requirements

- get current update status
- check for updates
- update status events
- preserve the existing Preferences-only authorization boundary
- preserve the existing renderer-facing method signatures
- no direct Tauri imports in renderer components

Acceptance

Renderer remains runtime agnostic and the DesktopBridge update surface does not
expand beyond Electron.

Commit

feat(tauri): migrate updater bridge

---

## Task 10.4

Settings Integration

Objective

Preserve update preferences.

Requirements

- preserve `updates.automatic`
- default remains `false`
- perform one startup check only when enabled
- enabling the setting triggers the same immediate check as Electron
- disabling the setting does not download or install anything
- persistence

Acceptance

Settings survive restart and startup/manual check behavior matches Electron.

Commit

feat(tauri): migrate updater settings

---

## Task 10.5

Update Status and Events

Objective

Preserve the existing update presentation contract.

Requirements

- preserve the existing `UpdateStatus` states and payloads
- preserve only `updates:status-changed`
- target only the Preferences window
- keep the existing Preferences status text
- do not add native notifications
- do not add updater menu items
- do not add download/install/restart controls

Acceptance

Preferences receives the initial snapshot and subsequent status changes exactly
as it does under Electron.

Commit

feat(tauri): migrate updater status

---

## Task 10.6

Electron → Tauri Migration

Objective

Implement the one-time migration path for existing Electron users.

Requirements

When the final Electron release detects that PsyDuck 2.0 (Tauri) is available:

- display a migration dialog
- explain that this is a one-time upgrade
- provide:
  - Download PsyDuck 2.0
  - Remind Me Later
- open the official release page when Download is selected
- do not attempt in-place framework replacement
- do not uninstall the Electron application
- do not add an automatic install path

Phase 10 implements and tests the dialog behavior. Phase 11 owns the production
release metadata and publication sequence that cause the final Electron release
to discover the approved Tauri transition release.

Acceptance

Existing Electron users can migrate safely.

Commit

feat(updater): add tauri migration flow

---

## Task 10.7

Permissions

Objective

Grant least-privilege updater permissions.

Requirements

- exact Preferences-window status/check permissions only
- no companion updater authority
- no renderer download/install/restart permission
- no wildcard permissions
- deny unused updater/plugin commands

Acceptance

Permission validation passes and the renderer cannot exceed the Electron update
surface.

Commit

feat(tauri): migrate updater permissions

---

## Validation

Run:

- npm run typecheck
- npm test
- npm run build
- cargo fmt
- cargo test
- cargo build
- Tauri permission validation
- Electron production build
- Tauri production build
- deterministic updater runtime tests
- DesktopBridge contract tests
- migration-dialog tests

Production signing, feed, artifact, installation, and update verification belong
to Phase 11 and are not Phase 10 validation gates.

---

## Manual Verification

Verify:

- startup update check
- manual update check
- update status text
- update status events
- update persistence
- migration dialog
- migration link
- DesktopBridge
- no renderer errors
- no permission warnings

Do not claim live production update detection, download, installation, or
restart verification during Phase 10. Those checks require Phase 11 release
infrastructure.

---

## Progress Tracking

Update `docs/migrating/progress.md` after every milestone.

Document:

- completed task
- implementation
- validation
- manual verification
- blockers
- next task

---

## Phase Exit Criteria

Phase 10 is complete only if:

✓ Native updater implemented

✓ DesktopBridge migrated

✓ Update settings preserved

✓ Update status and events preserved

✓ Electron → Tauri migration implemented

✓ Least-privilege permissions implemented

✓ Automated validation passes

✓ Manual verification passes

✓ Repository builds successfully

Phase 10 completion does not mean the production updater feed is releasable.
Phase 11 must implement updater signing and distribution infrastructure.
Production credential provisioning and live distribution remain Release
Candidate Checklist operations after repository migration.

Only then may Phase 11 begin.

---

# PHASE 11

Release Pipeline

Goal

Implement the repository-owned release pipeline that can produce, verify, and
atomically publish the migrated Tauri updater and cross-platform application
when externally managed production credentials and release approval are
provided.

Phase 11 owns release infrastructure and automation intentionally excluded
from Phase 10. Executing that infrastructure with production credentials,
publishing a real release, and completing live staged verification are
external release operations owned by the Release Candidate Checklist after
Phase 12.

---

## Scope

Phase 11 includes:

- configuration for a stable Tauri updater signing identity
- a committed-path contract for the externally supplied updater public key
- CI-only inputs for the private signing key and password
- `bundle.createUpdaterArtifacts`
- signed updater bundles and `.sig` files
- `latest.json` generation
- GitHub release feed configuration and hosting
- cross-platform Tauri artifact generation
- release asset naming and collision prevention
- CI/CD and release automation
- macOS signing and notarization automation
- Windows signing automation where configured
- production updater endpoint configuration
- production updater verification tooling
- final Electron transition-release metadata generation
- preservation of legacy Electron feed assets
- website download selection at cutover

Phase 11 does NOT introduce new renderer updater controls, menus,
notifications, automatic installation, or any other product behavior excluded
from Phase 10.

---

## Required Work

1. Configure the Tauri updater public-key path and production endpoints.
2. Reference private signing material only through approved GitHub Actions
   secrets.
3. Generate platform-specific Tauri bundles, updater archives, signatures, and
   `latest.json`.
4. Preserve the existing atomic draft/verify/publish release architecture.
5. Prevent Electron and Tauri assets from colliding or being selected
   ambiguously.
6. Keep legacy Electron update metadata available for existing installs.
7. Generate and preserve the final Electron transition metadata required to
   trigger the approved one-time PsyDuck 2.0 migration dialog.
8. Add macOS notarization and configured platform-signing automation.
9. Update release verification for every expected package, signature,
   checksum, URL, version, platform, and architecture.
10. Provide deterministic verification commands for the signed staged release
    path that the Release Candidate Checklist will execute before publication.

---

## Secret Material

Production secrets must never be generated, committed, logged, or embedded.

If production signing identities, Apple credentials, Windows certificates,
or GitHub Actions secrets are unavailable, implement the complete release
pipeline around externally provided secrets.

Use placeholders only where required by documentation.

Validate only the presence and wiring of secrets.

Never fabricate or commit secret values.

Production credential availability is not a Phase 11 or Phase 12 completion
gate. Missing credentials must fail release jobs closed without exposing
values, and the exact required external inputs must be documented for the
Release Candidate Checklist.

## Validation

Run all repository validation plus:

- tag/package/Cargo/Tauri version consistency
- Tauri updater configuration validation
- missing-signing-secret preflight behavior without logging secret values
- release artifact and `.sig` verifier tests using deterministic fixtures
- `latest.json` schema and URL verifier tests
- cross-platform packaging configuration validation
- signing/notarization workflow validation
- legacy Electron feed compatibility tests
- atomic draft/verify/publish workflow validation without publishing a
  production release

Live signed artifacts, staged updater checks, installation/restart checks,
transition-release discovery, production publication, and production
credential verification are Release Candidate Checklist operations.

---

## Phase Exit Criteria

Phase 11 is complete only if:

✓ Stable updater-signing configuration and external input contract implemented

✓ Updater public-key path and validation implemented

✓ Private signing material is accepted only through CI secrets

✓ Tauri updater artifact and signature generation is automated

✓ `latest.json` generation and hosting contract is automated

✓ GitHub release feed configured

✓ CI/CD builds every supported platform

✓ Required signing and notarization steps are implemented

✓ Production updater verification tooling is implemented

✓ Signed download, installation, and restart verification commands are
documented and automated where repository fixtures permit

✓ Final Electron transition metadata and verification tooling are implemented

✓ Legacy Electron feed remains supported

✓ Release workflow preserves atomic draft/verify/publish behavior

✓ Website selects the intended Tauri installers

✓ Missing production credentials fail release jobs closed without leaking
values

Actual credential provisioning, signed staging, transition-release
publication, updater installation/restart verification, and go-live approval
remain mandatory Release Candidate Checklist operations, not migration-phase
exit criteria.

Only then may Phase 11.5 begin.

---

# PHASE 11.5

## Functional Parity Closure

Phase 11.5 exists because the Phase 12 pre-removal audit found working
Electron behavior that was deferred by earlier phase contracts but never
assigned to a later executable migration phase.

These tasks are feature-domain migration work. They are not Electron-removal
cleanup and must be complete before any Electron implementation, dependency,
test, or compatibility path is deleted.

The migration-wide parity rule remains authoritative:

> If a migration document conflicts with the existing Electron
> implementation, the existing Electron implementation defines the required
> behavior unless an explicit redesign was approved before implementation.

Phase 11.5 must not redesign the application or add product functionality.

---

## Goal

Close every remaining functional gap between the working Electron application
and the migrated Tauri application so Phase 12 can remove Electron without
removing user-visible behavior.

Electron remains intact and releasable throughout this phase.

---

## Scope

Phase 11.5 includes:

- complete Set My Name behavior in Tauri
- complete Sticky Message behavior in Tauri
- a Rust Daily Planner backend with Electron-equivalent output
- hydration settings parity
- complete dynamic companion context-menu parity
- final disposition and verification requirements for the one-time
  Electron-to-Tauri migration dialog
- Tauri-only composition of the existing companion DesktopBridge contract
  after all of its domains are available
- exact event recovery, authorization, permissions, persistence, and tests
  required by those existing behaviors
- discovery and migration of any additional Electron-only behavior found
  while implementing or manually verifying this phase

Phase 11.5 does NOT include:

- Electron removal
- Electron dependency or build-script removal
- release-pipeline redesign
- renderer redesign
- new settings
- new menus or menu actions
- new planner capabilities
- new hydration behavior
- new notification behavior
- new migration or updater controls
- work beyond closing observable Electron parity gaps

---

## Architecture Rules

- Existing Electron behavior is the specification.
- DesktopBridge remains the only renderer abstraction.
- React renderers must remain runtime agnostic.
- Renderer components must not import Tauri APIs or perform runtime
  detection.
- Native menu callbacks, persistence, and privileged planner operations
  remain in Rust.
- Existing role-scoped commands, targeted events, recovery semantics, and
  least-privilege capabilities must be extended rather than bypassed.
- Electron implementations remain unchanged except where a parity regression
  fix is required and separately justified.
- No Electron code may be removed during Phase 11.5.

---

## Task 11.5.1 — Re-audit Remaining Electron-only Behavior

Before writing production code:

- compare the complete Electron and Tauri companion context menus
- compare every associated menu action and checked/enabled state
- inspect Set My Name panel requests, mutations, persistence, and events
- inspect Sticky Message set/clear requests, mutations, persistence, and
  events
- inspect Daily Planner source data, greeting rules, ordering, and renderer
  contract
- inspect hydration Preferences, runtime settings, persistence, and renderer
  timer integration
- inspect the complete `CompanionBridge` and identify every method still
  unavailable in Tauri
- inspect the final Electron-to-Tauri migration-dialog release obligation
- search for any other renderer-accessible Electron path without a working
  Tauri equivalent
- record findings and an exact parity matrix in
  `docs/migrating/progress.md`

Acceptance:

- every remaining Electron-only behavior has an owner in a Phase 11.5 task
- no production code changes were made before discovery completed
- no aspirational or undocumented feature is added to the parity scope

---

## Task 11.5.2 — Set My Name and Sticky Message Parity

Implement the existing Personal Assistant behavior through the migrated
settings runtime.

Requirements:

- preserve the current React panels and copy
- preserve Electron validation and persistence behavior
- emit only the existing targeted panel-request events
- preserve Sticky Message set and clear behavior
- expose the existing operations through DesktopBridge
- keep Electron behavior unchanged
- add only exact companion permissions

Acceptance:

- Set My Name opens, saves, updates runtime state, and survives restart
- Set Sticky Message opens, saves, displays, clears, and survives restart
- event recovery works after companion reload
- no renderer Tauri import or runtime detection is introduced

---

## Task 11.5.3 — Daily Planner Rust Backend

Port the existing `DailyPlannerService` behavior to Rust.

Requirements:

- preserve the existing `DailyPlannerBriefing` renderer contract
- preserve greeting periods and normalized user-name behavior
- use the migrated reminder store as the only reminder source
- preserve same-local-day filtering, completed/past-reminder exclusion,
  schedule selection, chronological ordering, and ID tie-breaking
- preserve the existing targeted panel-request event
- expose one authorized companion command through DesktopBridge
- do not redesign the planner UI or add planner capabilities

Acceptance:

- Daily Planner opens from the native context menu
- briefing content matches Electron for equivalent clocks, names, and
  reminders
- empty, invalid, completed, past, recurring, and tied reminders have
  Electron-equivalent results
- Rust tests use deterministic clocks

---

## Task 11.5.4 — Hydration Settings Parity

Connect the existing hydration Preferences and renderer-owned reminder timer
to the migrated settings backend.

Requirements:

- keep `WaterReminder` and its timing behavior in the renderer, matching the
  architecture plan
- preserve the existing `water.enabled` and `water.interval` schema
- preserve supported intervals, defaults, validation, persistence, and
  runtime-settings broadcasts
- enable the existing Preferences controls in Tauri
- preserve disabled and interval-change behavior without duplicate timers
- do not add native hydration scheduling or native notifications
- add only the exact Preferences mutation authority already represented by
  the existing settings patch contract

Acceptance:

- hydration enable/disable and interval changes save through DesktopBridge
- changes apply immediately and survive restart
- the renderer timer remains single-instance and follows the saved snapshot
- Electron behavior and schema remain unchanged

---

## Task 11.5.5 — Dynamic Companion Context Menu Parity

Complete the Tauri companion context menu using the Electron menu as the
authoritative hierarchy and behavior.

Requirements:

- preserve ordering, labels, separators, submenus, checkboxes, radio items,
  enabled states, and dynamic state snapshots
- preserve Set My Name, Reminders, Daily Planner, Sticky Message, Water
  Reminders, Eye Tracking, Always On Top, Pomodoro, Preferences, About,
  Restart, and Quit behavior
- use Rust-owned native callbacks for native actions
- use existing targeted renderer events only for existing renderer panels
- route renderer-originated context-menu requests through DesktopBridge
- do not add menu items or actions absent from Electron

Acceptance:

- the complete menu structure and state match Electron
- every action works and updates its checked/enabled state on the next open
- companion and Preferences role authorization remains least privilege
- no renderer console or permission warning appears

---

## Task 11.5.6 — Migration Dialog Final Disposition

Resolve the lifecycle of the approved one-time Electron-to-Tauri migration
dialog without inventing a Tauri-side replacement.

The dialog is an Electron transition mechanism, not ongoing Tauri product
functionality. Its authoritative implementation remains in the final
Electron transition release.

Requirements:

- verify the final Electron transition build contains the existing dialog,
  copy, Download action, Remind Me Later behavior, and official release URL
- preserve the published legacy Electron feed and transition assets required
  by installed Electron clients
- document the release-manager evidence required before public go-live
- do not port the dialog into Tauri unless a separate explicit redesign is
  approved
- define Phase 12 deletion as repository source cleanup independent of the
  later external release operation, without deleting already published legacy
  assets

Acceptance:

- the transition obligation has an explicit owner and verifiable completion
  record
- Phase 12 can delete Electron-only dialog source without claiming either that
  the Tauri application exposes an inapplicable legacy prompt or that the
  external transition release has already been published
- no automatic framework replacement, uninstallation, or installer chaining
  is introduced

---

## Task 11.5.7 — DesktopBridge Parity and Least Privilege

Compose the existing Tauri domain adapters into the complete renderer-facing
companion bridge only after every required method is implemented.

Requirements:

- preserve the renderer-facing API
- remove no Electron adapter or preload path
- expose no generic invoke, event, filesystem, HTTP, shell, or process access
- authorize exact commands by exact window label
- preserve targeted event routing and reload recovery
- keep Preferences and companion capabilities separated

Acceptance:

- `getCompanionBridge()` no longer returns `undefined` in Tauri
- every `CompanionBridge` member has a working, tested Tauri implementation
- narrow domain bridge consumers continue to work
- authorization tests prove cross-role calls are denied

---

## Task 11.5.8 — Final Parity Audit and Manual Gate

Repeat the complete Electron-versus-Tauri behavior audit before Phase 12.

Requirements:

- search every Electron IPC channel, preload method, renderer-accessible main
  service, native menu action, and Preferences control
- migrate any additional Electron-only behavior discovered by the audit using
  the same parity and least-privilege rules
- update the Phase 11.5 contract before implementing newly discovered scope
  if ownership or architecture is ambiguous
- preserve Electron until every parity item passes

Manual verification:

- Set My Name
- Sticky Message set, display, and clear
- Daily Planner
- hydration enable, disable, interval, firing, and restart persistence
- complete companion context-menu structure, state, and actions
- Preferences and companion event recovery after reload
- all previously completed Tauri features
- final Electron transition-dialog obligation
- no renderer errors or permission warnings

Acceptance:

- the parity matrix contains no remaining Electron-only user behavior
- automated and manual parity verification passes
- `docs/migrating/progress.md` contains the evidence
- repository remains buildable and Electron remains intact

---

## Validation

After every implementation milestone run:

- npm install when dependencies change
- npm run typecheck
- npm test
- npm run build
- cargo fmt
- cargo test
- cargo build
- Tauri permission validation
- Electron production build and smoke launch
- Tauri development and production smoke launches
- platform-specific verification where available

Fix every regression before continuing.

---

## Phase Exit Criteria

Phase 11.5 is complete only if:

✓ Set My Name has Tauri parity

✓ Sticky Message has Tauri parity

✓ Daily Planner has a tested Rust backend and Tauri bridge

✓ Hydration Preferences and runtime behavior have Tauri parity

✓ The complete dynamic companion context menu matches Electron

✓ Every existing companion panel event and operation works in Tauri

✓ `getCompanionBridge()` exposes a complete Tauri implementation

✓ Migration-dialog disposition and transition evidence are documented

✓ No additional renderer-accessible Electron-only behavior remains

✓ Least-privilege permissions and cross-role denial tests pass

✓ Electron remains fully functional and releasable

✓ Automated and manual validation passes

✓ `docs/migrating/progress.md` contains the completion evidence

Only then may Phase 12 begin.

---

# PHASE 12

## Electron Removal

Phase 12 begins only after:

- Phase 0–11.5 are complete
- Phase 11.5 functional parity has been manually verified
- Phase 11 release infrastructure is fully implemented and validated without
  requiring production credentials or publication

The objective of this phase is to permanently remove Electron from the repository while preserving all application functionality through the completed Tauri implementation.

---

## Scope

Phase 12 includes:

- Remove Electron Builder
- Remove Electron runtime dependencies
- Remove Electron development dependencies
- Remove Electron preload scripts
- Remove Electron main process
- Remove Electron IPC implementation
- Remove Electron Builder configuration
- Remove Electron packaging configuration
- Remove Electron-specific scripts
- Remove Electron-specific assets
- Remove Electron-only tests
- Remove Electron-only documentation
- Remove Electron compatibility shims
- Remove Electron feature flags
- Remove obsolete migration compatibility code
- Simplify DesktopBridge to a Tauri-only implementation
- Remove unused Electron permissions
- Remove dead code created during migration
- Update documentation to reference Tauri only
- Final repository cleanup

This phase must not introduce new application functionality.

It is a cleanup and consolidation phase only.

---

## Required Work

1. Remove every Electron runtime dependency from the repository.

2. Remove Electron Builder and all associated configuration.

3. Remove Electron preload scripts.

4. Remove the Electron main process.

5. Remove Electron IPC handlers and channels.

6. Remove Electron-specific DesktopBridge implementations while preserving the renderer API.

7. Simplify DesktopBridge to use only the Tauri backend.

8. Remove all Electron feature flags and compatibility branches.

9. Remove obsolete migration scaffolding that is no longer required.

10. Remove Electron packaging assets and build scripts.

11. Remove Electron-specific tests and fixtures.

12. Remove Electron-specific documentation.

13. Update repository documentation to describe Tauri as the only supported desktop runtime.

14. Verify that no Electron references remain in production code.

15. Verify that the release pipeline produces only Tauri artifacts.

16. Ensure the website download flow points exclusively to Tauri installers.

17. Perform final repository cleanup.

---

## Validation

Run the complete repository validation suite:

- npm install
- npm run typecheck
- npm test
- npm run build
- cargo fmt
- cargo test
- cargo build
- Tauri permission validation
- Tauri development build
- Tauri production build
- macOS packaged build
- Windows packaged build (where available)
- Linux packaged build (where available)

Additionally verify:

- package.json contains no Electron packages
- package-lock.json contains no Electron packages
- npm install does not install Electron
- no Electron imports remain
- no Electron runtime code remains
- no Electron Builder configuration remains
- no Electron preload scripts remain
- no Electron IPC remains
- no Electron assets remain
- DesktopBridge is Tauri-only
- release pipeline produces only Tauri artifacts
- website download flow targets Tauri installers
- repository builds successfully from a clean checkout

---

## Manual Verification

Verify:

- AI providers
- Companion window
- Cursor tracking
- Animations
- Tray
- Native menus
- Notifications
- Preferences
- Settings persistence
- Reminder system
- Pomodoro
- Water reminders
- Speech bubbles
- Updater runtime

Verify there are no functional regressions after Electron removal.

---

## Temporary Diagnostics

If temporary diagnostics are required:

- add diagnostics
- identify the issue
- remove diagnostics before completion

No debug code may remain in production.

---

## Documentation

Update:

- docs/migrating/progress.md

Record:

- removed Electron components
- repository cleanup
- validation performed
- manual verification
- final migration summary

---

## Commits

Create logical commits throughout the phase.

Suggested commit sequence:

```text
refactor(tauri): remove electron runtime

refactor(tauri): remove electron desktop bridge

refactor(tauri): remove electron packaging

refactor(tauri): remove migration compatibility

docs(tauri): finalize migration

chore(tauri): complete electron removal
```

If regressions are discovered:

```text
fix(tauri): ...
```

Do not squash unrelated work.

---

## Phase Exit Criteria

Phase 12 is complete only if:

✓ Electron Builder removed

✓ Electron runtime removed

✓ Electron dependencies removed

✓ Electron preload removed

✓ Electron main process removed

✓ Electron IPC removed

✓ Electron Builder configuration removed

✓ Electron packaging removed

✓ Electron compatibility code removed

✓ DesktopBridge is Tauri-only

✓ No Electron imports remain

✓ No Electron packages remain

✓ No Electron runtime code remains

✓ Repository builds using only Tauri

✓ Tauri production build succeeds

✓ macOS build succeeds

✓ Windows build succeeds (where supported)

✓ Linux build succeeds (where supported)

✓ AI works

✓ Tray works

✓ Notifications work

✓ Settings preserved

✓ Reminders preserved

✓ Pomodoro preserved

✓ Website download pipeline updated

✓ Release pipeline produces only Tauri artifacts

✓ No TODO items remain

✓ Repository is clean

✓ Migration documentation finalized

Only then may the migration be marked **COMPLETE**.

---

# Release Candidate Checklist

This checklist is an external release-operation gate that runs after the
repository migration is complete. It does not block Phase 12 engineering or
Electron source removal.

Release operators must not claim Ducky ready for public distribution until
every applicable item below is complete and its evidence is recorded in
`docs/RELEASING.md`.

## Production Credentials and Trust

- provision the stable Tauri updater signing identity
- commit and verify the matching updater public key through the documented
  reviewed path
- configure `TAURI_SIGNING_PRIVATE_KEY` and its password in the production CI
  environment
- configure required Apple signing/notarization credentials
- configure required Windows signing credentials and timestamp service
- verify every production credential is available to the intended protected
  workflow and is absent from source, logs, artifacts, and untrusted jobs
- run the release preflight and confirm it fails closed for missing or
  mismatched trust material

## Signed Transition Release

- create the final signed Electron transition release
- publish and retain its legacy updater metadata and referenced assets
- verify installed Electron clients discover the intended Tauri transition
  release
- verify the exact migration-dialog copy and both **Download PsyDuck 2.0** and
  **Remind Me Later** outcomes
- verify the Download action resolves only to the configured official release
  page
- record tag, commit, version, release URL, platforms, reviewer, date, and
  pass/fail evidence

## Staged Updater Verification

- generate signed Tauri packages, updater archives, `.sig` files, checksums,
  and `latest.json` for every supported platform and architecture
- verify signatures against the committed updater public key
- verify every metadata URL, version, platform, architecture, and checksum
- exercise clean installation and staged update detection
- verify signed download, installation, restart, settings preservation, and
  rollback behavior on supported macOS, Windows, and Linux systems
- confirm website download routes select only the intended Tauri installers

## Production Publication

- create the production GitHub release as a draft
- upload and redownload every staged asset for final verification
- confirm Electron and Tauri asset names and feeds cannot collide
- confirm legacy Electron metadata remains available
- publish atomically only after every build and verification job succeeds
- never replace assets on an already published release

## Final Go-Live

- confirm production updater detection from an installed supported Tauri
  release
- confirm public website downloads and release metadata resolve correctly
- confirm signing, notarization, installer reputation, and updater trust on
  each supported platform
- complete the security, rollback, support, and release-notes checklist
- obtain final release-manager approval and retain the completed evidence
  record

Only after this checklist passes may the migrated application be declared
ready for public go-live.
