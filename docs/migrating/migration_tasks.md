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

Tray + Menu

...

---

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