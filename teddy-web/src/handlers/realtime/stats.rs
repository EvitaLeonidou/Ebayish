use actix_web::http::StatusCode;
use actix_web::{HttpResponse, web};
use uuid::Uuid;

use crate::define_route_error;
use crate::middleware::jwt::Claims;
use teddy_services::WebSocketService;

define_route_error! {
    WebSocketStatsError {
        AuthenticationError => (StatusCode::UNAUTHORIZED, "Authentication required"),
        ServiceError => (StatusCode::INTERNAL_SERVER_ERROR, "WebSocket service error"),
    }
}

/// Get WebSocket service statistics (admin only)
pub async fn websocket_stats(
    websocket_service: web::Data<WebSocketService>,
    claims: Claims,
) -> Result<HttpResponse, WebSocketStatsError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(WebSocketStatsError::AuthenticationError);
    }

    let stats = websocket_service.get_stats();
    Ok(HttpResponse::Ok().json(stats))
}

/// Get item-specific subscription statistics
pub async fn item_websocket_stats(
    path: web::Path<Uuid>,
    websocket_service: web::Data<WebSocketService>,
    _claims: Claims, // Require authentication but allow any role
) -> Result<HttpResponse, WebSocketStatsError> {
    let item_id = path.into_inner();
    let subscriber_count = websocket_service.item_subscriber_count(item_id);

    let response = serde_json::json!({
        "item_id": item_id,
        "subscriber_count": subscriber_count
    });

    Ok(HttpResponse::Ok().json(response))
}