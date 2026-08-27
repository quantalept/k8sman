use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use kube::Client;
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::exec::ExecSession;
use crate::resources::ResilientDiscovery;

/// Per-context kube clients, cached so switching contexts doesn't reconnect every time.
#[derive(Default)]
pub struct ClientCache(pub Mutex<HashMap<String, Client>>);

/// Per-context API discovery results, cached since a full discovery run costs 2N+1 requests.
#[derive(Default)]
pub struct DiscoveryCache(pub Mutex<HashMap<String, Arc<ResilientDiscovery>>>);

/// Abort handles for cancellable background streams (watches, logs, port-forwards),
/// keyed by a stream id handed back to the frontend when the stream is started.
#[derive(Default)]
pub struct StreamRegistry(pub Mutex<HashMap<Uuid, AbortHandle>>);

/// Live exec sessions. Uses an async mutex since writing to a session's stdin or resizing
/// its terminal needs to hold the guard across an `.await`.
#[derive(Default)]
pub struct ExecSessions(pub AsyncMutex<HashMap<Uuid, ExecSession>>);

#[derive(Default)]
pub struct AppState {
    pub clients: ClientCache,
    pub discovery: DiscoveryCache,
    pub streams: StreamRegistry,
    pub exec_sessions: ExecSessions,
}
