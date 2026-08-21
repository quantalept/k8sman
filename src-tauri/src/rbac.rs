use k8s_openapi::api::rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, Role, RoleBinding, Subject};
use kube::api::{Api, ListParams};
use serde::Serialize;
use tauri::State;

use crate::cluster::get_client;
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleView {
    pub api_groups: Vec<String>,
    pub resources: Vec<String>,
    pub resource_names: Vec<String>,
    pub verbs: Vec<String>,
    pub non_resource_urls: Vec<String>,
}

impl From<&PolicyRule> for RuleView {
    fn from(r: &PolicyRule) -> Self {
        Self {
            api_groups: r.api_groups.clone().unwrap_or_default(),
            resources: r.resources.clone().unwrap_or_default(),
            resource_names: r.resource_names.clone().unwrap_or_default(),
            verbs: r.verbs.clone(),
            non_resource_urls: r.non_resource_urls.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingRules {
    pub binding_kind: String,
    pub binding_name: String,
    pub binding_namespace: Option<String>,
    pub role_kind: String,
    pub role_name: String,
    pub rules: Vec<RuleView>,
}

fn subject_matches(subject: &Subject, kind: &str, namespace: Option<&str>, name: &str) -> bool {
    if subject.kind != kind || subject.name != name {
        return false;
    }
    // ServiceAccount subjects always carry a namespace; User/Group subjects don't have one.
    if kind == "ServiceAccount" {
        return subject.namespace.as_deref() == namespace;
    }
    true
}

/// Resolves every Role/ClusterRole bound (directly) to the given subject - typically a
/// ServiceAccount - by aggregating RoleBindings and ClusterRoleBindings whose subjects match.
/// This is the same mechanism `kubectl auth can-i --list` relies on, and only needs read
/// access (unlike a SubjectAccessReview, which needs permission to create one).
#[tauri::command]
pub async fn get_subject_rules(
    state: State<'_, AppState>,
    context_name: String,
    kind: String,
    namespace: Option<String>,
    name: String,
) -> AppResult<Vec<BindingRules>> {
    let client = get_client(&state, &context_name)?;

    let role_bindings: Api<RoleBinding> = Api::all(client.clone());
    let cluster_role_bindings: Api<ClusterRoleBinding> = Api::all(client.clone());
    let cluster_roles: Api<ClusterRole> = Api::all(client.clone());

    let mut out = Vec::new();

    let rb_list = role_bindings.list(&ListParams::default()).await?;
    for rb in rb_list.items {
        let subjects = rb.subjects.as_deref().unwrap_or_default();
        if !subjects
            .iter()
            .any(|s| subject_matches(s, &kind, namespace.as_deref(), &name))
        {
            continue;
        }

        let role_ref = &rb.role_ref;
        let rules = if role_ref.kind == "ClusterRole" {
            cluster_roles
                .get(&role_ref.name)
                .await
                .ok()
                .and_then(|r| r.rules)
        } else {
            let binding_ns = rb.metadata.namespace.clone().unwrap_or_default();
            let roles_in_ns: Api<Role> = Api::namespaced(client.clone(), &binding_ns);
            roles_in_ns
                .get(&role_ref.name)
                .await
                .ok()
                .and_then(|r| r.rules)
        }
        .unwrap_or_default();

        out.push(BindingRules {
            binding_kind: "RoleBinding".into(),
            binding_name: rb.metadata.name.clone().unwrap_or_default(),
            binding_namespace: rb.metadata.namespace.clone(),
            role_kind: role_ref.kind.clone(),
            role_name: role_ref.name.clone(),
            rules: rules.iter().map(RuleView::from).collect(),
        });
    }

    let crb_list = cluster_role_bindings.list(&ListParams::default()).await?;
    for crb in crb_list.items {
        let subjects = crb.subjects.as_deref().unwrap_or_default();
        if !subjects
            .iter()
            .any(|s| subject_matches(s, &kind, namespace.as_deref(), &name))
        {
            continue;
        }

        let role_ref = &crb.role_ref;
        let rules = cluster_roles
            .get(&role_ref.name)
            .await
            .ok()
            .and_then(|r| r.rules)
            .unwrap_or_default();

        out.push(BindingRules {
            binding_kind: "ClusterRoleBinding".into(),
            binding_name: crb.metadata.name.clone().unwrap_or_default(),
            binding_namespace: None,
            role_kind: role_ref.kind.clone(),
            role_name: role_ref.name.clone(),
            rules: rules.iter().map(RuleView::from).collect(),
        });
    }

    Ok(out)
}
