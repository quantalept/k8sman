mod cluster;
mod cp;
mod crds;
mod debug;
mod error;
mod exec;
mod helm;
mod logs;
mod metrics;
mod nodes;
mod portforward;
mod rbac;
mod resources;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    // kube's TLS stack and reqwest's (used for Prometheus queries) each pull in rustls with
    // their own crypto backend choice; without an explicit default, rustls can't pick one
    // unambiguously at runtime and panics on first use. Install one up front.
    let _ = rustls::crypto::CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            cluster::list_kubeconfig_paths,
            cluster::list_contexts,
            cluster::add_kubeconfig,
            cluster::remove_kubeconfig,
            cluster::current_context,
            cluster::switch_context,
            resources::list_resource_kinds,
            resources::list_resources,
            resources::get_resource,
            resources::delete_resource,
            resources::apply_resource,
            resources::scale_resource,
            resources::restart_rollout,
            resources::start_watch,
            resources::stop_watch,
            nodes::cordon_node,
            nodes::taint_node,
            nodes::untaint_node,
            nodes::start_node_drain,
            debug::create_node_debug_pod,
            debug::add_ephemeral_container,
            logs::start_log_stream,
            logs::stop_log_stream,
            exec::start_exec,
            exec::exec_write,
            exec::exec_resize,
            exec::stop_exec,
            portforward::start_port_forward,
            portforward::stop_port_forward,
            cp::cp_to_pod,
            cp::cp_from_pod,
            metrics::get_pod_metrics,
            metrics::get_node_metrics,
            metrics::get_prometheus_url,
            metrics::set_prometheus_url,
            metrics::query_prometheus_range,
            rbac::get_subject_rules,
            crds::get_crd_printer_columns,
            helm::list_helm_releases,
            helm::get_helm_release,
            helm::list_helm_release_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
