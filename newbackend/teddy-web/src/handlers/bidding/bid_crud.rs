use teddy_domain::NewBid;
use teddy_services::{
    BidService, WebSocketService, websocket_service::AuctionEvent,
};
use actix_web::{HttpResponse, web};
use anyhow::Context;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;
use crate::errors::bidding::BidError;

#[derive(Debug, Deserialize)]
pub struct BidRequest {
    bidder_user_id: Uuid,
    bidder_rating: Option<i32>,
    time: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_amount")]
    amount: BigDecimal,
    bidder_location: Option<String>,
    bidder_country: Option<String>,
}

fn deserialize_amount<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;

    match value {
        serde_json::Value::String(s) => {
            tracing::debug!("Deserializing amount from string: {}", s);
            BigDecimal::from_str(&s)
                .map_err(|e| D::Error::custom(format!("Invalid decimal string: {}", e)))
        }
        serde_json::Value::Number(n) => {
            tracing::debug!("Deserializing amount from number: {:?}", n);
            if let Some(i) = n.as_i64() {
                Ok(BigDecimal::from(i))
            } else if let Some(f) = n.as_f64() {
                Ok(BigDecimal::from_f64(f).unwrap_or_else(|| BigDecimal::from(0)))
            } else {
                Err(D::Error::custom("Number too large for BigDecimal"))
            }
        }
        _ => Err(D::Error::custom("Amount must be a number or string")),
    }
}

#[derive(Serialize)]
pub struct Bid {
    id: Uuid,
    item_id: String,
    bidder_user_id: Uuid,
    bidder_rating: Option<i32>,
    time: DateTime<Utc>,
    amount: BigDecimal,
    bidder_location: Option<String>,
    bidder_country: Option<String>,
}

// Database representation struct
#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct BidRow {
    id: Uuid,
    item_id: String,
    bidder_user_id: Uuid,
    bidder_rating: Option<i32>,
    time: NaiveDateTime,
    amount: BigDecimal,
    bidder_location: Option<String>,
    bidder_country: Option<String>,
}

impl BidRequest {
    fn into_new_bid(self, item_id: String) -> Result<NewBid, String> {
        let bid = NewBid {
            item_id,
            bidder_user_id: self.bidder_user_id,
            bidder_rating: self.bidder_rating,
            time: self.time,
            amount: self.amount,
            bidder_location: self.bidder_location,
            bidder_country: self.bidder_country,
        };

        bid.validate()?;
        Ok(bid)
    }
}


#[derive(Serialize)]
pub struct CreateBidResponse {
    pub bid: Bid,
    pub is_buy_it_now: bool,
    pub auction_ended: bool,
}

