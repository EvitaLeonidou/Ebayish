use crate::services::{WebSocketService, websocket_service::AuctionEvent};
use actix_web::web;
use chrono::Utc;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

pub struct CountdownService;

impl CountdownService {
    pub fn start_countdown_timer(
        pool: web::Data<PgPool>,
        websocket_service: web::Data<WebSocketService>,
    ) {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                if let Err(e) = Self::send_countdown_updates(&pool, &websocket_service).await {
                    tracing::warn!("Failed to send countdown updates: {:?}", e);
                }
            }
        });
    }

    async fn send_countdown_updates(
        pool: &PgPool,
        websocket_service: &WebSocketService,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();

        let active_auctions = sqlx::query!(
            r#"
            SELECT item_id, name, ends
            FROM items 
            WHERE ends > $1 
            AND item_id NOT IN (SELECT item_id FROM auction_results)
            ORDER BY ends ASC
            "#,
            now.naive_utc()
        )
        .fetch_all(pool)
        .await?;

        let active_auctions_len = active_auctions.len();
        for auction in active_auctions {
            let item_id_str = auction.item_id;
            let item_id = Uuid::parse_str(&item_id_str).map_err(|_| "Invalid UUID")?;
            let ends_at = auction
                .ends
                .ok_or_else(|| "Auction has no end date".to_string())?
                .and_utc();

            let time_remaining = ends_at.signed_duration_since(now);
            let time_remaining_seconds = time_remaining.num_seconds().max(0);

            if websocket_service.item_subscriber_count(item_id) > 0 {
                let event = AuctionEvent::CountdownUpdate {
                    item_id,
                    time_remaining_seconds,
                    timestamp: now,
                };

                if let Err(e) = websocket_service.broadcast_event(event) {
                    tracing::warn!(
                        "Failed to broadcast countdown update for item {}: {:?}",
                        item_id,
                        e
                    );
                }
            }
        }

        tracing::debug!(
            "Sent countdown updates for {} active auctions",
            active_auctions_len
        );
        Ok(())
    }

    pub async fn send_item_countdown_update(
        pool: &PgPool,
        websocket_service: &WebSocketService,
        item_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();

        let auction = sqlx::query!(
            r#"
            SELECT item_id, name, ends
            FROM items 
            WHERE item_id = $1 AND ends > $2
            AND item_id NOT IN (SELECT item_id FROM auction_results)
            "#,
            item_id.to_string(),
            now.naive_utc()
        )
        .fetch_optional(pool)
        .await?;

        if let Some(auction) = auction {
            let ends_at = auction
                .ends
                .ok_or_else(|| "Auction has no end date".to_string())?
                .and_utc();

            let time_remaining = ends_at.signed_duration_since(now);
            let time_remaining_seconds = time_remaining.num_seconds().max(0);

            let event = AuctionEvent::CountdownUpdate {
                item_id,
                time_remaining_seconds,
                timestamp: now,
            };

            websocket_service
                .broadcast_event(event)
                .map_err(|e| format!("WebSocket broadcast error: {:?}", e))?;
            tracing::debug!("Sent countdown update for item {}", item_id);
        }

        Ok(())
    }

    pub async fn send_auction_started_event(
        pool: &PgPool,
        websocket_service: &WebSocketService,
        item_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let item = sqlx::query!(
            r#"
            SELECT item_id, name, price, ends
            FROM items 
            WHERE item_id = $1
            "#,
            item_id.to_string()
        )
        .fetch_optional(pool)
        .await?;

        if let Some(item) = item {
            let event = AuctionEvent::AuctionStarted {
                item_id,
                title: item.name,
                starting_price: item.price,
                ends_at: item
                    .ends
                    .ok_or_else(|| "Item has no end date".to_string())?
                    .and_utc(),
                timestamp: Utc::now(),
            };

            websocket_service
                .broadcast_event(event)
                .map_err(|e| format!("WebSocket broadcast error: {:?}", e))?;
            tracing::info!("Sent auction started event for item {}", item_id);
        }

        Ok(())
    }
}
