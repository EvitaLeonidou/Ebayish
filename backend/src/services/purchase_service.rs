use crate::domain::Notification;
use crate::error_handling::error_chain_fmt;
use crate::services::NotificationService;
use crate::services::websocket_service::{AuctionEvent, WebSocketService};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum PurchaseServiceError {
    #[error("Item not found")]
    ItemNotFound,
    #[error("Item is not available for purchase")]
    ItemNotAvailable,
    #[error("Item has already been sold")]
    ItemAlreadySold,
    #[error("This item is an auction, use bidding instead")]
    ItemIsAuction,
    #[error("User not found")]
    UserNotFound,
    #[error("Cannot purchase your own item")]
    CannotPurchaseOwnItem,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PurchaseServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Debug, Clone)]
pub struct PurchaseInfo {
    pub item_id: String,
    pub buyer_user_id: Uuid,
    pub seller_user_id: Uuid,
    pub purchase_price: BigDecimal,
    pub purchased_at: DateTime<Utc>,
    pub item_name: String,
}

pub struct PurchaseService;

impl PurchaseService {
    #[tracing::instrument(name = "Purchase item", skip(pool, websocket_service))]
    pub async fn purchase_item(
        pool: &PgPool,
        websocket_service: &WebSocketService,
        item_id: &str,
        buyer_user_id: Uuid,
    ) -> Result<PurchaseInfo, PurchaseServiceError> {
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire database transaction")?;

        let item_info = sqlx::query!(
            r#"SELECT item_id, listing_type, name, price, buy_price, seller_user_id, status
               FROM items WHERE item_id = $1"#,
            item_id
        )
        .fetch_optional(&mut *transaction)
        .await
        .context("Failed to fetch item info")?
        .ok_or(PurchaseServiceError::ItemNotFound)?;

        if item_info.status.as_deref() == Some("sold") {
            return Err(PurchaseServiceError::ItemAlreadySold);
        }

        if item_info.status.as_deref() == Some("ended") {
            return Err(PurchaseServiceError::ItemNotAvailable);
        }

        //check if user is trying to purchase their own item
        if item_info.seller_user_id == Some(buyer_user_id) {
            return Err(PurchaseServiceError::CannotPurchaseOwnItem);
        }

        let purchase_price = match item_info.listing_type.as_str() {
            "fixed_price" => item_info.price,
            "auction" => {
                // for auction items, use buy_price if available
                item_info
                    .buy_price
                    .clone()
                    .ok_or(PurchaseServiceError::ItemIsAuction)?
            }
            _ => return Err(PurchaseServiceError::ItemNotAvailable),
        };

        // Get buyer username for notification
        let buyer_username =
            sqlx::query_scalar!(r#"SELECT username FROM users WHERE id = $1"#, buyer_user_id)
                .fetch_optional(&mut *transaction)
                .await
                .context("Failed to fetch buyer username")?
                .ok_or(PurchaseServiceError::UserNotFound)?;

        // add topurchase record
        let purchase_id = Uuid::new_v4();
        sqlx::query!(
            r#"INSERT INTO purchases
               (id, item_id, buyer_user_id, seller_user_id, purchase_price, purchased_at)
               VALUES ($1, $2, $3, $4, $5, NOW())"#,
            purchase_id,
            item_id,
            buyer_user_id,
            item_info.seller_user_id,
            purchase_price
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to create purchase record")?;

        sqlx::query!(
            r#"UPDATE items SET status = 'sold' WHERE item_id = $1"#,
            item_id
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to update item status to sold")?;

        transaction
            .commit()
            .await
            .context("Failed to commit purchase transaction")?;

        let purchase_info = PurchaseInfo {
            item_id: item_id.to_string(),
            buyer_user_id,
            seller_user_id: item_info.seller_user_id.unwrap_or_default(),
            purchase_price: purchase_price.clone(),
            purchased_at: Utc::now(),
            item_name: item_info.name.clone(),
        };

        let item_id_uuid = Uuid::parse_str(item_id).map_err(|e| {
            PurchaseServiceError::WebSocketError(format!("Invalid item UUID: {}", e))
        })?;

        // create notification for seller about item sold
        if let Some(seller_id) = item_info.seller_user_id {
            let notification = Notification::new_item_sold(
                seller_id,
                item_id.to_string(),
                &buyer_username,
                purchase_price.clone(),
                &item_info.name,
            );

            if let Err(e) =
                NotificationService::create_notification(pool, &notification, websocket_service)
                    .await
            {
                tracing::warn!("Failed to create item sold notification: {}", e);
            }
        }

        //if this was an auction item (had buy_price), notify all existing bidders that they lost
        if item_info.listing_type == "auction" && item_info.buy_price.is_some() {
            //gets bidder excludes buyer
            let bidders = sqlx::query!(
                r#"SELECT DISTINCT b.bidder_user_id, u.username, MAX(b.amount) as final_bid
                   FROM bids b
                   JOIN users u ON b.bidder_user_id = u.id
                   WHERE b.item_id = $1 AND b.bidder_user_id != $2
                   GROUP BY b.bidder_user_id, u.username"#,
                item_id,
                buyer_user_id
            )
            .fetch_all(pool)
            .await
            .context("Failed to fetch auction bidders")?;

            //broadcasts notifications

            let bidder_count = bidders.len();
            for bidder in &bidders {
                if let (Some(bidder_id), Some(final_bid)) =
                    (bidder.bidder_user_id, bidder.final_bid.clone())
                {
                    let notification = Notification::new_auction_lost(
                        bidder_id,
                        item_id.to_string(),
                        &item_info.name,
                        final_bid,
                        purchase_price.clone(),
                        &buyer_username,
                    );

                    if let Err(e) = NotificationService::create_notification(
                        pool,
                        &notification,
                        websocket_service,
                    )
                    .await
                    {
                        tracing::warn!(
                            "Failed to create auction lost notification for user {}: {}",
                            bidder_id,
                            e
                        );
                    }
                }
            }

            tracing::info!(
                "Sent auction lost notifications to {} bidders for buy now purchase",
                bidder_count
            );
        }

        let event = AuctionEvent::ItemSold {
            item_id: item_id_uuid,
            buyer_username,
            price: purchase_price,
            timestamp: Utc::now(),
        };

        if let Err(e) = websocket_service.broadcast_event(event) {
            tracing::warn!("Failed to broadcast ItemSold event: {:?}", e);
        }

        tracing::info!("Item {} purchased by user {}", item_id, buyer_user_id);

        Ok(purchase_info)
    }

    #[tracing::instrument(name = "Get buyer purchases", skip(pool))]
    pub async fn get_buyer_purchases(
        pool: &PgPool,
        buyer_user_id: Uuid,
    ) -> Result<Vec<PurchaseInfo>, PurchaseServiceError> {
        let purchases = sqlx::query!(
            r#"SELECT p.item_id, p.buyer_user_id, p.seller_user_id, p.purchase_price, p.purchased_at, i.name
               FROM purchases p
               JOIN items i ON p.item_id = i.item_id
               WHERE p.buyer_user_id = $1
               ORDER BY p.purchased_at DESC"#,
            buyer_user_id
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch buyer purchases")?;

        Ok(purchases
            .into_iter()
            .map(|row| PurchaseInfo {
                item_id: row.item_id.unwrap_or_default(),
                buyer_user_id: row.buyer_user_id.unwrap_or_default(),
                seller_user_id: row.seller_user_id.unwrap_or_default(),
                purchase_price: row.purchase_price,
                purchased_at: row.purchased_at.and_utc(),
                item_name: row.name,
            })
            .collect())
    }

    #[tracing::instrument(name = "Get seller sales", skip(pool))]
    pub async fn get_seller_sales(
        pool: &PgPool,
        seller_user_id: Uuid,
    ) -> Result<Vec<PurchaseInfo>, PurchaseServiceError> {
        let sales = sqlx::query!(
            r#"SELECT p.item_id, p.buyer_user_id, p.seller_user_id, p.purchase_price, p.purchased_at, i.name
               FROM purchases p
               JOIN items i ON p.item_id = i.item_id
               WHERE p.seller_user_id = $1
               ORDER BY p.purchased_at DESC"#,
            seller_user_id
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch seller sales")?;

        Ok(sales
            .into_iter()
            .map(|row| PurchaseInfo {
                item_id: row.item_id.unwrap_or_default(),
                buyer_user_id: row.buyer_user_id.unwrap_or_default(),
                seller_user_id: row.seller_user_id.unwrap_or_default(),
                purchase_price: row.purchase_price,
                purchased_at: row.purchased_at.and_utc(),
                item_name: row.name,
            })
            .collect())
    }

    #[tracing::instrument(name = "Check if item is sold", skip(pool))]
    pub async fn is_item_sold(pool: &PgPool, item_id: &str) -> Result<bool, PurchaseServiceError> {
        let sold = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM purchases WHERE item_id = $1)"#,
            item_id
        )
        .fetch_one(pool)
        .await
        .context("Failed to check if item is sold")?
        .unwrap_or(false);

        Ok(sold)
    }
}
