use actix_web::{HttpResponse, web};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::errors::bidding::BidError;

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