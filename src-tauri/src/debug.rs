use k8s_openapi::api::core::v1::{EphemeralContainer, Pod};
use kube::api::{Api, PostParams};
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Debug pods created via `create_node_debug_pod` live here, mirroring `kubectl debug
/// node/<x>`'s default. A privileged pod on the host network/PID namespace is powerful
/// enough that it belongs alongside other cluster-admin tooling, not in an app namespace.
const DEBUG_NAMESPACE: &str = "kube-system";

#[derive(Debug, Clone, Serialize)]
pub struct DebugPodRef {
    pub namespace: String,
    pub name: String,
}

/// Create a privileged debug pod pinned to a specific node (bypassing the scheduler via
/// `spec.nodeName`, so it lands even on a cordoned/tainted node) with the host's root
/// filesystem mounted at `/host` - the same shape as `kubectl debug node/<x>`.
#[tauri::command]
pub async fn create_node_debug_pod(
    state: State<'_, AppState>,
    context_name: String,
    node_name: String,
    image: Option<String>,
) -> AppResult<DebugPodRef> {
    let client = get_client(&state, &context_name)?;
    let api: Api<Pod> = Api::namespaced(client, DEBUG_NAMESPACE);

    let image = image.unwrap_or_else(|| "busybox:1.36".to_string());
    let pod_name = format!("node-debug-{}", &uuid::Uuid::new_v4().to_string()[..8]);

    let pod: Pod = serde_json::from_value(serde_json::json!({
        "metadata": {
            "name": pod_name,
            "namespace": DEBUG_NAMESPACE,
            "labels": { "app.kubernetes.io/managed-by": "k8sman-node-debug" },
        },
        "spec": {
            "nodeName": node_name,
            "hostPID": true,
            "hostNetwork": true,
            "restartPolicy": "Never",
            "tolerations": [{ "operator": "Exists" }],
            "containers": [{
                "name": "debug",
                "image": image,
                "command": ["sleep", "infinity"],
                "stdin": true,
                "tty": true,
                "securityContext": { "privileged": true },
                "volumeMounts": [{ "name": "host-root", "mountPath": "/host" }],
            }],
            "volumes": [{ "name": "host-root", "hostPath": { "path": "/" } }],
        },
    }))
    .map_err(|e| AppError::Message(format!("failed to build debug pod: {e}")))?;

    let created = api.create(&PostParams::default(), &pod).await?;
    let name = created
        .metadata
        .name
        .ok_or_else(|| AppError::Message("created pod has no name".into()))?;
    Ok(DebugPodRef {
        namespace: DEBUG_NAMESPACE.to_string(),
        name,
    })
}

/// Attach an ephemeral debug container to a running pod. Ephemeral containers can't be
/// removed once attached, and the subresource patch doesn't merge lists, so this fetches
/// the current list and replaces it with the existing containers plus the new one.
#[tauri::command]
pub async fn add_ephemeral_container(
    state: State<'_, AppState>,
    context_name: String,
    namespace: String,
    pod: String,
    container_name: String,
    image: String,
    target_container: Option<String>,
) -> AppResult<Value> {
    let client = get_client(&state, &context_name)?;
    let api: Api<Pod> = Api::namespaced(client, &namespace);

    let mut current = api.get_ephemeral_containers(&pod).await?;
    let new_container: EphemeralContainer = serde_json::from_value(serde_json::json!({
        "name": container_name,
        "image": image,
        "stdin": true,
        "tty": true,
        "targetContainerName": target_container,
    }))
    .map_err(|e| AppError::Message(format!("failed to build ephemeral container: {e}")))?;

    let spec = current
        .spec
        .get_or_insert_with(k8s_openapi::api::core::v1::PodSpec::default);
    spec.ephemeral_containers
        .get_or_insert_with(Vec::new)
        .push(new_container);

    let updated = api
        .replace_ephemeral_containers(&pod, &PostParams::default(), &current)
        .await?;
    Ok(serde_json::to_value(updated).unwrap_or(Value::Null))
}
