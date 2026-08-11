use crate::domain::NewBid;
use anyhow::Context;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum BidServiceError {
    #[error("Item not found")]
    ItemNotFound,
    #[error("Bid amount must be higher than current price: {current_price}")]
    BidTooLow { current_price: BigDecimal },
    #[error("Auction has ended")]
    AuctionEnded,
    #[error("Bid amount must meet minimum increment requirement")]
    InsufficientIncrement,
    #[error("Cannot bid on your own item")]
    SelfBidding,
    #[error("Buy-it-now price has been met, auction ended")]
    BuyItNowTriggered,
    #[error("Reserve price not met: minimum {reserve_price}")]
    ReservePriceNotMet { reserve_price: BigDecimal },
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

use crate::error_handling::error_chain_fmt;

impl std::fmt::Debug for BidServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Debug, Clone)]
pub struct ItemInfo {
    pub item_id: String,
    pub current_price: BigDecimal,
    pub buy_price: Option<BigDecimal>,
    pub ends: DateTime<Utc>,
    pub seller_user_id: Uuid,
    pub number_of_bids: i32,
    pub price: BigDecimal,
    pub is_ended: bool,
}

#[derive(Debug, Clone)]
pub struct BidResult {
    pub bid_id: Uuid,
    pub new_current_price: BigDecimal,
    pub is_buy_it_now: bool,
    pub auction_ended: bool,
}

pub struct BidService;

impl BidService {
    #[instrument(name = "Create bid with validation", skip(pool))]
    pub async fn create_bid(pool: &PgPool, new_bid: NewBid) -> Result<BidResult, BidServiceError> {
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire database transaction")?;

        let item_info = Self::get_item_info(&mut *transaction, &new_bid.item_id).await?;
        Self::validate_bid(&new_bid, &item_info).await?;

        let is_buy_it_now = item_info
            .buy_price
            .as_ref()
            .is_some_and(|buy_price| new_bid.amount >= *buy_price);

        let final_amount = if is_buy_it_now {
            item_info.buy_price.as_ref().unwrap().clone()
        } else {
            new_bid.amount.clone()
        };

        let bid_id = Uuid::new_v4();

        sqlx::query!(
            r#"INSERT INTO bids (id, item_id, bidder_user_id, bidder_rating, time, amount, bidder_location, bidder_country)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
            bid_id,
            new_bid.item_id,
            new_bid.bidder_user_id,
            new_bid.bidder_rating,
            new_bid.time.naive_utc(),
            final_amount,
            new_bid.bidder_location,
            new_bid.bidder_country
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to insert bid")?;

        //update item values
        sqlx::query!(
            r#"UPDATE items SET currently = $1, number_of_bids = number_of_bids + 1 WHERE item_id = $2"#,
            final_amount,
            new_bid.item_id
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to update item current price")?;

        //if buy it now end the auction
        let auction_ended = if is_buy_it_now {
            Self::end_auction(&mut *transaction, &new_bid.item_id).await?;
            true
        } else {
            false
        };

        transaction
            .commit()
            .await
            .context("Failed to commit bid transaction")?;

        Ok(BidResult {
            bid_id,
            new_current_price: final_amount,
            is_buy_it_now,
            auction_ended,
        })
    }

    #[instrument(name = "Get item info", skip(executor))]
    async fn get_item_info<'c, E>(executor: E, item_id: &str) -> Result<ItemInfo, BidServiceError>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let item_row = sqlx::query!(
            r#"SELECT item_id, currently, buy_price, ends, seller_user_id, number_of_bids, price
               FROM items WHERE item_id = $1"#,
            item_id
        )
        .fetch_optional(executor)
        .await
        .context("Failed to fetch item")?
        .ok_or(BidServiceError::ItemNotFound)?;

        //auction on top, buy it now on bottom
        let (ends_utc, is_ended) = match item_row.ends {
            Some(ends) => {
                let ends_utc = ends.and_utc();
                (ends_utc, ends_utc < Utc::now())
            }
            None => {
                let far_future = DateTime::parse_from_rfc3339("2099-12-31T23:59:59Z")
                    .unwrap()
                    .to_utc();
                (far_future, false)
            }
        };

        Ok(ItemInfo {
            item_id: item_row.item_id,
            current_price: item_row.currently.unwrap_or(item_row.price.clone()),
            buy_price: item_row.buy_price,
            ends: ends_utc,
            seller_user_id: item_row
                .seller_user_id
                .ok_or(BidServiceError::ItemNotFound)?,
            number_of_bids: item_row.number_of_bids.unwrap_or(0),
            price: item_row.price,
            is_ended,
        })
    }

