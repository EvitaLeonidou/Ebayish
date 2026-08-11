#![allow(clippy::collapsible_if)]

use crate::define_route_error;
use crate::domain::NewBid;
use crate::domain::Notification;
use crate::jwt_middleware::Claims;
use crate::services::{
    BidService, NotificationService, WebSocketService, bid_service::BidServiceError,
    websocket_service::AuctionEvent,
};
use actix_web::{HttpResponse, web};
use anyhow::Context;
use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

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

#[derive(sqlx::FromRow)]
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

define_route_error! {
    BidError {
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid bid data provided"),
        NotFound => (StatusCode::NOT_FOUND, "Bid not found"),
        ItemNotFound => (StatusCode::NOT_FOUND, "Item not found"),
        BidTooLow => (StatusCode::BAD_REQUEST, "Bid amount must be higher than current price"),
        AuctionEnded => (StatusCode::BAD_REQUEST, "Auction has ended"),
        SelfBidding => (StatusCode::BAD_REQUEST, "Cannot bid on your own item"),
        BuyItNowTriggered => (StatusCode::OK, "Buy-it-now triggered, auction ended"),
        ServiceError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal service error"),
    }
}

impl From<BidServiceError> for BidError {
    fn from(err: BidServiceError) -> Self {
        match err {
            BidServiceError::ItemNotFound => BidError::ItemNotFound,
            BidServiceError::BidTooLow { .. } => BidError::BidTooLow,
            BidServiceError::AuctionEnded => BidError::AuctionEnded,
            BidServiceError::SelfBidding => BidError::SelfBidding,
            BidServiceError::BuyItNowTriggered => BidError::BuyItNowTriggered,
            _ => BidError::ServiceError,
        }
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
    claims: Claims,
    pool: web::Data<PgPool>,
    websocket_service: web::Data<WebSocketService>,
) -> Result<HttpResponse, BidError> {
    let item_id = path.into_inner();

    let requesting_user_id =
        uuid::Uuid::parse_str(&claims.sub).map_err(|_| BidError::ValidationError)?;

    if requesting_user_id != json.bidder_user_id {
        return Err(BidError::ValidationError);
    }

    let new_bid = json.0.into_new_bid(item_id.clone()).map_err(|e| {
        tracing::error!("Bid validation error: {}", e);
        BidError::ValidationError
    })?;

    let result = BidService::create_bid(pool.get_ref(), new_bid.clone()).await?;

    let item_info = sqlx::query!(
        "SELECT name, seller_user_id FROM items WHERE item_id = $1",
        new_bid.item_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch item info")?;

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

    let item_uuid = uuid::Uuid::parse_str(&new_bid.item_id).unwrap_or_else(|_| {
        tracing::warn!(
            "Failed to parse item_id '{}' as UUID, using default for WebSocket event",
            new_bid.item_id
        );
        uuid::Uuid::nil()
    });

    let bid_count = sqlx::query_scalar!(
        "SELECT COUNT(*) as count FROM bids WHERE item_id = $1",
        new_bid.item_id
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to fetch bid count")?
    .unwrap_or(0) as i32;

    //notify seller of the bid
    if let Some(item) = &item_info {
        if let Some(seller_id) = item.seller_user_id {
            let notification = Notification::new_bid_received(
                seller_id,
                new_bid.item_id.clone(),
                &bidder_username,
                result.new_current_price.clone(),
                &item.name,
                bid_count,
            );

            if let Err(e) = NotificationService::create_notification(
                pool.get_ref(),
                &notification,
                websocket_service.get_ref(),
            )
            .await
            {
                tracing::warn!("Failed to create bid notification: {}", e);
            }
        }
    }

    if !result.auction_ended {
        //notify bidder who got outbid by this bid
        let previous_highest_bidder = sqlx::query!(
            "SELECT bidder_user_id, amount FROM bids
             WHERE item_id = $1 AND bidder_user_id != $2 AND amount < $3
             ORDER BY amount DESC, time DESC LIMIT 1",
            new_bid.item_id,
            new_bid.bidder_user_id,
            result.new_current_price
        )
        .fetch_optional(pool.get_ref())
        .await
        .context("Failed to fetch previous highest bidder")?;

        if let Some(prev_bid) = previous_highest_bidder {
            if let Some(item) = &item_info {
                tracing::info!(
                    "Creating outbid notification for user {} who bid {} and was outbid by {} with {}",
                    prev_bid.bidder_user_id.unwrap_or_default(),
                    prev_bid.amount,
                    bidder_username,
                    result.new_current_price
                );

                let notification = Notification::new_bid_outbid(
                    prev_bid.bidder_user_id.unwrap_or_default(),
                    new_bid.item_id.clone(),
                    &item.name,
                    prev_bid.amount,
                    result.new_current_price.clone(),
                    &bidder_username,
                );

                if let Err(e) = NotificationService::create_notification(
                    pool.get_ref(),
                    &notification,
                    websocket_service.get_ref(),
                )
                .await
                {
                    tracing::warn!("Failed to create outbid notification: {}", e);
                } else {
                    tracing::info!("Successfully created outbid notification");
                }
            }
        } else {
            tracing::info!("No previous bidder found to notify about being outbid");
        }
    }

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

    if let Err(e) = websocket_service.broadcast_event(event) {
        tracing::warn!("Failed to broadcast WebSocket event: {:?}", e);
    }

    if result.is_buy_it_now {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::Created().json(response))
    }
}

#[tracing::instrument(name = "Get all bids", skip(pool))]
pub async fn get_bids(pool: web::Data<PgPool>) -> Result<HttpResponse, BidError> {
    let bid_rows = sqlx::query_as::<_, BidRow>(
        r#"SELECT id, item_id, bidder_user_id, bidder_rating, time, amount, bidder_location, bidder_country
           FROM bids ORDER BY time DESC"#,
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch bids")?;

    let bids: Vec<Bid> = bid_rows
        .into_iter()
        .map(|row| Bid {
            id: row.id,
            item_id: row.item_id,
            bidder_user_id: row.bidder_user_id,
            bidder_rating: row.bidder_rating,
            time: row.time.and_utc(),
            amount: row.amount,
            bidder_location: row.bidder_location,
            bidder_country: row.bidder_country,
        })
        .collect();

    Ok(HttpResponse::Ok().json(bids))
}

#[tracing::instrument(name = "Get bids for item", skip(pool))]
pub async fn get_bids_for_item(
    path: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, BidError> {
    let item_id = path.into_inner();

    let bid_rows = sqlx::query_as::<_, BidRow>(
        r#"SELECT id, item_id, bidder_user_id, bidder_rating, time, amount, bidder_location, bidder_country
           FROM bids WHERE item_id = $1 ORDER BY time DESC"#,
    )
    .bind(item_id)
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch bids for item")?;

    let bids: Vec<Bid> = bid_rows
        .into_iter()
        .map(|row| Bid {
            id: row.id,
            item_id: row.item_id,
            bidder_user_id: row.bidder_user_id,
            bidder_rating: row.bidder_rating,
            time: row.time.and_utc(),
            amount: row.amount,
            bidder_location: row.bidder_location,
            bidder_country: row.bidder_country,
        })
        .collect();

    Ok(HttpResponse::Ok().json(bids))
}

#[tracing::instrument(name = "Get bid by ID", skip(pool))]
pub async fn get_bid(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, BidError> {
    let bid_id = path.into_inner();

    let bid_row = sqlx::query_as::<_, BidRow>(
        r#"SELECT id, item_id, bidder_user_id, bidder_rating, time, amount, bidder_location, bidder_country
           FROM bids WHERE id = $1"#,
    )
    .bind(bid_id)
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch bid")?;

    let bid = bid_row.map(|row| Bid {
        id: row.id,
        item_id: row.item_id,
        bidder_user_id: row.bidder_user_id,
        bidder_rating: row.bidder_rating,
        time: row.time.and_utc(),
        amount: row.amount,
        bidder_location: row.bidder_location,
        bidder_country: row.bidder_country,
    });

    match bid {
        Some(bid) => Ok(HttpResponse::Ok().json(bid)),
        None => Err(BidError::NotFound),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_amount_integer() {
        let json_data = json!({
            "bidder_user_id": "550e8400-e29b-41d4-a716-446655440000",
            "time": "2023-01-01T00:00:00Z",
            "amount": 15
        });

        let bid_request: BidRequest =
            serde_json::from_value(json_data).expect("Failed to deserialize");
        assert_eq!(bid_request.amount, BigDecimal::from(15));
    }

    #[test]
    fn test_deserialize_amount_string() {
        let json_data = json!({
            "bidder_user_id": "550e8400-e29b-41d4-a716-446655440000",
            "time": "2023-01-01T00:00:00Z",
            "amount": "15.50"
        });

        let bid_request: BidRequest =
            serde_json::from_value(json_data).expect("Failed to deserialize");
        assert_eq!(bid_request.amount, BigDecimal::from_str("15.50").unwrap());
    }
}
