# Agent LogBook CLI API Smoke Test

This page gives a copy-paste path for testing the agent-facing HTTP API without
adding UI buttons.

## Start The Server

From the repository root:

```powershell
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
$env:HOME = $env:USERPROFILE
cd C:\cchv\claude-code-history-viewer\src-tauri
cargo run --features webui-server -- --serve --host 127.0.0.1 --port 3727 --token dev-token --dist ../dist
```

Leave that terminal open. In a second terminal, check that the service is alive:

```powershell
Invoke-RestMethod http://127.0.0.1:3727/health
```

## Query Local Sessions

Omit `host`, `username`, and `keyPath` to query the machine running Agent
LogBook. `provider` must be one of `claude`, `codex`, or `opencode`.

```powershell
$Base = "http://127.0.0.1:3727"
$Headers = @{ Authorization = "Bearer dev-token" }

$LocalListBody = @{
  provider = "opencode"
} | ConvertTo-Json

$LocalSessions = Invoke-RestMethod `
  -Method Post `
  -Uri "$Base/api/list_remote_sessions" `
  -Headers $Headers `
  -ContentType "application/json" `
  -Body $LocalListBody

$LocalSessions.sessions | Select-Object -First 10 `
  sessionId, projectName, title, messageCount, lastModified
```

Fetch one local session log:

```powershell
$LocalSessionId = $LocalSessions.sessions[0].sessionId

$LocalLogBody = @{
  provider = "opencode"
  sessionId = $LocalSessionId
  maxMessages = 50
} | ConvertTo-Json

$LocalLog = Invoke-RestMethod `
  -Method Post `
  -Uri "$Base/api/get_remote_session_log" `
  -Headers $Headers `
  -ContentType "application/json" `
  -Body $LocalLogBody

$LocalLog.session
$LocalLog.messages | Select-Object -First 5 type, timestamp, uuid
```

## Query A Remote Worker

Set the connection variables once. `provider` must be one of `claude`, `codex`,
or `opencode`. For remote queries, `host`, `username`, and `keyPath` must be
provided together.

```powershell
$Base = "http://127.0.0.1:3727"
$Headers = @{ Authorization = "Bearer dev-token" }

$HostName = "100.x.y.z"                         # Tailscale IP or MagicDNS name
$Port = 22
$Username = "ubuntu"
$KeyPath = "C:\Users\Administrator\.ssh\id_ed25519"
$Provider = "opencode"
```

List sessions:

```powershell
$ListBody = @{
  host = $HostName
  port = $Port
  username = $Username
  keyPath = $KeyPath
  provider = $Provider
} | ConvertTo-Json

$Sessions = Invoke-RestMethod `
  -Method Post `
  -Uri "$Base/api/list_remote_sessions" `
  -Headers $Headers `
  -ContentType "application/json" `
  -Body $ListBody

$Sessions.sessions | Select-Object -First 10 `
  sessionId, projectName, title, messageCount, lastModified
```

Fetch one session log:

```powershell
$SessionId = $Sessions.sessions[0].sessionId

$LogBody = @{
  host = $HostName
  port = $Port
  username = $Username
  keyPath = $KeyPath
  provider = $Provider
  sessionId = $SessionId
  maxMessages = 50
} | ConvertTo-Json

$Log = Invoke-RestMethod `
  -Method Post `
  -Uri "$Base/api/get_remote_session_log" `
  -Headers $Headers `
  -ContentType "application/json" `
  -Body $LogBody

$Log.session
$Log.messages | Select-Object -First 5 type, timestamp, uuid
```

Save the fetched log to a local JSON file:

```powershell
$Log | ConvertTo-Json -Depth 80 | Set-Content .\remote-session-log.json -Encoding UTF8
```

## curl Equivalent

```bash
curl -sS -X POST "http://127.0.0.1:3727/api/list_remote_sessions" \
  -H "Authorization: Bearer dev-token" \
  -H "Content-Type: application/json" \
  -d '{
    "host": "100.x.y.z",
    "port": 22,
    "username": "ubuntu",
    "keyPath": "C:\\Users\\Administrator\\.ssh\\id_ed25519",
    "provider": "opencode"
  }'
```

## Security Notes

- Local queries require no SSH fields.
- Remote queries accept SSH key authentication only.
- Do not put private key passphrases or bearer tokens into shared logs.
- Bind to `127.0.0.1` for local testing. Use firewalling and a real token before
  exposing the server beyond the local machine.
- The query is read-through: credentials are not stored as remote sources.
