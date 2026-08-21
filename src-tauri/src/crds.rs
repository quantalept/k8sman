use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::api::Api;
use serde::Serialize;
use tauri::State;

use crate::cluster::get_client;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrinterColumn {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub json_path: String,
}

/// Fetches a CRD's `additionalPrinterColumns` for the given served version - the same
/// columns `kubectl get <crd>` uses for its default table output.
#[tauri::command]
pub async fn get_crd_printer_columns(
    state: State<'_, AppState>,
    context_name: String,
    group: String,
    plural: String,
    version: String,
) -> AppResult<Vec<PrinterColumn>> {
    let client = get_client(&state, &context_name)?;
    let crds: Api<CustomResourceDefinition> = Api::all(client);
    let crd_name = format!("{plural}.{group}");
    let crd = crds.get(&crd_name).await.map_err(AppError::Kube)?;

    let version_spec = crd
        .spec
        .versions
        .iter()
        .find(|v| v.name == version)
        .ok_or_else(|| AppError::Message(format!("version {version} not found on CRD {crd_name}")))?;

    Ok(version_spec
        .additional_printer_columns
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|c| PrinterColumn {
            name: c.name,
            type_: c.type_,
            json_path: c.json_path,
        })
        .collect())
}
