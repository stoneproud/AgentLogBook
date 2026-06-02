---
name: agent-logbook-query
description: Query coding-agent conversation history through an Agent LogBook HTTP server. Use when a user or supervising agent needs to list local sessions or fetch a specific Claude Code, Codex CLI, or OpenCode session log, including from a VM reachable by SSH key in a Tailscale-style private network.
---

# Agent LogBook Query

## Overview

Use Agent LogBook as a read-only bridge to coding-agent history. Omitting SSH
fields queries the local machine running Agent LogBook; providing `host`,
`username`, and `keyPath` together queries a remote worker over SSH key auth.

## Workflow

1. Confirm the Agent LogBook server base URL and bearer token. Default local dev
   is `http://127.0.0.1:3727` with whatever token the operator chose.
2. Ask for or derive `provider`. For local queries, omit SSH fields. For remote
   queries, provide `host`, `port`, `username`, and `keyPath`.
3. Call `POST /api/list_remote_sessions` to identify the likely session.
4. Call `POST /api/get_remote_session_log` with `sessionId` and a bounded
   `maxMessages` value.
5. Summarize only the relevant messages by default. Treat the raw transcript as
   sensitive operational data.

## API Reference

Read `references/api.md` when you need exact request/response shapes or
copy-paste PowerShell examples.

## Guardrails

- Do not request or use SSH password authentication.
- Omit `host`, `username`, and `keyPath` for local queries; do not send partial
  SSH fields.
- Do not print private key contents, passphrases, or bearer tokens.
- Prefer `maxMessages` over full logs unless the task truly requires everything.
- If the API returns an error, report the endpoint and sanitized target fields.
