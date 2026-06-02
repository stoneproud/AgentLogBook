# Agent LogBook Remote Session Query API

Base URL defaults to `http://127.0.0.1:3727`.

Use `Authorization: Bearer <token>` for API requests. `/health` is unauthenticated.

## Endpoints

### POST `/api/list_remote_sessions`

Local request:

```json
{
  "provider": "opencode"
}
```

Remote request:


```json
{
  "host": "100.x.y.z",
  "port": 22,
  "username": "ubuntu",
  "keyPath": "C:\\Users\\Administrator\\.ssh\\id_ed25519",
  "provider": "opencode"
}
```

Fields:

- `provider`: `claude`, `codex`, or `opencode`.
- `host`: optional. Tailscale IP, MagicDNS name, or SSH host.
- `port`: SSH port. Defaults to `22` when omitted.
- `username`: optional SSH username.
- `keyPath`: optional local private key path on the machine running Agent LogBook.
- `passphrase`: optional encrypted-key passphrase. Avoid echoing it in logs.

Omit `host`, `username`, and `keyPath` together for local queries. Provide all
three together for remote SSH queries.

Response:

```json
{
  "host": "local",
  "provider": "opencode",
  "sessions": [
    {
      "provider": "opencode",
      "sessionId": "...",
      "actualSessionId": "...",
      "projectName": "workspace",
      "title": "Example session",
      "messageCount": 8,
      "firstMessageTime": "2026-05-26T...",
      "lastMessageTime": "2026-05-26T...",
      "lastModified": "2026-05-26T...",
      "sourceLabel": "..."
    }
  ]
}
```

### POST `/api/get_remote_session_log`

Request includes the same local/remote fields plus:

```json
{
  "sessionId": "...",
  "maxMessages": 50
}
```

`sessionId` may be `sessionId`, `actualSessionId`, or the file path suffix
returned by the scanner. `maxMessages` defaults to 200 and returns the newest
messages when truncation is needed.

Response:

```json
{
  "host": "100.x.y.z",
  "provider": "opencode",
  "session": { "...": "RemoteSessionSummary" },
  "messages": [],
  "truncated": false
}
```

## Minimal PowerShell Workflow

```powershell
$Base = "http://127.0.0.1:3727"
$Headers = @{ Authorization = "Bearer dev-token" }
$Common = @{ provider = "opencode" }

# Remote variant:
# $Common = @{
#   host = "100.x.y.z"
#   port = 22
#   username = "ubuntu"
#   keyPath = "C:\Users\Administrator\.ssh\id_ed25519"
#   provider = "opencode"
# }

$Sessions = Invoke-RestMethod -Method Post -Uri "$Base/api/list_remote_sessions" `
  -Headers $Headers -ContentType "application/json" `
  -Body ($Common | ConvertTo-Json)

$SessionId = $Sessions.sessions[0].sessionId
$LogBody = $Common + @{ sessionId = $SessionId; maxMessages = 50 }
$Log = Invoke-RestMethod -Method Post -Uri "$Base/api/get_remote_session_log" `
  -Headers $Headers -ContentType "application/json" `
  -Body ($LogBody | ConvertTo-Json)
```

## Operational Guardrails

- Prefer localhost binding for tests: `--host 127.0.0.1`.
- Never request password auth; remote mode is intentionally SSH-key-only.
- Treat returned messages as potentially sensitive user data. Summarize only the
  task-relevant parts unless the caller explicitly needs raw transcript lines.
- If SSH fails, report sanitized connection context: host, port, username, and
  provider. Do not print passphrases or private key contents.
