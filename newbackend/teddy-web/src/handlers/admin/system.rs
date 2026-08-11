// teddy-web/src/handlers/admin/system.rs

use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use teddy_services::AuctionService;
use crate::dto::admin::AuctionResultResponse;
use crate::errors::admin::AuctionError;

#[tracing::instrument(name = "Force end auction", skip(pool))]
pub async fn force_end_auction(
    path: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AuctionError> {
    let item_id = path.into_inner();
    let result = AuctionService::force_end_auction(pool.get_ref(), &item_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to force end auction: {:?}", e);
            AuctionError::UnexpectedError(anyhow::anyhow!("Service error: {}", e))
        })?;

    let response = AuctionResultResponse {
        item_id: result.item_id,
        seller_user_id: result.seller_user_id.to_string(),
        winner_user_id: result.winner_user_id.map(|id| id.to_string()),
        winning_amount: result.winning_amount.map(|amt| amt.to_string()),
        ended_at: result.ended_at.to_rfc3339(),
        total_bids: result.total_bids,
    };
    Ok(HttpResponse::Ok().json(response))
}