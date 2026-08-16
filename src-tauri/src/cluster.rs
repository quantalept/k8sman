use std::path::PathBuf;

use kube::config::{KubeConfigOptions, Kubeconfig};
use kube::{Client, Config};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

const STORE_FILE: &str = "settings.json";
const KEY_KUBECONFIGS: &str = "kubeconfigPaths";
const KEY_LAST_CONTEXT: &str = "lastContext";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInfo {
    pub name: String,
    pub cluster: String,
    pub namespace: Option<String>,
    /// Path of the kubeconfig file this context was read from.
    pub source: String,
}

fn default_kubeconfig_path() -> Option<PathBuf> {
    if let Ok(v) = std::env::var("KUBECONFIG") {
        if let Some(first) = std::env::split_paths(&v).next() {
            return Some(first);
        }
    }
    home::home_dir().map(|h| h.join(".kube").join("config"))
}

fn known_paths(app: &AppHandle) -> AppResult<Vec<PathBuf>> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Message(e.to_string()))?;

    if let Some(value) = store.get(KEY_KUBECONFIGS) {
        let configured: Vec<PathBuf> = serde_json::from_value::<Vec<String>>(value)
            .unwrap_or_default()
            .into_iter()
            .map(PathBuf::from)
            .collect();
        if !configured.is_empty() {
            return Ok(configured);
        }
    }

    Ok(default_kubeconfig_path().into_iter().collect())
}

fn save_known_paths(app: &AppHandle, paths: &[PathBuf]) -> AppResult<()> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Message(e.to_string()))?;
    let strs: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    store.set(KEY_KUBECONFIGS, serde_json::json!(strs));
    store.save().map_err(|e| AppError::Message(e.to_string()))?;
    Ok(())
}

/// List the kubeconfig files currently known to the app.
#[tauri::command]
pub async fn list_kubeconfig_paths(app: AppHandle) -> AppResult<Vec<String>> {
    Ok(known_paths(&app)?
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

/// List every context found across all known kubeconfig files.
#[tauri::command]
pub async fn list_contexts(app: AppHandle) -> AppResult<Vec<ContextInfo>> {
    let paths = known_paths(&app)?;
    let mut out = Vec::new();

    for path in paths {
        let kubeconfig = match Kubeconfig::read_from(&path) {
            Ok(k) => k,
            Err(_) => continue,
        };
        let source = path.to_string_lossy().to_string();
        for named in &kubeconfig.contexts {
            if let Some(context) = &named.context {
                out.push(ContextInfo {
                    name: named.name.clone(),
                    cluster: context.cluster.clone(),
                    namespace: context.namespace.clone(),
                    source: source.clone(),
                });
            }
        }
    }

    Ok(out)
}

/// Import an additional kubeconfig file and return the updated context list.
#[tauri::command]
pub async fn add_kubeconfig(app: AppHandle, path: String) -> AppResult<Vec<ContextInfo>> {
    let new_path = PathBuf::from(&path);
    Kubeconfig::read_from(&new_path)
        .map_err(|e| AppError::Message(format!("invalid kubeconfig: {e}")))?;

    let mut paths = known_paths(&app)?;
    if !paths.contains(&new_path) {
        paths.push(new_path);
        save_known_paths(&app, &paths)?;
    }

    list_contexts(app).await
}

/// Forget a previously imported kubeconfig file and return the updated context list.
#[tauri::command]
pub async fn remove_kubeconfig(app: AppHandle, path: String) -> AppResult<Vec<ContextInfo>> {
    let mut paths = known_paths(&app)?;
    paths.retain(|p| p.to_string_lossy() != path);
    save_known_paths(&app, &paths)?;
    list_contexts(app).await
}

/// The last context that was switched to, if any, restored across app restarts.
#[tauri::command]
pub async fn current_context(app: AppHandle) -> AppResult<Option<String>> {
    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Message(e.to_string()))?;
    Ok(store
        .get(KEY_LAST_CONTEXT)
        .and_then(|v| v.as_str().map(|s| s.to_string())))
}

/// Connect to the given context, caching the client for reuse, and remember it as current.
#[tauri::command]
pub async fn switch_context(
    app: AppHandle,
    state: State<'_, AppState>,
    context_name: String,
) -> AppResult<ContextInfo> {
    let contexts = list_contexts(app.clone()).await?;
    let info = contexts
        .into_iter()
        .find(|c| c.name == context_name)
        .ok_or_else(|| AppError::Message(format!("unknown context: {context_name}")))?;

    let already_cached = state.clients.0.lock().unwrap().contains_key(&context_name);

    if !already_cached {
        let kubeconfig = Kubeconfig::read_from(&info.source)
            .map_err(|e| AppError::Message(format!("failed to read kubeconfig: {e}")))?;
        let options = KubeConfigOptions {
            context: Some(context_name.clone()),
            ..Default::default()
        };
        let config = Config::from_custom_kubeconfig(kubeconfig, &options)
            .await
            .map_err(|e| AppError::Message(format!("failed to build config: {e}")))?;
        let client = Client::try_from(config)?;
        state
            .clients
            .0
            .lock()
            .unwrap()
            .insert(context_name.clone(), client);
    }

    let store = app
        .store(STORE_FILE)
        .map_err(|e| AppError::Message(e.to_string()))?;
    store.set(KEY_LAST_CONTEXT, serde_json::json!(context_name));
    store.save().map_err(|e| AppError::Message(e.to_string()))?;

    Ok(info)
}

/// Fetch the cached client for a context, connecting first if necessary.
pub fn get_client(state: &AppState, context_name: &str) -> AppResult<Client> {
    state
        .clients
        .0
        .lock()
        .unwrap()
        .get(context_name)
        .cloned()
        .ok_or_else(|| AppError::Message(format!("not connected to context: {context_name}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KUBECONFIG: &str = r#"
apiVersion: v1
kind: Config
clusters:
- name: test-cluster
  cluster:
    server: https://127.0.0.1:6443
    insecure-skip-tls-verify: true
contexts:
- name: test-context
  context:
    cluster: test-cluster
    namespace: default
current-context: test-context
users: []
"#;

    #[test]
    fn parses_contexts_from_kubeconfig() {
        let kubeconfig = Kubeconfig::from_yaml(TEST_KUBECONFIG).expect("should parse test kubeconfig");
        assert_eq!(kubeconfig.contexts.len(), 1);
        assert_eq!(kubeconfig.contexts[0].name, "test-context");
        let ctx = kubeconfig.contexts[0].context.as_ref().unwrap();
        assert_eq!(ctx.cluster, "test-cluster");
        assert_eq!(ctx.namespace.as_deref(), Some("default"));
    }

    #[tokio::test]
    async fn builds_config_from_custom_kubeconfig() {
        let kubeconfig = Kubeconfig::from_yaml(TEST_KUBECONFIG).expect("should parse test kubeconfig");
        let options = KubeConfigOptions {
            context: Some("test-context".into()),
            ..Default::default()
        };
        let config = Config::from_custom_kubeconfig(kubeconfig, &options)
            .await
            .expect("should build config even for an unreachable cluster");
        assert_eq!(config.default_namespace, "default");
    }
}
