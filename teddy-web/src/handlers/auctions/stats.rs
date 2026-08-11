use crate::define_route_error;
use teddy_services::auctions::management::{AuctionService, AuctionServiceError};
use actix_web::{HttpResponse, web};
use reqwest::StatusCode;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct AuctionResultResponse {
    pub item_id: String,
    pub seller_user_id: String,
    pub winner_user_id: Option<String>,
    pub winning_amount: Option<String>,
    pub ended_at: String,
    pub total_bids: i32,
}

#[derive(Serialize)]
pub struct AuctionStatsResponse {
    pub active_auctions: i64,
    pub ended_today: i64,
    pub total_bids_today: i64,
}

define_route_error! {
    AuctionError {
        NotFound => (StatusCode::NOT_FOUND, "Auction not found"),
        ServiceError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal auction service error"),
    }
}

impl From<AuctionServiceError> for AuctionError {
    fn from(err: AuctionServiceError) -> Self {
        match err {
            AuctionServiceError::AuctionNotFound => AuctionError::NotFound,
            _ => AuctionError::ServiceError,
        }
    }
}

#[tracing::instrument(name = "Get auction statistics", skip(pool))]
pub async fn get_auction_stats(pool: web::Data<PgPool>) -> Result<HttpResponse, AuctionError> {
    let stats = AuctionService::get_auction_stats(pool.get_ref()).await?;

    let response = AuctionStatsResponse {
        active_auctions: stats.active_auctions,
        ended_today: stats.ended_today,
        total_bids_today: stats.total_bids_today,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[tracing::instrument(name = "Get all auction results", skip(pool))]
pub async fn get_auction_results(pool: web::Data<PgPool>) -> Result<HttpResponse, AuctionError> {
    let results = AuctionService::get_auction_results(pool.get_ref()).await?;

    let response: Vec<AuctionResultResponse> = results
        .into_iter()
        .map(|result| AuctionResultResponse {
            item_id: result.item_id,
            seller_user_id: result.seller_user_id.to_string(),
            winner_user_id: result.winner_user_id.map(|id| id.to_string()),
            winning_amount: result.winning_amount.map(|amt| amt.to_string()),
            ended_at: result.ended_at.to_rfc3339(),
            total_bids: result.total_bids,
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

#[tracing::instrument(name = "Get auction result", skip(pool))]
pub async fn get_auction_result(
    path: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AuctionError> {
    let item_id = path.into_inner();

    let result = AuctionService::get_auction_result(pool.get_ref(), &item_id).await?;

    match result {
        Some(auction_info) => {
            let response = AuctionResultResponse {
                item_id: auction_info.item_id,
                seller_user_id: auction_info.seller_user_id.to_string(),
                winner_user_id: auction_info.winner_user_id.map(|id| id.to_string()),
                winning_amount: auction_info.winning_amount.map(|amt| amt.to_string()),
                ended_at: auction_info.ended_at.to_rfc3339(),
                total_bids: auction_info.total_bids,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        None => Err(AuctionError::NotFound),
    }
}