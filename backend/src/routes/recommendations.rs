#![allow(clippy::await_holding_lock)]

use crate::jwt_middleware::Claims;
use crate::services::RecommendationService;
use actix_web::{HttpResponse, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TrackViewRequest {
    pub item_id: String,
}

#[derive(Serialize)]
pub struct RecommendationResponse {
    pub recommendations: Vec<RecommendationItem>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct RecommendationItem {
    pub item_id: String,
    pub name: String,
    pub price: f32,
    pub currently: Option<f32>,
    pub buy_price: Option<f32>,
    pub listing_type: String,
    pub status: String,
    pub images: Vec<String>,
    pub seller_user_id: Uuid,
    pub location: Option<String>,
    pub ends: Option<chrono::DateTime<chrono::Utc>>,
    pub number_of_bids: Option<i32>,
}

pub async fn get_recommendations(
    path: web::Path<Uuid>,
    query: web::Query<PaginationQuery>,
    db_pool: web::Data<PgPool>,
    recommendation_service: web::Data<std::sync::Mutex<RecommendationService>>,
) -> ActixResult<HttpResponse> {
    let user_id = path.into_inner();
    let limit = query.limit.unwrap_or(10).min(50);

    let service_clone = {
        let service = recommendation_service.lock().map_err(|e| {
            tracing::error!("Failed to acquire recommendation service lock: {}", e);
            actix_web::error::ErrorInternalServerError("Recommendation service unavailable")
        })?;
        service.clone()
    };

    let item_ids = service_clone
        .get_recommendations(&user_id, limit, &db_pool)
        .await;

    if item_ids.is_empty() {
        tracing::info!(
            "No recommendations found for user {}, falling back to latest items",
            user_id
        );
        return get_latest_items(db_pool, limit, Some(user_id)).await;
    }

    let recommendations = fetch_recommended_items(&db_pool, &item_ids)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch recommended items: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch recommendations")
        })?;

    Ok(HttpResponse::Ok().json(RecommendationResponse {
        total: recommendations.len(),
        recommendations,
    }))
}

pub async fn track_category_view(
    req_body: web::Json<TrackViewRequest>,
    db_pool: web::Data<PgPool>,
    claims: Claims,
) -> ActixResult<HttpResponse> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        tracing::error!("Invalid user ID in token: {}", claims.sub);
        actix_web::error::ErrorBadRequest("Invalid user ID")
    })?;
    let item_id = &req_body.item_id;

    RecommendationService::track_category_view(&db_pool, &user_id, item_id)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to track category view for user {} item {}: {}",
                user_id,
                item_id,
                e
            );
            actix_web::error::ErrorInternalServerError("Failed to track view")
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({"success": true})))
}

pub async fn retrain_model(
    db_pool: web::Data<PgPool>,
    recommendation_service: web::Data<std::sync::Mutex<RecommendationService>>,
    _claims: Claims, // Admin only - add admin check if needed
) -> ActixResult<HttpResponse> {
    let mut service = recommendation_service.lock().map_err(|e| {
        tracing::error!("Failed to acquire recommendation service lock: {}", e);
        actix_web::error::ErrorInternalServerError("Recommendation service unavailable")
    })?;

    service.train_model(&db_pool).await.map_err(|e| {
        tracing::error!("Failed to retrain recommendation model: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to retrain model")
    })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Model retrained successfully"
    })))
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    limit: Option<usize>,
}

