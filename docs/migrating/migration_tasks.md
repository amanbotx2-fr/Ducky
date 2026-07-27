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

Settings

...

---

# PHASE 6

Credentials

...

---

# PHASE 7

Reminder System

...

---

# PHASE 8

Pomodoro

...

---

# PHASE 9

AI System

Subtasks

OpenAI

Gemini

Grok

Ollama

Custom Provider

AI Actions

Rate Limiting

Cancellation

Diagnostics

Exit Criteria

All providers functional.

---

# PHASE 10

Updater

...

---

# PHASE 11

Release Pipeline

...

---

# PHASE 12

Electron Removal

Only begins after

ALL previous phases complete.

Tasks

Remove Electron Builder

Remove Electron dependencies

Remove preload

Remove main process

Remove IPC

Remove Builder config

Acceptance

Repository builds using Tauri only.

---

# FINAL ACCEPTANCE

The migration is complete only if:

✓ Electron removed

✓ Tauri production build succeeds

✓ macOS build succeeds

✓ Windows build succeeds

✓ Linux build succeeds

✓ AI works

✓ Tray works

✓ Notifications work

✓ Settings preserved

✓ Reminders preserved

✓ Pomodoro preserved

✓ Website download pipeline updated

✓ Release pipeline updated

✓ No TODO items remain

✓ No Electron dependencies remain

Only then may the migration be marked COMPLETE.