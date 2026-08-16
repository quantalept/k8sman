# k8sman — Tauri + Rust (kube-rs) + Vue Kubernetes Desktop Client

## Context

OpenLens/Freelens are Electron-based and heavyweight. This project builds a lightweight
alternative using Tauri (Rust backend, native webview) with `kube-rs` talking directly to
the Kubernetes API (no shelling out to `kubectl`), and a Vue 3 + Vite frontend. The repo
started out empty aside from a LICENSE and two dark-themed dashboard mockups
(`design/screen.png`, `design/screen1.png`) — sidebar nav, card-based metrics with
sparklines, sortable resource tables. Those mockups also showed an "AI Copilot" panel; that
is **not** in scope for v1 (not part of the requested feature list) and was treated purely
as visual/style reference — dark theme, card layout, table style.

v1 feature scope (as specified):
1. Cluster configuration
2. Connect / context switching / resource browsing
3. Log watching
4. Pod exec
5. Copying files to/from pods
6. Port-forward
7. Basic metrics/graphs (metrics-server now, Prometheus-ready abstraction for later)

## Tech decisions

- **Backend**: Rust, Tauri v2, `kube` crate (client + runtime + config + ws features), `k8s-openapi`, `tokio`.
- **Frontend**: Vue 3 + TypeScript + Vite, package manager **pnpm**, UI kit **Naive UI**
  (dark-theme-first, tree-shakable, has the data-table/tree/dialog primitives the mockups need).
- **State**: Pinia (frontend), `Mutex`-guarded `AppState` (backend).
- **Terminal UI** (logs + exec): `xterm.js` (`@xterm/xterm` + fit addon).
- **Charts** (metrics): `uPlot` — small footprint, good for live-updating sparklines/graphs, wrapped in a thin Vue component.
- **Routing**: vue-router.

## Backend architecture (`src-tauri/`)

Modules, one per feature area:

- `cluster.rs` — kubeconfig discovery/import (multiple kubeconfig files, not just `~/.kube/config`),
  context listing, and a `Client` cache (`HashMap<String, kube::Client>` behind a `Mutex` in
  `AppState`) so switching contexts doesn't rebuild connections repeatedly.
- `discovery.rs` / `resources.rs` — use `kube::discovery::Discovery` to enumerate API
  resources dynamically (so CRDs work, not just hardcoded Pod/Deployment/etc). Generic
  list/get via `Api<DynamicObject>`. Live updates via `kube_runtime::watcher()` streamed to
  the frontend as Tauri events, keyed by a `stream_id`.
- `logs.rs` — `Api<Pod>::log_stream()` piped line-by-line into `app.emit("log:{stream_id}", ...)`;
  cancellable via a registry of abort handles.
- `exec.rs` — `Api<Pod>::exec()` with `AttachParams` (tty), bridging stdin (`exec_write` command)
  and stdout/stderr (`exec-output:{session_id}` events); resize wired to `AttachedProcess::resize_window`.
- `cp.rs` — kube-rs has no native `cp`; implemented the way `kubectl cp` does it: `tar` over
  `exec`. Download = `exec("tar cf - -C <dir> <file>")` + unpack locally with the `tar` crate.
  Upload = build an in-memory tar with the `tar` crate + `exec("tar xf - -C <dest>")`, write tar
  bytes to stdin. Local file picking via `tauri-plugin-dialog`. v1 scope: single file/dir path
  in, single path out — no interactive remote file browser (that needs `ls` parsing over exec;
  defer to a later version).
- `portforward.rs` — `Api<Pod>::portforward()` per local TCP connection: a local
  `TcpListener` accepts connections, each gets its own portforward stream, bridged with
  `tokio::io::copy_bidirectional`.
- `metrics.rs` — `MetricsProvider` trait (`pod_metrics()`, `node_metrics()`), with a
  `MetricsServerProvider` impl querying the `metrics.k8s.io/v1beta1` API via the dynamic API.
  Leaves room for a future `PrometheusProvider`. Stateless point-in-time fetch commands;
  the frontend polls (e.g. every 10–15s) and keeps its own rolling window for the graphs —
  keeps the Rust side simple, no duplicate buffering.

Cross-cutting:
- A `StreamRegistry` (`Mutex<HashMap<Uuid, AbortHandle>>`) in `AppState`, reused by
  watch/logs/exec/port-forward for start/stop symmetry.
- `thiserror`-based error enum, serialized to the frontend as `{ code, message }`.
- Local app config (known kubeconfig paths, last-used context, UI prefs) persisted via
  `tauri-plugin-store` in the app config dir.
- Tauri v2 capability/permission scoping so the webview has no direct filesystem/network
  access — everything routes through Rust commands.

## Frontend architecture (`src/`)

- `router/` — `/dashboard`, `/namespaces`, `/nodes`, `/pods`, `/pods/:ns/:name`,
  `/services`, `/configmaps`, `/secrets`, `/storage`, `/portforwards`, `/settings`.
- `stores/` (Pinia) — `cluster.ts`, `resources.ts` (generic per-GVK+namespace cache/watch
  subscriptions), `logs.ts`, `exec.ts`, `portforward.ts`, `metrics.ts`.
- `components/` — `Sidebar.vue`, `ResourceTable.vue` (generic, column-config driven so it
  covers Pods/Deployments/Services/etc without per-type components), `LogViewer.vue`
  (xterm), `ExecTerminal.vue` (xterm + resize), `PortForwardManager.vue`, `MetricCard.vue`
  (uPlot sparkline, matches the mockup's CPU/Memory cards), `ClusterSwitcher.vue`.
- `api/` — thin `invoke()` wrappers per feature area, plus `events.ts` for `listen()`
  subscriptions to the backend's streamed events.

## Build order (each stage reuses the streaming pattern established before it)

1. Scaffold: Tauri + Vue/Vite + pnpm, Naive UI, Pinia, vue-router, base dark-theme layout
   (sidebar + content + top bar) matching the mockups' look. **Done.**
2. Cluster configuration + context switching: `list_contexts`, `switch_context`,
   `current_context`, client cache, `ClusterSwitcher.vue`. **Done.**
3. Resource browsing: discovery + dynamic list/watch + generic `ResourceTable.vue`;
   Namespaces/Nodes/Pods/Services/ConfigMaps/Secrets views.
4. Log watching: backend stream + `LogViewer.vue`. Establishes the start/stop-stream IPC
   pattern reused below. **Done.**
5. Pod exec: backend bridge + `ExecTerminal.vue` with resize. **Done.**
6. Port-forward: backend TCP bridge + `PortForwardManager.vue`. **Done.**
7. File copy: tar-based `cp.rs` + minimal upload/download dialogs. **Done.**
8. Metrics/graphs: metrics-server dynamic API + frontend polling + `MetricCard.vue` on the dashboard.

## Verification

- `cargo build` in `src-tauri/` after each backend module lands.
- `pnpm dev` (Tauri dev mode) to exercise the UI live against a real cluster (kind/minikube
  is fine) after each stage — in particular: switch context, browse pods, tail logs, exec
  into a pod, copy a file in and out, open a port-forward and curl it, and confirm the
  dashboard metric cards populate (requires metrics-server installed in the test cluster).
- No existing test suite to extend yet; add basic `cargo test` coverage for the tar cp
  encode/decode logic and the generic resource column-mapping logic as those land.
