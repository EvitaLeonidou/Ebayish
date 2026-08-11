use actix::prelude::*;
use actix_web::web;
use actix_web_actors::ws;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::error_handling::error_chain_fmt;

#[derive(thiserror::Error)]
pub enum WebSocketError {
    #[error("Connection not found")]
    ConnectionNotFound,
    #[error("Broadcast error: {0}")]
    BroadcastError(#[from] broadcast::error::SendError<AuctionEvent>),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

pub type WebSocketResult<T> = Result<T, Box<WebSocketError>>;

impl std::fmt::Debug for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AuctionEvent {
    BidPlaced {
        item_id: Uuid,
        bid_id: Uuid,
        bidder_username: String,
        amount: BigDecimal,
        current_price: BigDecimal,
        bid_count: i32,
        timestamp: DateTime<Utc>,
    },
    AuctionEnded {
        item_id: Uuid,
        winner_username: Option<String>,
        winning_bid: Option<BigDecimal>,
        timestamp: DateTime<Utc>,
    },
    CountdownUpdate {
        item_id: Uuid,
        time_remaining_seconds: i64,
        timestamp: DateTime<Utc>,
    },
    AuctionStarted {
        item_id: Uuid,
        title: String,
        starting_price: BigDecimal,
        ends_at: DateTime<Utc>,
        timestamp: DateTime<Utc>,
    },
    ItemSold {
        item_id: Uuid,
        buyer_username: String,
        price: BigDecimal,
        timestamp: DateTime<Utc>,
    },
    NotificationReceived {
        user_id: Uuid,
        notification_id: Uuid,
        title: String,
        message: String,
        notification_type: String,
        timestamp: DateTime<Utc>,
        item_id: Option<String>, //chat_room_id kept name for back compat
    },

    NewMessage {
        chat_room_id: Uuid,
        message_id: Uuid,
        sender_username: String,
        content: String,
        timestamp: DateTime<Utc>,
    },
    MessageDeleted {
        chat_room_id: Uuid,
        message_id: Uuid,
        deleted_by_username: String,
        timestamp: DateTime<Utc>,
    },
    ChatRoomCreated {
        chat_room_id: Uuid,
        other_user_id: Uuid,
        other_username: String,
        timestamp: DateTime<Utc>,
    },
    MessageNotification {
        user_id: Uuid,
        chat_room_id: Uuid,
        sender_username: String,
        preview: String,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub user_id: Option<Uuid>,
    pub username: Option<String>,
    pub subscribed_items: Vec<Uuid>,
    pub connection_id: Uuid,
}

#[derive(Clone)]
pub struct WebSocketService {
    event_sender: broadcast::Sender<AuctionEvent>,
    connections: Arc<Mutex<HashMap<Uuid, ConnectionInfo>>>,
    item_subscriptions: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>,
}

impl Default for WebSocketService {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn get_event_receiver(&self) -> broadcast::Receiver<AuctionEvent> {
        self.event_sender.subscribe()
    }

    pub fn register_connection(&self, connection_info: ConnectionInfo) -> WebSocketResult<()> {
        let connection_id = connection_info.connection_id;

        {
            let mut connections = self.connections.lock().unwrap();
            connections.insert(connection_id, connection_info.clone());
        }

        tracing::info!("WebSocket connection registered: {}", connection_id);
        Ok(())
    }

    pub fn unregister_connection(&self, connection_id: Uuid) -> WebSocketResult<()> {
        let connection_info = {
            let mut connections = self.connections.lock().unwrap();
            connections.remove(&connection_id)
        };

        if let Some(info) = connection_info {
            let mut subscriptions = self.item_subscriptions.lock().unwrap();
            for item_id in &info.subscribed_items {
                if let Some(subscribers) = subscriptions.get_mut(item_id) {
                    subscribers.retain(|&id| id != connection_id);
                    if subscribers.is_empty() {
                        subscriptions.remove(item_id);
                    }
                }
            }

            tracing::info!("WebSocket connection unregistered: {}", connection_id);
        }

        Ok(())
    }

    pub fn subscribe_to_item(&self, connection_id: Uuid, item_id: Uuid) -> WebSocketResult<()> {
        {
            let mut connections = self.connections.lock().unwrap();
            if let Some(connection) = connections.get_mut(&connection_id) {
                if !connection.subscribed_items.contains(&item_id) {
                    connection.subscribed_items.push(item_id);
                }
            } else {
                return Err(Box::new(WebSocketError::ConnectionNotFound));
            }
        }

        {
            let mut subscriptions = self.item_subscriptions.lock().unwrap();
            subscriptions
                .entry(item_id)
                .or_default()
                .push(connection_id);
        }

        tracing::debug!(
            "Connection {} subscribed to item {}",
            connection_id,
            item_id
        );
        Ok(())
    }

    pub fn unsubscribe_from_item(&self, connection_id: Uuid, item_id: Uuid) -> WebSocketResult<()> {
        {
            let mut connections = self.connections.lock().unwrap();
            if let Some(connection) = connections.get_mut(&connection_id) {
                connection.subscribed_items.retain(|&id| id != item_id);
            }
        }

        {
            let mut subscriptions = self.item_subscriptions.lock().unwrap();
            if let Some(subscribers) = subscriptions.get_mut(&item_id) {
                subscribers.retain(|&id| id != connection_id);
                if subscribers.is_empty() {
                    subscriptions.remove(&item_id);
                }
            }
        }

        tracing::debug!(
            "Connection {} unsubscribed from item {}",
            connection_id,
            item_id
        );
        Ok(())
    }

    pub fn broadcast_event(&self, event: AuctionEvent) -> WebSocketResult<()> {
        tracing::debug!("Broadcasting event: {:?}", event);

        match self.event_sender.send(event) {
            Ok(subscriber_count) => {
                tracing::debug!("Event broadcasted to {} subscribers", subscriber_count);
                Ok(())
            }
            Err(e) => Err(Box::new(WebSocketError::BroadcastError(e))),
        }
    }

    pub fn broadcast_notification_to_user(
        &self,
        user_id: Uuid,
        notification_id: Uuid,
        title: String,
        message: String,
        notification_type: String,
        item_id: Option<String>,
    ) -> WebSocketResult<()> {
        let event = AuctionEvent::NotificationReceived {
            user_id,
            notification_id,
            title,
            message,
            notification_type,
            timestamp: Utc::now(),
            item_id,
        };

        tracing::info!("Broadcasting notification to user {}: {:?}", user_id, event);
        let result = self.broadcast_event(event);
        if result.is_ok() {
            tracing::info!("Notification broadcast successful");
        } else {
            tracing::error!("Notification broadcast failed: {:?}", result);
        }
        result
    }

    pub fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

    pub fn item_subscriber_count(&self, item_id: Uuid) -> usize {
        self.item_subscriptions
            .lock()
            .unwrap()
            .get(&item_id)
            .map(|subs| subs.len())
            .unwrap_or(0)
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
}

#[derive(Debug, Serialize)]
pub struct WebSocketStats {
    pub active_connections: usize,
    pub total_subscriptions: usize,
    pub items_with_subscribers: usize,
}

pub struct WebSocketActor {
    connection_id: Uuid,
    websocket_service: web::Data<WebSocketService>,
    event_receiver: Option<broadcast::Receiver<AuctionEvent>>,
    user_id: Option<Uuid>,
    username: Option<String>,
}

impl WebSocketActor {
    pub fn new(
        websocket_service: web::Data<WebSocketService>,
        user_id: Option<Uuid>,
        username: Option<String>,
    ) -> Self {
        let connection_id = Uuid::new_v4();
        let event_receiver = websocket_service.get_event_receiver();

        Self {
            connection_id,
            websocket_service,
            event_receiver: Some(event_receiver),
            user_id,
            username,
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                if let Ok(request) = serde_json::from_str::<WebSocketRequest>(&text) {
                    self.handle_websocket_request(request, ctx);
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        let connection_info = ConnectionInfo {
            user_id: self.user_id,
            username: self.username.clone(),
            subscribed_items: Vec::new(),
            connection_id: self.connection_id,
        };

        if let Err(e) = self.websocket_service.register_connection(connection_info) {
            tracing::error!("Failed to register WebSocket connection: {:?}", e);
            ctx.stop();
            return;
        }

        if let Some(mut receiver) = self.event_receiver.take() {
            let addr = ctx.address();

            actix_web::rt::spawn(async move {
                while let Ok(event) = receiver.recv().await {
                    if addr.send(BroadcastEvent(event)).await.is_err() {
                        break;
                    }
                }
            });
        }
    }

    fn finished(&mut self, _: &mut Self::Context) {
        if let Err(e) = self
            .websocket_service
            .unregister_connection(self.connection_id)
        {
            tracing::error!("Failed to unregister WebSocket connection: {:?}", e);
        }
    }
}

impl Actor for WebSocketActor {
    type Context = ws::WebsocketContext<Self>;
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketRequest {
    Subscribe { item_id: Uuid },
    Unsubscribe { item_id: Uuid },
    Ping,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketResponse {
    Subscribed { item_id: Uuid },
    Unsubscribed { item_id: Uuid },
    Error { message: String },
    Pong,
}

impl WebSocketActor {
    fn handle_websocket_request(
        &self,
        request: WebSocketRequest,
        ctx: &mut ws::WebsocketContext<Self>,
    ) {
        match request {
            WebSocketRequest::Subscribe { item_id } => {
                match self
                    .websocket_service
                    .subscribe_to_item(self.connection_id, item_id)
                {
                    Ok(_) => {
                        let response = WebSocketResponse::Subscribed { item_id };
                        if let Ok(json) = serde_json::to_string(&response) {
                            ctx.text(json);
                        }
                    }
                    Err(e) => {
                        let response = WebSocketResponse::Error {
                            message: format!("Failed to subscribe: {:?}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&response) {
                            ctx.text(json);
                        }
                    }
                }
            }
            WebSocketRequest::Unsubscribe { item_id } => {
                match self
                    .websocket_service
                    .unsubscribe_from_item(self.connection_id, item_id)
                {
                    Ok(_) => {
                        let response = WebSocketResponse::Unsubscribed { item_id };
                        if let Ok(json) = serde_json::to_string(&response) {
                            ctx.text(json);
                        }
                    }
                    Err(e) => {
                        let response = WebSocketResponse::Error {
                            message: format!("Failed to unsubscribe: {:?}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&response) {
                            ctx.text(json);
                        }
                    }
                }
            }
            WebSocketRequest::Ping => {
                let response = WebSocketResponse::Pong;
                if let Ok(json) = serde_json::to_string(&response) {
                    ctx.text(json);
                }
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct BroadcastEvent(AuctionEvent);

impl Handler<BroadcastEvent> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: BroadcastEvent, ctx: &mut Self::Context) -> Self::Result {
        let event = msg.0;

        match &event {
            AuctionEvent::NotificationReceived { user_id, .. } => {
                tracing::info!(
                    "Processing notification for user {} on connection {:?}",
                    user_id,
                    self.user_id
                );
                if let Some(actor_user_id) = self.user_id {
                    if actor_user_id != *user_id {
                        tracing::debug!(
                            "Skipping notification - not for this user (expected: {}, actual: {})",
                            user_id,
                            actor_user_id
                        );
                        return;
                    }
                    tracing::info!("Sending notification to user {}", user_id);
                } else {
                    tracing::debug!("Skipping notification - unauthenticated connection");
                    return;
                }
            }
            _ => {
                tracing::debug!("Processing non-notification event: {:?}", event);
            }
        }

        if let Ok(json) = serde_json::to_string(&event) {
            ctx.text(json);
        }
    }
}