async fn fetch_recommended_items(
    db_pool: &PgPool,
    item_ids: &[String],
) -> Result<Vec<RecommendationItem>, sqlx::Error> {
    if item_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = sqlx::query!(
        r#"
        SELECT
            i.item_id,
            i.name,
            COALESCE(i.buy_price, i.currently, i.price) as display_price,
            i.currently,
            i.buy_price,
            COALESCE(i.listing_type, 'auction') as listing_type,
            COALESCE(i.status, 'active') as status,
            i.seller_user_id,
            i.location,
            i.ends,
            i.number_of_bids,
            COALESCE(
                ARRAY_AGG(
                    CONCAT('/api/uploads/items/', i.item_id, '/', img.filename)
                    ORDER BY img.display_order
                ) FILTER (WHERE img.filename IS NOT NULL),
                '{}'
            ) as images
        FROM items i
        LEFT JOIN item_images img ON i.item_id = img.item_id
        WHERE i.item_id = ANY($1)
        AND i.status NOT IN ('sold', 'ended')
        GROUP BY i.item_id, i.name, i.currently, i.buy_price, i.price,
                 i.listing_type, i.status, i.seller_user_id, i.location,
                 i.ends, i.number_of_bids
        ORDER BY array_position($1, i.item_id)
        "#,
        item_ids
    )
    .fetch_all(db_pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| RecommendationItem {
            item_id: row.item_id,
            name: row.name,
            price: row
                .display_price
                .map(|p| p.to_string().parse::<f32>().unwrap_or(0.0))
                .unwrap_or(0.0),
            currently: row
                .currently
                .map(|c| c.to_string().parse::<f32>().unwrap_or(0.0)),
            buy_price: row
                .buy_price
                .map(|bp| bp.to_string().parse::<f32>().unwrap_or(0.0)),
            listing_type: row.listing_type.unwrap_or_else(|| "auction".to_string()),
            status: row.status.unwrap_or_else(|| "active".to_string()),
            seller_user_id: row.seller_user_id.unwrap_or_default(),
            location: row.location,
            ends: row.ends.map(|dt| dt.and_utc()),
            number_of_bids: row.number_of_bids,
            images: row.images.unwrap_or_default(),
        })
        .collect())
}

async fn get_latest_items(
    db_pool: web::Data<PgPool>,
    limit: usize,
    exclude_user_id: Option<Uuid>,
) -> ActixResult<HttpResponse> {
    let rows = sqlx::query!(
        r#"
        SELECT
            i.item_id,
            i.name,
            COALESCE(i.buy_price, i.currently, i.price) as display_price,
            i.currently,
            i.buy_price,
            COALESCE(i.listing_type, 'auction') as listing_type,
            COALESCE(i.status, 'active') as status,
            i.seller_user_id,
            i.location,
            i.ends,
            i.number_of_bids,
            COALESCE(
                ARRAY_AGG(
                    CONCAT('/api/uploads/items/', i.item_id, '/', img.filename)
                    ORDER BY img.display_order
                ) FILTER (WHERE img.filename IS NOT NULL),
                '{}'
            ) as images
        FROM items i
        LEFT JOIN item_images img ON i.item_id = img.item_id
        WHERE i.status NOT IN ('sold', 'ended')
        AND ($2::uuid IS NULL OR i.seller_user_id != $2)
        GROUP BY i.item_id, i.name, i.currently, i.buy_price, i.price,
                 i.listing_type, i.status, i.seller_user_id, i.location,
                 i.ends, i.number_of_bids, i.created_at
        ORDER BY i.created_at DESC
        LIMIT $1
        "#,
        limit as i64,
        exclude_user_id
    )
    .fetch_all(db_pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch latest items: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch latest items")
    })?;

    let recommendations: Vec<RecommendationItem> = rows
        .into_iter()
        .map(|row| RecommendationItem {
            item_id: row.item_id,
            name: row.name,
            price: row
                .display_price
                .map(|p| p.to_string().parse::<f32>().unwrap_or(0.0))
                .unwrap_or(0.0),
            currently: row
                .currently
                .map(|c| c.to_string().parse::<f32>().unwrap_or(0.0)),
            buy_price: row
                .buy_price
                .map(|bp| bp.to_string().parse::<f32>().unwrap_or(0.0)),
            listing_type: row.listing_type.unwrap_or_else(|| "auction".to_string()),
            status: row.status.unwrap_or_else(|| "active".to_string()),
            seller_user_id: row.seller_user_id.unwrap_or_default(),
            location: row.location,
            ends: row.ends.map(|dt| dt.and_utc()),
            number_of_bids: row.number_of_bids,
            images: row.images.unwrap_or_default(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(RecommendationResponse {
        total: recommendations.len(),
        recommendations,
    }))
}
