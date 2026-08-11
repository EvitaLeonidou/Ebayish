//! src/routes/user_websockets.rs
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::define_route_error;
use crate::jwt_middleware::Claims;
use crate::services::user_websocket_service::UserWebSocketService;

#[derive(Deserialize)]
pub struct UserSubscriptionRequest {
    event_types: Vec<String>,
}

#[derive(Serialize)]
pub struct UserSubscriptionResponse {
    message: String,
    user_id: Uuid,
    event_types: Vec<String>,
}

define_route_error! {
    UserWebSocketError {
        AuthenticationError => (StatusCode::UNAUTHORIZED, "Authentication required"),
        Forbidden => (StatusCode::FORBIDDEN, "Access denied"),
        ServiceError => (StatusCode::INTERNAL_SERVER_ERROR, "WebSocket service error"),
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid subscription request"),
    }
}

#[tracing::instrument(
    name = "Subscribe to user events",
    skip(_user_websocket_service, json, claims)
)]
pub async fn subscribe_to_user_events(
    path: web::Path<Uuid>,
    claims: Claims,
    json: web::Json<UserSubscriptionRequest>,
    _user_websocket_service: web::Data<UserWebSocketService>,
) -> Result<HttpResponse, UserWebSocketError> {
    let target_user_id = path.into_inner();

    let requesting_user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| UserWebSocketError::AuthenticationError)?;

    if requesting_user_id != target_user_id && claims.role != "admin" {
        return Err(UserWebSocketError::Forbidden);
    }

    let valid_event_types = ["listings", "bids"];
    for event_type in &json.event_types {
        if !valid_event_types.contains(&event_type.as_str()) {
            return Err(UserWebSocketError::ValidationError);
        }
    }

    let response = UserSubscriptionResponse {
        message: "Successfully subscribed to user events".to_string(),
        user_id: target_user_id,
        event_types: json.event_types.clone(),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[tracing::instrument(
    name = "Get user WebSocket stats",
    skip(user_websocket_service, claims)
)]
pub async fn get_user_websocket_stats(
    path: web::Path<Uuid>,
    claims: Claims,
    user_websocket_service: web::Data<UserWebSocketService>,
) -> Result<HttpResponse, UserWebSocketError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(UserWebSocketError::Forbidden);
    }

    let user_id = path.into_inner();
    let subscriber_count = user_websocket_service.user_subscriber_count(user_id);

    let response = serde_json::json!({
        "user_id": user_id,
        "subscriber_count": subscriber_count
    });

    Ok(HttpResponse::Ok().json(response))
}
