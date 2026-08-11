use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Result, web};
use actix_web_actors::ws;
use jsonwebtoken::{DecodingKey, Validation, decode};
use uuid::Uuid;

use crate::define_route_error;
use crate::jwt_middleware::Claims;
use crate::services::websocket_service::{WebSocketActor, WebSocketService};

define_route_error! {
    WebSocketError {
        AuthenticationError => (StatusCode::UNAUTHORIZED, "Authentication required"),
        WebSocketUpgradeError => (StatusCode::BAD_REQUEST, "WebSocket upgrade failed"),
        ServiceError => (StatusCode::INTERNAL_SERVER_ERROR, "WebSocket service error"),
    }
}

fn extract_claims_from_query(req: &HttpRequest) -> Option<Claims> {
    let query_string = req.query_string();
    tracing::info!("WebSocket query string: {}", query_string);

    let token = query_string.split('&').find_map(|param| {
        let mut parts = param.split('=');
        if parts.next() == Some("token") {
            parts.next()
        } else {
            None
        }
    });

    if let Some(token) = token {
        tracing::info!(
            "Found token in query parameters: {}",
            &token[..std::cmp::min(token.len(), 20)]
        );

        let secret = b"opekepescam";
        let validation = Validation::default();

        match decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation) {
            Ok(token_data) => {
                tracing::info!("Successfully decoded JWT from query parameters");
                Some(token_data.claims)
            }
            Err(e) => {
                tracing::warn!("Failed to decode JWT from query parameters: {:?}", e);
                None
            }
        }
    } else {
        tracing::info!("No token found in query parameters");
        None
    }
}

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    websocket_service: web::Data<WebSocketService>,
    claims: Option<Claims>,
) -> Result<HttpResponse, WebSocketError> {
    tracing::info!("WebSocket connection request received");
    tracing::info!("JWT claims from middleware present: {}", claims.is_some());

    let final_claims = claims.or_else(|| {
        tracing::info!("No middleware claims, trying query parameters");
        extract_claims_from_query(&req)
    });

    if let Some(ref claims) = final_claims {
        tracing::info!(
            "Final claims: sub={}, username={}, role={}",
            claims.sub,
            claims.username,
            claims.role
        );
    } else {
        tracing::info!("No JWT claims found - WebSocket will be unauthenticated");
    }

    let (user_id, username) = if let Some(claims) = final_claims {
        let user_uuid = Uuid::parse_str(&claims.sub).ok();
        tracing::info!(
            "Parsed user_id: {:?}, username: {:?}",
            user_uuid,
            claims.username
        );
        (user_uuid, Some(claims.username))
    } else {
        tracing::info!("No valid JWT found - WebSocket will be unauthenticated");
        (None, None)
    };

    let actor = WebSocketActor::new(websocket_service, user_id, username);

    ws::start(actor, &req, stream).map_err(|e| {
        tracing::error!("Failed to start WebSocket connection: {:?}", e);
        WebSocketError::WebSocketUpgradeError
    })
}

pub async fn websocket_stats(
    websocket_service: web::Data<WebSocketService>,
    claims: Claims,
) -> Result<HttpResponse, WebSocketError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(WebSocketError::AuthenticationError);
    }

    let stats = websocket_service.get_stats();
    Ok(HttpResponse::Ok().json(stats))
}

pub async fn item_websocket_stats(
    path: web::Path<Uuid>,
    websocket_service: web::Data<WebSocketService>,
    _claims: Claims,
) -> Result<HttpResponse, WebSocketError> {
    let item_id = path.into_inner();
    let subscriber_count = websocket_service.item_subscriber_count(item_id);

    let response = serde_json::json!({
        "item_id": item_id,
        "subscriber_count": subscriber_count
    });

    Ok(HttpResponse::Ok().json(response))
}
