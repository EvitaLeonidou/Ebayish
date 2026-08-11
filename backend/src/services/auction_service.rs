use crate::services::bid_service::{BidService, BidServiceError};
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time;
use tracing::{info, instrument, warn};
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum AuctionServiceError {
    #[error("Auction not found")]
    AuctionNotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Bid service error: {0}")]
    BidServiceError(#[from] BidServiceError),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

use crate::error_handling::error_chain_fmt;

impl std::fmt::Debug for AuctionServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Debug, Clone)]
pub struct AuctionInfo {
    pub item_id: String,
    pub seller_user_id: Uuid,
    pub winner_user_id: Option<Uuid>,
    pub winning_amount: Option<bigdecimal::BigDecimal>,
    pub ended_at: DateTime<Utc>,
    pub total_bids: i32,
}

#[derive(Debug, Clone)]
pub struct AuctionStats {
    pub active_auctions: i64,
    pub ended_today: i64,
    pub total_bids_today: i64,
}

pub struct AuctionService;

impl AuctionService {
    /// Start the auction monitoring background task
    #[instrument(name = "Start auction monitor", skip(pool))]
    pub async fn start_auction_monitor(pool: PgPool) {
        info!("Starting auction monitoring service");

        //checks every 30 seconds
        let mut interval = time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            match Self::process_expired_auctions(&pool).await {
                Ok(ended_count) => {
                    if ended_count > 0 {
                        info!("Processed {} expired auctions", ended_count);
                    }
                }
                Err(e) => {
                    warn!("Failed to process expired auctions: {}", e);
                }
            }
        }
    }

    // Process all expired auctions and determine winners
    #[instrument(name = "Process expired auctions", skip(pool))]
    pub async fn process_expired_auctions(pool: &PgPool) -> Result<usize, AuctionServiceError> {
        let expired_auctions = sqlx::query!(
            r#"SELECT item_id FROM items 
               WHERE ends < NOW() 
               AND item_id NOT IN (
                   SELECT item_id FROM auction_results 
                   WHERE item_id IS NOT NULL
               )"#
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch expired auctions")?;

        let mut ended_count = 0;

        for auction in expired_auctions {
            match Self::end_auction(pool, &auction.item_id).await {
                Ok(_) => {
                    ended_count += 1;
                    info!("Successfully ended auction: {}", auction.item_id);
                }
                Err(e) => {
                    warn!("Failed to end auction {}: {}", auction.item_id, e);
                }
            }
        }

        Ok(ended_count)
    }

    #[instrument(name = "End auction", skip(pool))]
    pub async fn end_auction(
        pool: &PgPool,
        item_id: &str,
    ) -> Result<AuctionInfo, AuctionServiceError> {
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire database transaction")?;

        let item_info = sqlx::query!(
            r#"SELECT item_id, seller_user_id, number_of_bids, currently 
               FROM items WHERE item_id = $1"#,
            item_id
        )
        .fetch_optional(&mut *transaction)
        .await
        .context("Failed to fetch item info")?
        .ok_or(AuctionServiceError::AuctionNotFound)?;

        let winner = BidService::get_auction_winner(pool, item_id).await?;
        let winning_amount = if winner.is_some() {
            item_info.currently
        } else {
            None
        };

        // Create auction results record
        sqlx::query!(
            r#"INSERT INTO auction_results
               (item_id, seller_user_id, winner_user_id, winning_amount, ended_at, total_bids)
               VALUES ($1, $2, $3, $4, NOW(), $5)
               ON CONFLICT (item_id) DO NOTHING"#,
            item_id,
            item_info.seller_user_id,
            winner,
            winning_amount,
            item_info.number_of_bids
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to create auction results")?;

        sqlx::query!(
            r#"UPDATE items SET status = 'ended' WHERE item_id = $1"#,
            item_id
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to update item status to ended")?;

        transaction
            .commit()
            .await
            .context("Failed to commit auction ending transaction")?;

        Ok(AuctionInfo {
            item_id: item_id.to_string(),
            seller_user_id: item_info.seller_user_id.unwrap_or_default(),
            winner_user_id: winner,
            winning_amount,
            ended_at: Utc::now(),
            total_bids: item_info.number_of_bids.unwrap_or(0),
        })
    }

    #[instrument(name = "Get auction stats", skip(pool))]
    pub async fn get_auction_stats(pool: &PgPool) -> Result<AuctionStats, AuctionServiceError> {
        let active_auctions =
            sqlx::query_scalar!(r#"SELECT COUNT(*) FROM items WHERE ends > NOW()"#)
                .fetch_one(pool)
                .await
                .context("Failed to count active auctions")?
                .unwrap_or(0);

        let ended_today = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM auction_results 
               WHERE ended_at >= CURRENT_DATE"#
        )
        .fetch_one(pool)
        .await
        .context("Failed to count auctions ended today")?
        .unwrap_or(0);

        let total_bids_today = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM bids 
               WHERE DATE(time) = CURRENT_DATE"#
        )
        .fetch_one(pool)
        .await
        .context("Failed to count bids today")?
        .unwrap_or(0);

        Ok(AuctionStats {
            active_auctions,
            ended_today,
            total_bids_today,
        })
    }

    // Get all auction results
    #[instrument(name = "Get auction results", skip(pool))]
    pub async fn get_auction_results(
        pool: &PgPool,
    ) -> Result<Vec<AuctionInfo>, AuctionServiceError> {
        let results = sqlx::query!(
            r#"SELECT item_id, seller_user_id, winner_user_id, winning_amount, ended_at, total_bids
               FROM auction_results ORDER BY ended_at DESC"#
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch auction results")?;

        Ok(results
            .into_iter()
            .map(|row| AuctionInfo {
                item_id: row.item_id,
                seller_user_id: row.seller_user_id.unwrap_or_default(),
                winner_user_id: row.winner_user_id,
                winning_amount: row.winning_amount,
                ended_at: row.ended_at.and_utc(),
                total_bids: row.total_bids.unwrap_or(0),
            })
            .collect())
    }

    #[instrument(name = "Get auction result", skip(pool))]
    pub async fn get_auction_result(
        pool: &PgPool,
        item_id: &str,
    ) -> Result<Option<AuctionInfo>, AuctionServiceError> {
        let result = sqlx::query!(
            r#"SELECT item_id, seller_user_id, winner_user_id, winning_amount, ended_at, total_bids
               FROM auction_results WHERE item_id = $1"#,
            item_id
        )
        .fetch_optional(pool)
        .await
        .context("Failed to fetch auction result")?;

        Ok(result.map(|row| AuctionInfo {
            item_id: row.item_id,
            seller_user_id: row.seller_user_id.unwrap_or_default(),
            winner_user_id: row.winner_user_id,
            winning_amount: row.winning_amount,
            ended_at: row.ended_at.and_utc(),
            total_bids: row.total_bids.unwrap_or(0),
        }))
    }

    // Force end an auction admin function sets end time to now
    #[instrument(name = "Force end auction", skip(pool))]
    pub async fn force_end_auction(
        pool: &PgPool,
        item_id: &str,
    ) -> Result<AuctionInfo, AuctionServiceError> {
        sqlx::query!(
            r#"UPDATE items SET ends = NOW() WHERE item_id = $1"#,
            item_id
        )
        .execute(pool)
        .await
        .context("Failed to update auction end time")?;

        Self::end_auction(pool, item_id).await
    }
}
