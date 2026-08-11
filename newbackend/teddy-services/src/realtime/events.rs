use crate::auctions::{AuctionEvent, WebSocketNotifier};
use uuid::Uuid;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub subscribed_items: Vec<Uuid>,
    pub connection_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct WebSocketStats {
    pub active_connections: usize,
    pub total_subscriptions: usize,
    pub items_with_subscribers: usize,
}

#[derive(Clone)]
pub struct WebSocketService {
    /// Broadcast sender for auction events
    event_sender: broadcast::Sender<AuctionEvent>,
    /// Active connections mapped by connection ID
    connections: Arc<Mutex<HashMap<Uuid, ConnectionInfo>>>,
    /// Item subscriptions - maps item_id to list of connection_ids
    item_subscriptions: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>,
}

impl WebSocketService {
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(1000);

        Self {
            event_sender,
            connections: Arc::new(Mutex::new(HashMap::new())),
            item_subscriptions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn broadcast_event(&self, event: AuctionEvent) -> Result<(), anyhow::Error> {
        match self.event_sender.send(event) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Broadcast error: {}", e)),
        }
    }

    pub fn get_stats(&self) -> WebSocketStats {
        let connections = self.connections.lock().unwrap();
        let subscriptions = self.item_subscriptions.lock().unwrap();

        WebSocketStats {
            active_connections: connections.len(),
            total_subscriptions: subscriptions.values().map(|v| v.len()).sum(),
            items_with_subscribers: subscriptions.len(),
        }
    }

    pub fn item_subscriber_count(&self, item_id: Uuid) -> usize {
        self.item_subscriptions
            .lock()
            .unwrap()
            .get(&item_id)
            .map(|subs| subs.len())
            .unwrap_or(0)
    }
}

impl Default for WebSocketService {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketNotifier for WebSocketService {
    fn item_subscriber_count(&self, item_id: Uuid) -> usize {
        self.item_subscriber_count(item_id)
    }

    fn broadcast_event(&self, event: AuctionEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.broadcast_event(event).map_err(|e| {
            let boxed_error: Box<dyn std::error::Error + Send + Sync> = Box::new(std::io::Error::new(std::io::ErrorKind::Other, e));
            boxed_error
        })
    }
}