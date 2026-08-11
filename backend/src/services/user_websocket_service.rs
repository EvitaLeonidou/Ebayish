//! src/services/user_websocket_service.rs

#![allow(clippy::too_many_arguments)]

use actix_web::web;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::error_handling::error_chain_fmt;
use crate::services::websocket_service::WebSocketService;

#[derive(thiserror::Error)]
pub enum UserWebSocketError {
    #[error("User not found")]
    UserNotFound,
    #[error("Broadcast error: {0}")]
    BroadcastError(#[from] broadcast::error::SendError<UserEvent>),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

pub type UserWebSocketResult<T> = Result<T, Box<UserWebSocketError>>;

impl std::fmt::Debug for UserWebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum UserEvent {
    UserListingUpdate {
        user_id: Uuid,
        item_id: Uuid,
        new_current_price: BigDecimal,
        new_bid_count: i32,
        latest_bidder: String,
        timestamp: DateTime<Utc>,
    },
    UserAuctionEnded {
        user_id: Uuid,
        item_id: Uuid,
        final_price: BigDecimal,
        winner_username: Option<String>,
        total_bids: i32,
        timestamp: DateTime<Utc>,
    },
    UserBidStatusUpdate {
        user_id: Uuid,
        item_id: Uuid,
        bid_id: Uuid,
        old_status: String,
        new_status: String,
        new_current_price: BigDecimal,
        outbid_by: String,
        timestamp: DateTime<Utc>,
    },
    UserAuctionResult {
        user_id: Uuid,
        item_id: Uuid,
        result: String,
        final_price: BigDecimal,
        user_final_bid: BigDecimal,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone)]
pub struct UserSubscription {
    pub user_id: Uuid,
    pub connection_id: Uuid,
    pub event_types: Vec<String>,
}

#[derive(Clone)]
pub struct UserWebSocketService {
    user_event_sender: broadcast::Sender<UserEvent>,
    user_subscriptions: Arc<Mutex<HashMap<Uuid, Vec<UserSubscription>>>>,
}

impl UserWebSocketService {
    pub fn new(_websocket_service: web::Data<WebSocketService>) -> Self {
        let (user_event_sender, _) = broadcast::channel(1000);

        Self {
            user_event_sender,
            user_subscriptions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_user_event_receiver(&self) -> broadcast::Receiver<UserEvent> {
        self.user_event_sender.subscribe()
    }

    pub fn subscribe_to_user_events(
        &self,
        user_id: Uuid,
        connection_id: Uuid,
        event_types: Vec<String>,
    ) -> UserWebSocketResult<()> {
        let subscription = UserSubscription {
            user_id,
            connection_id,
            event_types,
        };

        let mut subscriptions = self.user_subscriptions.lock().unwrap();
        subscriptions.entry(user_id).or_default().push(subscription);

        tracing::debug!(
            "User {} subscribed to events with connection {}",
            user_id,
            connection_id
        );
        Ok(())
    }

    pub fn unsubscribe_user_connection(&self, connection_id: Uuid) -> UserWebSocketResult<()> {
        let mut subscriptions = self.user_subscriptions.lock().unwrap();

        for (_, user_subs) in subscriptions.iter_mut() {
            user_subs.retain(|sub| sub.connection_id != connection_id);
        }

        subscriptions.retain(|_, subs| !subs.is_empty());

        tracing::debug!("Unsubscribed connection {} from user events", connection_id);
        Ok(())
    }

    pub fn broadcast_user_event(&self, event: UserEvent) -> UserWebSocketResult<()> {
        tracing::debug!("Broadcasting user event: {:?}", event);

        match self.user_event_sender.send(event) {
            Ok(subscriber_count) => {
                tracing::debug!("User event broadcasted to {} subscribers", subscriber_count);
                Ok(())
            }
            Err(e) => Err(Box::new(UserWebSocketError::BroadcastError(e))),
        }
    }

    pub fn user_subscriber_count(&self, user_id: Uuid) -> usize {
        self.user_subscriptions
            .lock()
            .unwrap()
            .get(&user_id)
            .map(|subs| subs.len())
            .unwrap_or(0)
    }

    pub fn trigger_listing_update(
        &self,
        seller_id: Uuid,
        item_id: Uuid,
        new_current_price: BigDecimal,
        new_bid_count: i32,
        latest_bidder: String,
    ) -> UserWebSocketResult<()> {
        let event = UserEvent::UserListingUpdate {
            user_id: seller_id,
            item_id,
            new_current_price,
            new_bid_count,
            latest_bidder,
            timestamp: Utc::now(),
        };

        self.broadcast_user_event(event)
    }

    pub fn trigger_auction_ended(
        &self,
        seller_id: Uuid,
        item_id: Uuid,
        final_price: BigDecimal,
        winner_username: Option<String>,
        total_bids: i32,
    ) -> UserWebSocketResult<()> {
        let event = UserEvent::UserAuctionEnded {
            user_id: seller_id,
            item_id,
            final_price,
            winner_username,
            total_bids,
            timestamp: Utc::now(),
        };

        self.broadcast_user_event(event)
    }

    pub fn trigger_bid_status_update(
        &self,
        bidder_id: Uuid,
        item_id: Uuid,
        bid_id: Uuid,
        old_status: String,
        new_status: String,
        new_current_price: BigDecimal,
        outbid_by: String,
    ) -> UserWebSocketResult<()> {
        let event = UserEvent::UserBidStatusUpdate {
            user_id: bidder_id,
            item_id,
            bid_id,
            old_status,
            new_status,
            new_current_price,
            outbid_by,
            timestamp: Utc::now(),
        };

        self.broadcast_user_event(event)
    }

    pub fn trigger_auction_result(
        &self,
        bidder_id: Uuid,
        item_id: Uuid,
        result: String,
        final_price: BigDecimal,
        user_final_bid: BigDecimal,
    ) -> UserWebSocketResult<()> {
        let event = UserEvent::UserAuctionResult {
            user_id: bidder_id,
            item_id,
            result,
            final_price,
            user_final_bid,
            timestamp: Utc::now(),
        };

        self.broadcast_user_event(event)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum UserWebSocketRequest {
    SubscribeToUser {
        user_id: Uuid,
        event_types: Vec<String>,
    },
    UnsubscribeFromUser {
        user_id: Uuid,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum UserWebSocketResponse {
    UserSubscribed {
        user_id: Uuid,
        event_types: Vec<String>,
    },
    UserUnsubscribed {
        user_id: Uuid,
    },
    UserEventError {
        message: String,
    },
}
