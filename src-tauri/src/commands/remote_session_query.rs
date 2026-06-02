//! Read-through queries for local or SSH-key-only remote provider sessions.
//!
//! These commands are intentionally narrower than persisted remote sources:
//! they do not save credentials and, when remote fields are present, accept only
//! key authentication. Omitting SSH fields queries local provider history.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{ClaudeMessage, ClaudeProject, ClaudeSession};
use crate::providers;
use crate::remote::source::{
    ProviderKind, RemoteAuth, RemotePodmanSettings, RemoteProviderPaths, RemoteSource,
    RemoteSystemKind,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSessionQuery {
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub key_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passphrase: Option<String>,
    pub provider: ProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_messages: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSessionSummary {
    pub provider: String,
    pub session_id: String,
    pub actual_session_id: String,
    pub project_name: String,
    pub title: Option<String>,
    pub message_count: usize,
    pub first_message_time: String,
    pub last_message_time: String,
    pub last_modified: String,
    pub source_label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSessionListResult {
    pub host: String,
    pub provider: String,
    pub sessions: Vec<RemoteSessionSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSessionLogResult {
    pub host: String,
    pub provider: String,
    pub session: RemoteSessionSummary,
    pub messages: Vec<ClaudeMessage>,
    pub truncated: bool,
}

#[derive(Debug, Clone)]
struct DiscoveredSession {
    summary: RemoteSessionSummary,
    file_path: String,
}

fn default_ssh_port() -> u16 {
    22
}

fn empty_other_provider_paths(provider: ProviderKind) -> RemoteProviderPaths {
    RemoteProviderPaths {
        claude: Some(if provider == ProviderKind::Claude {
            vec!["~/.claude".to_string()]
        } else {
            Vec::new()
        }),
        codex: Some(if provider == ProviderKind::Codex {
            vec!["~/.codex".to_string()]
        } else {
            Vec::new()
        }),
        opencode: Some(if provider == ProviderKind::OpenCode {
            vec!["~/.local/share/opencode".to_string()]
        } else {
            Vec::new()
        }),
    }
}

fn build_source(query: &RemoteSessionQuery) -> Result<RemoteSource, String> {
    let host = query.host.trim();
    let username = query.username.trim();
    let key_path = query.key_path.trim();
    if host.is_empty() || username.is_empty() || key_path.is_empty() {
        return Err("host, username, and keyPath are required".to_string());
    }
    if query.port == 0 {
        return Err("port must be greater than 0".to_string());
    }

    Ok(RemoteSource {
        id: format!("adhoc-session-query-{}", Uuid::new_v4()),
        enabled: true,
        host: host.to_string(),
        port: query.port,
        username: username.to_string(),
        system: RemoteSystemKind::Linux,
        auth: RemoteAuth::Key {
            key_path: key_path.to_string(),
            passphrase_ref: None,
            passphrase: query.passphrase.clone(),
        },
        paths: Some(empty_other_provider_paths(query.provider)),
        podman: Some(RemotePodmanSettings { enabled: false }),
        last_sync_at: None,
        last_sync_status: None,
        last_sync_error: None,
        last_sync_stats: None,
    })
}

fn uses_remote_ssh(query: &RemoteSessionQuery) -> Result<bool, String> {
    let has_host = !query.host.trim().is_empty();
    let has_username = !query.username.trim().is_empty();
    let has_key_path = !query.key_path.trim().is_empty();

    if has_host || has_username || has_key_path {
        if has_host && has_username && has_key_path {
            return Ok(true);
        }
        return Err(
            "host, username, and keyPath must be provided together for remote SSH queries"
                .to_string(),
        );
    }

    Ok(false)
}

fn discovered_session_matches(session: &DiscoveredSession, needle: &str) -> bool {
    session.summary.session_id == needle
        || session.summary.actual_session_id == needle
        || session.file_path == needle
        || session.file_path.ends_with(needle)
}

fn to_summary(
    provider: ProviderKind,
    source_label: &str,
    session: &ClaudeSession,
) -> RemoteSessionSummary {
    RemoteSessionSummary {
        provider: provider.as_str().to_string(),
        session_id: session.session_id.clone(),
        actual_session_id: session.actual_session_id.clone(),
        project_name: session.project_name.clone(),
        title: session.summary.clone(),
        message_count: session.message_count,
        first_message_time: session.first_message_time.clone(),
        last_message_time: session.last_message_time.clone(),
        last_modified: session.last_modified.clone(),
        source_label: source_label.to_string(),
    }
}

async fn scan_claude_root(
    root: &crate::remote::sync::InjectedRoot,
) -> Result<Vec<DiscoveredSession>, String> {
    let projects = crate::commands::project::scan_projects(root.local_path.clone()).await?;
    let mut out = Vec::new();
    for project in projects {
        let sessions =
            crate::commands::session::load_project_sessions(project.path.clone(), Some(false))
                .await?;
        for session in sessions {
            out.push(DiscoveredSession {
                summary: to_summary(ProviderKind::Claude, &root.discriminator, &session),
                file_path: session.file_path,
            });
        }
    }
    Ok(out)
}

fn scan_provider_root(
    provider: ProviderKind,
    root: &crate::remote::sync::InjectedRoot,
) -> Result<Vec<DiscoveredSession>, String> {
    let projects: Vec<ClaudeProject> = match provider {
        ProviderKind::Codex => providers::codex::scan_projects_from_path(&root.local_path)?,
        ProviderKind::OpenCode => {
            let projects = providers::opencode::scan_projects_from_path(&root.local_path)?;
            providers::opencode::scope_projects_to_base(projects, &root.local_path)
        }
        ProviderKind::Claude => return Ok(Vec::new()),
    };

    let mut out = Vec::new();
    for project in projects {
        let sessions = match provider {
            ProviderKind::Codex => providers::codex::load_sessions(&project.path, false),
            ProviderKind::OpenCode => providers::opencode::load_sessions(&project.path, false),
            ProviderKind::Claude => Ok(Vec::new()),
        }?;
        for session in sessions {
            out.push(DiscoveredSession {
                summary: to_summary(provider, &root.discriminator, &session),
                file_path: session.file_path,
            });
        }
    }
    Ok(out)
}

async fn scan_local_claude() -> Result<Vec<DiscoveredSession>, String> {
    let base = providers::claude::get_base_path().ok_or_else(|| "Claude not found".to_string())?;
    let projects = crate::commands::project::scan_projects(base).await?;
    let mut out = Vec::new();
    for project in projects {
        let sessions =
            crate::commands::session::load_project_sessions(project.path.clone(), Some(false))
                .await?;
        for session in sessions {
            out.push(DiscoveredSession {
                summary: to_summary(ProviderKind::Claude, "local", &session),
                file_path: session.file_path,
            });
        }
    }
    Ok(out)
}

fn scan_local_provider(provider: ProviderKind) -> Result<Vec<DiscoveredSession>, String> {
    let projects: Vec<ClaudeProject> = match provider {
        ProviderKind::Codex => providers::codex::scan_projects()?,
        ProviderKind::OpenCode => providers::opencode::scan_projects()?,
        ProviderKind::Claude => return Ok(Vec::new()),
    };

    let mut out = Vec::new();
    for project in projects {
        let sessions = match provider {
            ProviderKind::Codex => providers::codex::load_sessions(&project.path, false),
            ProviderKind::OpenCode => providers::opencode::load_sessions(&project.path, false),
            ProviderKind::Claude => Ok(Vec::new()),
        }?;
        for session in sessions {
            out.push(DiscoveredSession {
                summary: to_summary(provider, "local", &session),
                file_path: session.file_path,
            });
        }
    }
    Ok(out)
}

async fn collect_local_sessions(
    query: &RemoteSessionQuery,
) -> Result<Vec<DiscoveredSession>, String> {
    let mut sessions = match query.provider {
        ProviderKind::Claude => scan_local_claude().await?,
        ProviderKind::Codex => scan_local_provider(ProviderKind::Codex)?,
        ProviderKind::OpenCode => scan_local_provider(ProviderKind::OpenCode)?,
    };
    sessions.sort_by(|a, b| b.summary.last_modified.cmp(&a.summary.last_modified));
    Ok(sessions)
}

async fn collect_sessions(query: &RemoteSessionQuery) -> Result<Vec<DiscoveredSession>, String> {
    if !uses_remote_ssh(query)? {
        return collect_local_sessions(query).await;
    }

    let source = build_source(query)?;
    let outcome = crate::remote::sync_one(&source).await.map_err(|error| {
        crate::commands::remote_sync::public_error_for_source(error, Some(&source))
    })?;

    let mut sessions = Vec::new();
    match query.provider {
        ProviderKind::Claude => {
            for root in &outcome.injected_paths.claude {
                sessions.extend(scan_claude_root(root).await?);
            }
        }
        ProviderKind::Codex => {
            for root in &outcome.injected_paths.codex {
                sessions.extend(scan_provider_root(ProviderKind::Codex, root)?);
            }
        }
        ProviderKind::OpenCode => {
            for root in &outcome.injected_paths.opencode {
                sessions.extend(scan_provider_root(ProviderKind::OpenCode, root)?);
            }
        }
    }

    sessions.sort_by(|a, b| b.summary.last_modified.cmp(&a.summary.last_modified));
    Ok(sessions)
}

#[tauri::command]
pub async fn list_remote_sessions(
    query: RemoteSessionQuery,
) -> Result<RemoteSessionListResult, String> {
    let sessions = collect_sessions(&query)
        .await?
        .into_iter()
        .map(|session| session.summary)
        .collect::<Vec<_>>();

    Ok(RemoteSessionListResult {
        host: if query.host.trim().is_empty() {
            "local".to_string()
        } else {
            query.host
        },
        provider: query.provider.as_str().to_string(),
        sessions,
    })
}

#[tauri::command]
pub async fn get_remote_session_log(
    query: RemoteSessionQuery,
) -> Result<RemoteSessionLogResult, String> {
    let session_id = query
        .session_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "sessionId is required".to_string())?;
    let sessions = collect_sessions(&query).await?;
    let session = sessions
        .into_iter()
        .find(|candidate| discovered_session_matches(candidate, session_id))
        .ok_or_else(|| "session not found".to_string())?;

    let mut messages = match query.provider {
        ProviderKind::Claude => {
            crate::commands::session::load_session_messages(session.file_path.clone()).await?
        }
        ProviderKind::Codex => providers::codex::load_messages(&session.file_path)?,
        ProviderKind::OpenCode => providers::opencode::load_messages(&session.file_path)?,
    };

    let max_messages = query.max_messages.unwrap_or(200);
    let truncated = messages.len() > max_messages;
    if truncated {
        let start = messages.len().saturating_sub(max_messages);
        messages = messages.split_off(start);
    }

    Ok(RemoteSessionLogResult {
        host: if query.host.trim().is_empty() {
            "local".to_string()
        } else {
            query.host
        },
        provider: query.provider.as_str().to_string(),
        session: session.summary,
        messages,
        truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query(provider: ProviderKind) -> RemoteSessionQuery {
        RemoteSessionQuery {
            host: "worker.tailnet.example".to_string(),
            port: 22,
            username: "ubuntu".to_string(),
            key_path: "/home/me/.ssh/id_ed25519".to_string(),
            passphrase: None,
            provider,
            session_id: None,
            max_messages: None,
        }
    }

    #[test]
    fn build_source_uses_key_auth_only() {
        let source = build_source(&query(ProviderKind::OpenCode)).expect("source");

        assert!(matches!(source.auth, RemoteAuth::Key { .. }));
    }

    #[test]
    fn build_source_limits_paths_to_requested_provider() {
        let source = build_source(&query(ProviderKind::OpenCode)).expect("source");
        let paths = source.paths.expect("paths");

        assert_eq!(paths.claude, Some(Vec::new()));
        assert_eq!(paths.codex, Some(Vec::new()));
        assert_eq!(
            paths.opencode,
            Some(vec!["~/.local/share/opencode".to_string()])
        );
    }

    #[test]
    fn empty_connection_fields_select_local_mode() {
        let mut query = query(ProviderKind::OpenCode);
        query.host.clear();
        query.username.clear();
        query.key_path.clear();

        assert!(!uses_remote_ssh(&query).expect("mode"));
    }

    #[test]
    fn partial_connection_fields_are_rejected() {
        let mut query = query(ProviderKind::OpenCode);
        query.username.clear();

        assert!(uses_remote_ssh(&query).is_err());
    }
}
