<div align="center">

<img src="src-tauri/icons/icon.png" alt="Ducky" width="96" />

# Ducky

### A small native desktop AI companion for focused work.

![Version](https://img.shields.io/badge/version-1.1.0-5b6cff?style=flat-square)
![Platforms](https://img.shields.io/badge/macOS%20%7C%20Windows%20%7C%20Linux-supported?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-2ea44f?style=flat-square)

</div>

<p align="center">
  <img src="docs/images/hero.png" alt="Ducky desktop companion">
</p>

Ducky lives quietly on your desktop, helps you stay focused, and gives AI a
simple, personal place to live. It is local-first, cross-platform, and built
with Tauri v2, Rust, React, TypeScript, and Vite.

## Features

| | | |
|---|---|---|
| **AI companion**<br>Chat with OpenAI, Gemini, Claude, Grok, Ollama, and compatible endpoints. | **Smart reminders**<br>One-time and recurring reminders with a focused widget. | **Daily planner**<br>See today's schedule at a glance. |
| **Sticky message**<br>Keep one lightweight message visible. | **Pomodoro**<br>Run persistent focus sessions beside Ducky. | **Model Explorer**<br>Search, favorite, and switch between discovered models. |
| **Native desktop**<br>Tray, Preferences, dragging, and always-on-top support. | **Cross-platform**<br>macOS, Windows, and Linux packages. | **Native updates**<br>Automatic update checks through the native runtime. |

## Screenshot gallery

<table align="center">
  <tr>
    <td align="center"><strong>Desktop</strong><br><img src="docs/images/hero.png" alt="Ducky desktop companion" width="360"></td>
    <td align="center"><strong>AI conversation</strong><br><img src="docs/images/chat.png" alt="AI conversation" width="360"></td>
  </tr>
  <tr>
    <td align="center"><strong>Planner</strong><br><img src="docs/images/planner.png" alt="Daily Planner" width="360"></td>
    <td align="center"><strong>Preferences</strong><br><img src="docs/images/preferences.png" alt="Preferences" width="360"></td>
  </tr>
  <tr>
    <td align="center"><strong>Sticky message</strong><br><img src="docs/images/sticky-notes.png" alt="Sticky message" width="360"></td>
    <td align="center"><strong>Pomodoro</strong><br><img src="docs/images/pomodoro.png" alt="Pomodoro" width="360"></td>
  </tr>
  <tr>
    <td align="center"><strong>Reminders</strong><br><img src="docs/images/reminders.png" alt="Reminders" width="360"></td>
    <td align="center"><strong>About</strong><br><img src="docs/images/about.png" alt="About Ducky" width="360"></td>
  </tr>
</table>

## AI providers

- **OpenAI, Gemini, Claude, and Grok** — cloud providers using your own API
  credential.
- **Ollama** — local models through the loopback Ollama service.
- **OpenRouter and compatible endpoints** — OpenAI-compatible model
  catalogs and chat APIs.

Credentials stay in native secure storage and never enter normal renderer
state.

## Install

Download the latest release for your platform from
[GitHub Releases](https://github.com/amanbotx2-fr/Ducky/releases).

- macOS: open the DMG and drag Ducky to Applications.
- Windows: run the NSIS or MSI installer.
- Linux: launch the AppImage or install the DEB package.

## Build from source

Install Node.js 22.12+, npm 10+, the Rust toolchain, and the
[Tauri v2 platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri:dev
```

Build native packages with:

```bash
npm run tauri:build
```

## Validation

```bash
npm run typecheck
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```

## Releasing

Stable releases are built for macOS, Windows, and Linux when a matching
`vX.Y.Z` tag is pushed. See the
[automated release process](docs/RELEASING.md).

## Repository layout

`src-tauri` · Rust application lifecycle, native domains, persistence, menus,
tray, permissions, and packaging<br>
`src/desktop` · Private Tauri adapters behind the renderer-facing
DesktopBridge<br>
`src/renderer` · Companion and Preferences React applications<br>
`src/engine` · Runtime-neutral animation and input primitives<br>
`src/shared` · Typed renderer/native contracts and validation helpers<br>
`character` · Source artwork and animation frames<br>
`website` · Next.js landing site and permanent download routes

## License

[MIT](LICENSE) © Aman
