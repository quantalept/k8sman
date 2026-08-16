mod cluster;
mod cp;
mod error;
mod exec;
mod logs;
mod metrics;
mod portforward;
mod resources;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

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
            resources::start_watch,
            resources::stop_watch,
            logs::start_log_stream,
            logs::stop_log_stream,
            exec::start_exec,
            exec::exec_write,
            exec::exec_resize,
            exec::stop_exec,
            portforward::start_port_forward,
            portforward::stop_port_forward,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
