<div align="center">

<img src="docs/assets/app-icon.png" alt="Agent LogBook Logo" width="120" />

# Agent LogBook

**A desktop logbook and local service layer for coding-agent conversations.**

Browse, search, and analyze conversations from **Claude Code**, **Gemini CLI**, **Antigravity**, **Codex CLI**, **Cline**, **Cursor**, **Aider**, **OpenCode**, and **ForgeCode** — as a desktop app or headless server. 100% offline.

[![License](https://img.shields.io/github/license/stoneproud/AgentLogBook)](LICENSE)
[![Rust Tests](https://img.shields.io/github/actions/workflow/status/stoneproud/AgentLogBook/rust-tests.yml?label=Rust%20Tests)](https://github.com/stoneproud/AgentLogBook/actions/workflows/rust-tests.yml)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

[Report Bug](https://github.com/stoneproud/AgentLogBook/issues)

Agent LogBook is an independent soft fork based on [jhlee0409/claude-code-history-viewer](https://github.com/jhlee0409/claude-code-history-viewer). The original project, license, and author attribution are preserved; this fork hosts larger experiments around local APIs, remote session sync, and agent-facing history access.

</div>

---

<p align="center">
  <img width="49%" alt="Conversation History" src="https://github.com/user-attachments/assets/9a18304d-3f08-4563-a0e6-dd6e6dfd227e" />
  <img width="49%" alt="Analytics Dashboard" src="https://github.com/user-attachments/assets/0f869344-4a7c-4f1f-9de3-701af10fc255" />
</p>
<p align="center">
  <img width="49%" alt="Token Statistics" src="https://github.com/user-attachments/assets/d30f3709-1afb-4f76-8f06-1033a3cb7f4a" />
  <img width="49%" alt="Recent Edits" src="https://github.com/user-attachments/assets/8c9fbff3-55dd-4cfc-a135-ddeb719f3057" />
</p>

## Quick Start

This fork does not publish prebuilt binaries — build from source:

```bash
git clone https://github.com/stoneproud/AgentLogBook.git
cd AgentLogBook

# Option 1: Using just (recommended)
brew install just    # or: cargo install just
just setup
just dev             # Development
just tauri-build     # Production build

# Option 2: Using pnpm directly
pnpm install
pnpm tauri:dev       # Development
pnpm tauri:build     # Production build
```

**Requirements**: Node.js 18+, pnpm, Rust toolchain

**Headless server** — access from any browser:

```bash
just serve-build-run   # Build frontend, embed into server binary, and run
# → http://localhost:3727
```

See [Server Mode](#server-mode-webui) for Docker, systemd, and CLI options.

## What This Fork Adds

On top of the upstream viewer, Agent LogBook adds a service layer for getting agent history **into and out of** the machine you're sitting at:

| Feature | Description |
|---------|-------------|
| **SSH Remote Session Sync** | Pull Claude Code / Codex CLI / OpenCode session history from SSH-accessible Linux and Windows machines into a local cache (pure-Rust SFTP, key + password auth, incremental mtime+size sync). Synced data flows through the normal scanner — projects, search, and stats all work unchanged. |
| **Credential Storage in OS Keychain** | Remote credentials are stored in the operating system keychain, not plaintext config. |
| **Podman Container Discovery** | Discover and scan agent histories that live inside local or remote Podman containers, with configurable discovery. |
| **Agent-Facing Log Query API** | HTTP API (`/api/list_remote_sessions`, `/api/get_remote_session_log`, …) so other agents and scripts can query session history programmatically. See [docs/agent-api-cli.md](docs/agent-api-cli.md) and the bundled [`agent-logbook-query` skill](skills/agent-logbook-query/SKILL.md). |
| **History Backup & Restore** | Export and restore full history backups. |

> **Note**: This fork has auto-update disabled — updates come from rebuilding the source. The upstream auto-updater (which would replace your build with upstream binaries) is intentionally turned off.

## Why This Exists

AI coding assistants generate thousands of conversation messages, but none of them provide a way to look back at your history across tools — let alone across machines.

**Nine assistants. One logbook.** Switch between Claude Code, Gemini CLI, Antigravity, Codex CLI, Cline, Cursor, Aider, OpenCode, and ForgeCode sessions seamlessly — compare token usage, search across providers, and analyze your workflow in a single interface.

| Provider | Data Location | What You Get |
|----------|--------------|--------------|
| **Claude Code** | `~/.claude/projects/` | Full conversation history, tool use, thinking, costs |
| **Gemini CLI** | `~/.gemini/history/` | Conversation history with tool calls |
| **Antigravity** | `~/.gemini/antigravity/` | Conversation state under `brain/` plus token monitor data under `.token-monitor/rpc-cache/v1/` |
| **Codex CLI** | `~/.codex/sessions/` | Session rollouts with agent responses |
| **Cline** | `~/.cline/tasks/` | Task-based conversation history |
| **Cursor** | `~/.cursor/` | Composer and chat conversations |
| **Aider** | Project directories | Chat history and edit logs |
| **OpenCode** | `~/.local/share/opencode/` | Conversation sessions and tool results |
| **ForgeCode** | `~/.forge/.forge.db` | Conversation history from SQLite database |

No vendor lock-in. No cloud dependency. Your local conversation files, beautifully rendered.

## Table of Contents

- [What This Fork Adds](#what-this-fork-adds)
- [Features](#features)
- [Build from Source](#quick-start)
- [Server Mode (WebUI)](#server-mode-webui)
- [Usage](#usage)
- [Accessibility](#accessibility)
- [Tech Stack](#tech-stack)
- [Data Privacy](#data-privacy)
- [Troubleshooting](#troubleshooting)
- [License](#license)

## Features

### Core

| Feature | Description |
|---------|-------------|
| **Multi-Provider Support** | Unified viewer for **Claude Code**, **Gemini CLI**, **Antigravity**, **Codex CLI**, **Cline**, **Cursor**, **Aider**, **OpenCode**, and **ForgeCode** — filter by provider, compare across tools |
| **Conversation Browser** | Navigate conversations by project/session with worktree grouping |
| **Global Search** | Search across all conversations from all providers instantly |
| **Analytics Dashboard** | Dual-mode token stats (billing vs conversation), cost breakdown, and provider distribution charts |
| **Session Board** | Multi-session visual analysis with pixel view, attribute brushing, and activity timeline |
| **Settings Manager** | Scope-aware Claude Code settings editor with MCP server management |
| **Message Navigator** | Right-side collapsible TOC for quick conversation navigation |
| **Real-time Monitoring** | Live session file watching for instant updates |
| **WebUI Server Mode** | Run as a headless web server with `--serve` — access from any browser, deploy on VPS/Docker |
| **WSL Support** | Windows Subsystem for Linux integration — scan Claude Code projects inside WSL distros |
| **External Session Launch** | `--session <uuid>` CLI flag with single-instance enforcement |
| **Session Management** | Delete sessions (move to trash), reveal JSONL files, native rename, copy resume command |
| **Screenshot Capture** | Long screenshot with range selection, preview modal, and multi-selection export |
| **Archive Management** | Create, browse, rename, and export session archives with per-file download |
| **ANSI Color Rendering** | Terminal output displayed with original ANSI colors |
| **Multi-language** | English, Korean, Japanese, Chinese (Simplified & Traditional) |
| **Recent Edits** | View file modification history and restore |

## Server Mode (WebUI)

Run the viewer as a headless HTTP server — no desktop environment required. Ideal for VPS, remote servers, or Docker. The server binary embeds the frontend — **a single file is all you need**.

> See the full [Server Mode Guide](docs/server-guide.md) for step-by-step instructions covering local testing, VPS setup, Docker, and more.

### Build and Start

```bash
just serve-build           # Build frontend + embed into server binary
just serve-build-run       # Build and run (embedded assets)

# Or run in development (external dist/):
just serve-dev             # Build frontend + run server with --dist
```

Output:

```
🔑 Auth token: b77f41d4-ec24-4102-8f7a-8a942d6dd4a0
   Open in browser: http://192.168.1.10:3727?token=b77f41d4-ec24-4102-8f7a-8a942d6dd4a0
👁 File watcher active: /home/user/.claude/projects
🚀 WebUI server running at http://0.0.0.0:3727
```

Open the URL in your browser — the token is saved automatically.

**CLI options:**

| Flag | Default | Description |
|------|---------|-------------|
| `--serve` | — | **Required.** Starts the HTTP server instead of the desktop app |
| `--port <number>` | `3727` | Server port |
| `--host <address>` | `0.0.0.0` | Bind address (`127.0.0.1` for local only) |
| `--token <value>` | auto (uuid v4) | Custom authentication token |
| `--no-auth` | — | Disable authentication (not recommended for public networks) |
| `--dist <path>` | embedded | Override built-in frontend with external `dist/` directory |

### Authentication

All `/api/*` endpoints are protected by Bearer token authentication. The token is auto-generated on each server start and printed to stderr.

- **Browser access**: Use the `?token=...` URL printed at startup. The token is saved to `localStorage` automatically.
- **API access**: Include `Authorization: Bearer <token>` header.
- **Custom token**: `--token my-secret-token` to set your own.
- **Environment variable**: `CCHV_TOKEN=your-token cchv-server --serve` (useful for systemd/Docker).
- **Disable**: `--no-auth` to skip authentication entirely (only use on trusted networks).

### Agent-Facing Query API

Other agents and scripts can query session history over the same server. List sessions for a provider (local or remote over SSH), then fetch a full session log:

```
POST /api/list_remote_sessions    { "provider": "claude" }
POST /api/get_remote_session_log  { "provider": "claude", "sessionId": "..." }
```

See [docs/agent-api-cli.md](docs/agent-api-cli.md) for a copy-paste smoke test and [`skills/agent-logbook-query`](skills/agent-logbook-query/SKILL.md) for the agent skill that wraps it.

### Real-time Updates

The server watches `~/.claude/projects/` for file changes and pushes updates to the browser via Server-Sent Events (SSE). When you use Claude Code in another terminal, the viewer updates automatically — no manual refresh needed.

### Docker

```bash
docker compose up -d
```

Check the token after startup:

```bash
docker compose logs webui
# 🔑 Auth token: ... ← paste this URL in your browser
```

The `docker-compose.yml` mounts `~/.claude`, `~/.codex`, and `~/.local/share/opencode` as read-only volumes.

### systemd Service

For persistent server on Linux, use the provided systemd template:

```bash
sudo cp contrib/cchv.service /etc/systemd/system/
sudo systemctl edit --full cchv.service   # Set User= to your username
sudo systemctl enable --now cchv.service
```

### Health Check

```
GET /health
→ { "status": "ok" }
```

## Usage

1. Launch the app
2. It automatically scans for conversation data from all supported providers
3. Browse projects in the left sidebar — filter by provider using the tab bar
4. Click a session to view messages
5. Use tabs to switch between Messages, Analytics, Token Stats, Recent Edits, and Session Board
6. To pull history from another machine, add an SSH remote source in settings — synced sessions appear alongside local ones

### Command-line flags

Launch the app pre-focused on a specific session by passing a `--session` flag:

```bash
# Full UUID
claude-code-history-viewer --session 1265cd74-caa9-472e-b343-c4f44b5cf12c

# UUID prefix (8+ hex-or-dash chars, up to 36) — first match wins
claude-code-history-viewer --session 1265cd74

# Equals form also works
claude-code-history-viewer --session=1265cd74
```

The viewer scans every known project, navigates to the matching session, and falls back to normal startup if no session matches. Values that are neither hex-or-dash of length 8..36 nor an absolute path are silently ignored.

## Accessibility

The app includes accessibility features for keyboard-only, low-vision, and screen-reader users.

- Keyboard-first navigation:
  - Skip links for Project Explorer, Main Content, Message Navigator, and Settings
  - Project tree navigation with `ArrowUp/ArrowDown/Home/End`, type-ahead search, and `*` to expand sibling groups
  - Message navigator navigation with `ArrowUp/ArrowDown/Home/End` and `Enter` to open the focused message
- Visual accessibility:
  - Persistent global font size scaling (`90%`, `100%`, `110%`, `120%`, `130%`)
  - High contrast mode toggle in settings
- Screen reader support:
  - Landmark and tree/list semantics (`navigation`, `tree`, `treeitem`, `group`, `listbox`, `option`)
  - Live announcements for status/loading and project tree navigation/selection changes
  - Inline keyboard-help descriptions via `aria-describedby`

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Backend** | ![Rust](https://img.shields.io/badge/Rust-000?logo=rust&logoColor=white) ![Tauri](https://img.shields.io/badge/Tauri_v2-24C8D8?logo=tauri&logoColor=white) |
| **Frontend** | ![React](https://img.shields.io/badge/React_19-61DAFB?logo=react&logoColor=black) ![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white) ![Tailwind](https://img.shields.io/badge/Tailwind_CSS-06B6D4?logo=tailwindcss&logoColor=white) |
| **State** | ![Zustand](https://img.shields.io/badge/Zustand-433E38?logo=react&logoColor=white) |
| **Build** | ![Vite](https://img.shields.io/badge/Vite-646CFF?logo=vite&logoColor=white) |
| **i18n** | ![i18next](https://img.shields.io/badge/i18next-26A69A?logo=i18next&logoColor=white) 5 languages |

## Data Privacy

**100% offline.** No conversation data is sent to any server. No analytics, no tracking, no telemetry.

Remote session sync is point-to-point over SSH between your own machines; credentials live in the OS keychain. Your data stays on your machines.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "No Claude data found" | Make sure `~/.claude` exists with conversation history |
| Performance issues | Large histories may be slow initially — the app uses virtual scrolling |
| Remote sync fails | Verify SSH connectivity (`ssh user@host`) and that the remote data directories exist |

## Development

Run checks before committing:

```bash
pnpm tsc --build .         # TypeScript
pnpm vitest run            # Tests
pnpm lint                  # Lint
cd src-tauri && cargo test -- --test-threads=1   # Rust tests
```

See [Development Commands](CLAUDE.md#development-commands) for the full list of available commands.

## License

[MIT](LICENSE) — free for personal and commercial use.

Based on [Claude Code History Viewer](https://github.com/jhlee0409/claude-code-history-viewer) by JaeHyeok Lee. See [NOTICE.md](NOTICE.md) for attribution.
