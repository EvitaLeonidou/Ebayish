use chrono::Utc;
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

// Trait for WebSocket services to avoid direct dependency on specific implementations
pub trait WebSocketNotifier: Send + Sync {
    fn item_subscriber_count(&self, item_id: Uuid) -> usize;
    fn broadcast_event(&self, event: AuctionEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Event types for auction notifications
#[derive(Debug, Clone)]
pub enum AuctionEvent {
    CountdownUpdate {
        item_id: Uuid,
        time_remaining_seconds: i64,
        timestamp: chrono::DateTime<Utc>,
    },
    AuctionStarted {
        item_id: Uuid,
        title: String,
        starting_price: bigdecimal::BigDecimal,
        ends_at: chrono::DateTime<Utc>,
        timestamp: chrono::DateTime<Utc>,
    },
    BidPlaced {
        item_id: Uuid,
        bid_id: Uuid,
        bidder_username: String,
        amount: bigdecimal::BigDecimal,
        current_price: bigdecimal::BigDecimal,
        bid_count: i32,
        timestamp: chrono::DateTime<Utc>,
    },
    AuctionEnded {
        item_id: Uuid,
        winner_username: Option<String>,
        winning_bid: Option<bigdecimal::BigDecimal>,
        timestamp: chrono::DateTime<Utc>,
    },
}

/// Service for managing real-time countdown timers for auctions
pub struct CountdownService;

impl CountdownService {
    /// Start the countdown timer background task
    /// This will continuously monitor active auctions and send countdown updates
    pub fn start_countdown_timer<W: WebSocketNotifier + 'static>(
        pool: PgPool,
        websocket_service: W,
    ) {
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10)); // Send updates every 10 seconds

            loop {
                interval.tick().await;

                if let Err(e) = Self::send_countdown_updates(&pool, &websocket_service).await {
                    tracing::warn!("Failed to send countdown updates: {:?}", e);
                }
            }
        });
    }

    /// Send countdown updates for all active auctions
    async fn send_countdown_updates<W: WebSocketNotifier>(
        pool: &PgPool,
        websocket_service: &W,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();

        // Query for active auctions (not ended, end time in future)
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
            let ends_at = auction.ends.and_utc();

            // Calculate time remaining
            let time_remaining = ends_at.signed_duration_since(now);
            let time_remaining_seconds = time_remaining.num_seconds().max(0);

            // Only send updates for auctions that have subscribers
            if websocket_service.item_subscriber_count(item_id) > 0 {
                let event = AuctionEvent::CountdownUpdate {
                    item_id,
                    time_remaining_seconds,
                    timestamp: now,
                };

                // Don't fail the entire loop if one broadcast fails
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

    /// Send countdown updates for a specific item (useful for immediate updates after bids)
    pub async fn send_item_countdown_update<W: WebSocketNotifier>(
        pool: &PgPool,
        websocket_service: &W,
        item_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now();

        // Query for the specific item
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
            let ends_at = auction.ends.and_utc();

            // Calculate time remaining
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

    /// Send auction started event when new auctions begin
    pub async fn send_auction_started_event<W: WebSocketNotifier>(
        pool: &PgPool,
        websocket_service: &W,
        item_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Query for the item details
        let item = sqlx::query!(
            r#"
            SELECT item_id, name, first_bid, ends
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
                starting_price: item.first_bid,
                ends_at: item.ends.and_utc(),
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