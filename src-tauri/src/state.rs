use std::collections::HashMap;
use std::sync::Mutex;

use kube::Client;
use tokio::task::AbortHandle;
use uuid::Uuid;

/// Per-context kube clients, cached so switching contexts doesn't reconnect every time.
#[derive(Default)]
pub struct ClientCache(pub Mutex<HashMap<String, Client>>);

/// Abort handles for cancellable background streams (watches, logs, exec, port-forwards),
/// keyed by a stream id handed back to the frontend when the stream is started.
#[derive(Default)]
pub struct StreamRegistry(pub Mutex<HashMap<Uuid, AbortHandle>>);

#[derive(Default)]
pub struct AppState {
    pub clients: ClientCache,
    pub streams: StreamRegistry,
}