    #[instrument(name = "Validate bid", skip(item_info))]
    async fn validate_bid(bid: &NewBid, item_info: &ItemInfo) -> Result<(), BidServiceError> {
        if item_info.is_ended {
            return Err(BidServiceError::AuctionEnded);
        }

        //stop seller from bidding
        if bid.bidder_user_id == item_info.seller_user_id {
            return Err(BidServiceError::SelfBidding);
        }

        #[allow(clippy::collapsible_if)]
        if let Some(buy_price) = &item_info.buy_price {
            if bid.amount >= *buy_price {
                return Ok(());
            }
        }

        let minimum_bid =
            Self::calculate_minimum_bid(&item_info.current_price, item_info.number_of_bids);
        if bid.amount < minimum_bid {
            return Err(BidServiceError::BidTooLow {
                current_price: minimum_bid,
            });
        }

        Ok(())
    }

    #[instrument(name = "Calculate minimum bid")]
    fn calculate_minimum_bid(current_price: &BigDecimal, bid_count: i32) -> BigDecimal {
        if bid_count == 0 {
            return current_price + BigDecimal::from_f64(0.01).unwrap_or_default();
        }

        let increment = if *current_price < BigDecimal::from(25) {
            BigDecimal::from_f64(0.50).unwrap_or_default()
        } else if *current_price < BigDecimal::from(100) {
            BigDecimal::from(1)
        } else if *current_price < BigDecimal::from(250) {
            BigDecimal::from_f64(2.50).unwrap_or_default()
        } else if *current_price < BigDecimal::from(500) {
            BigDecimal::from(5)
        } else {
            BigDecimal::from(10)
        };

        current_price + increment
    }

    #[instrument(name = "End auction", skip(executor))]
    async fn end_auction<'c, E>(executor: E, item_id: &str) -> Result<(), BidServiceError>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        sqlx::query!(
            r#"UPDATE items SET ends = NOW() WHERE item_id = $1"#,
            item_id
        )
        .execute(executor)
        .await
        .context("Failed to end auction")?;

        Ok(())
    }

    #[instrument(name = "Get auction winner", skip(pool))]
    pub async fn get_auction_winner(
        pool: &PgPool,
        item_id: &str,
    ) -> Result<Option<Uuid>, BidServiceError> {
        let winner = sqlx::query!(
            r#"SELECT bidder_user_id FROM bids 
               WHERE item_id = $1 
               ORDER BY amount DESC, time ASC 
               LIMIT 1"#,
            item_id
        )
        .fetch_optional(pool)
        .await
        .context("Failed to fetch auction winner")?;

        Ok(winner.and_then(|row| row.bidder_user_id))
    }

    #[instrument(name = "Check and end expired auctions", skip(pool))]
    pub async fn check_and_end_expired_auctions(
        pool: &PgPool,
    ) -> Result<Vec<String>, BidServiceError> {
        let expired_items = sqlx::query!(
            r#"SELECT item_id FROM items WHERE ends < NOW() AND ends > NOW() - INTERVAL '1 minute'"#
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch expired auctions")?;

        let item_ids: Vec<String> = expired_items.into_iter().map(|row| row.item_id).collect();

        tracing::info!("Found {} expired auctions", item_ids.len());

        Ok(item_ids)
    }

    #[instrument(name = "Get bid history", skip(pool))]
    pub async fn get_bid_history(
        pool: &PgPool,
        item_id: &str,
    ) -> Result<Vec<(Uuid, BigDecimal, DateTime<Utc>)>, BidServiceError> {
        let bids = sqlx::query!(
            r#"SELECT bidder_user_id, amount, time FROM bids 
               WHERE item_id = $1 
               ORDER BY amount DESC, time ASC"#,
            item_id
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch bid history")?;

        Ok(bids
            .into_iter()
            .filter_map(|row| {
                row.bidder_user_id
                    .map(|user_id| (user_id, row.amount, row.time.and_utc()))
            })
            .collect())
    }
}