#[tracing::instrument(name = "Create bid", skip(json, pool, websocket_service))]
pub async fn create_bid(
    path: web::Path<String>,
    json: web::Json<BidRequest>,
    pool: web::Data<PgPool>,
    websocket_service: web::Data<WebSocketService>,
) -> Result<HttpResponse, BidError> {
    let item_id = path.into_inner();

    let new_bid = json.0.into_new_bid(item_id.clone()).map_err(|e| {
        tracing::error!("Bid validation error: {}", e);
        BidError::ValidationError
    })?;

    // Use the new BidService for advanced business logic
    let result = BidService::create_bid(pool.get_ref(), new_bid.clone()).await?;

    // Get bidder username for WebSocket event
    let bidder_username = sqlx::query_scalar!(
        "SELECT username FROM users WHERE id = $1",
        new_bid.bidder_user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch bidder username")?
    .unwrap_or_else(|| "unknown".to_string());

    let bid = Bid {
        id: result.bid_id,
        item_id: new_bid.item_id.clone(),
        bidder_user_id: new_bid.bidder_user_id,
        bidder_rating: new_bid.bidder_rating,
        time: new_bid.time,
        amount: result.new_current_price.clone(),
        bidder_location: new_bid.bidder_location,
        bidder_country: new_bid.bidder_country,
    };

    let response = CreateBidResponse {
        bid,
        is_buy_it_now: result.is_buy_it_now,
        auction_ended: result.auction_ended,
    };

    // Try to parse item_id as UUID for WebSocket event - if it fails, use a default UUID
    let item_uuid = uuid::Uuid::parse_str(&new_bid.item_id).unwrap_or_else(|_| {
        tracing::warn!(
            "Failed to parse item_id '{}' as UUID, using default for WebSocket event",
            new_bid.item_id
        );
        uuid::Uuid::nil()
    });

    // Get current bid count for this item
    let bid_count = sqlx::query_scalar!(
        "SELECT COUNT(*) as count FROM bids WHERE item_id = $1",
        new_bid.item_id
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to fetch bid count")?
    .unwrap_or(0) as i32;

    // Broadcast WebSocket event for new bid
    let event = if result.auction_ended {
        AuctionEvent::AuctionEnded {
            item_id: item_uuid,
            winner_username: Some(bidder_username),
            winning_bid: Some(result.new_current_price),
            timestamp: Utc::now(),
        }
    } else {
        AuctionEvent::BidPlaced {
            item_id: item_uuid,
            bid_id: result.bid_id,
            bidder_username,
            amount: new_bid.amount,
            current_price: result.new_current_price,
            bid_count,
            timestamp: new_bid.time,
        }
    };

    // Broadcast the event (don't fail if WebSocket broadcast fails)
    if let Err(e) = websocket_service.broadcast_event(event) {
        tracing::warn!("Failed to broadcast WebSocket event: {:?}", e);
    }

    if result.is_buy_it_now {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::Created().json(response))
    }
}

#[tracing::instrument(name = "Update bid", skip(json, pool))]
pub async fn update_bid(
    path: web::Path<Uuid>,
    json: web::Json<BidRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, BidError> {
    let bid_id = path.into_inner();

    // Get the current bid to get the item_id
    let current_bid_row = sqlx::query!(r#"SELECT item_id FROM bids WHERE id = $1"#, bid_id)
        .fetch_optional(pool.get_ref())
        .await
        .context("Failed to fetch current bid")?
        .ok_or(BidError::NotFound)?;

    let updated_bid = json
        .0
        .into_new_bid(current_bid_row.item_id.unwrap_or_default())
        .map_err(|_| BidError::ValidationError)?;

    let rows_affected = sqlx::query!(
        r#"UPDATE bids SET bidder_user_id = $1, bidder_rating = $2, time = $3, amount = $4,
           bidder_location = $5, bidder_country = $6 WHERE id = $7"#,
        updated_bid.bidder_user_id,
        updated_bid.bidder_rating,
        updated_bid.time.naive_utc(),
        updated_bid.amount,
        updated_bid.bidder_location,
        updated_bid.bidder_country,
        bid_id
    )
    .execute(pool.get_ref())
    .await
    .context("Failed to update bid")?
    .rows_affected();

    if rows_affected == 0 {
        return Err(BidError::NotFound);
    }

    let bid = Bid {
        id: bid_id,
        item_id: updated_bid.item_id,
        bidder_user_id: updated_bid.bidder_user_id,
        bidder_rating: updated_bid.bidder_rating,
        time: updated_bid.time,
        amount: updated_bid.amount,
        bidder_location: updated_bid.bidder_location,
        bidder_country: updated_bid.bidder_country,
    };

    Ok(HttpResponse::Ok().json(bid))
}

#[tracing::instrument(name = "Delete bid", skip(pool))]
pub async fn delete_bid(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, BidError> {
    let bid_id = path.into_inner();

    let rows_affected = sqlx::query!(r#"DELETE FROM bids WHERE id = $1"#, bid_id)
        .execute(pool.get_ref())
        .await
        .context("Failed to delete bid")?
        .rows_affected();

    if rows_affected == 0 {
        return Err(BidError::NotFound);
    }

    Ok(HttpResponse::NoContent().finish())
}