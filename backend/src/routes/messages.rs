use crate::domain::{CreateChatRoomRequest, NewMessage};
use crate::jwt_middleware::Claims;
use crate::services::MessageService;
use actix_web::{HttpResponse, Result, web};
use uuid::Uuid;

pub async fn create_chat_room(
    claims: Claims,
    request: web::Json<CreateChatRoomRequest>,
    message_service: web::Data<MessageService>,
) -> Result<HttpResponse> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID"))?;

    match message_service
        .create_or_get_chat_room(user_id, request.into_inner())
        .await
    {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(err) => {
            tracing::error!("Failed to create chat room: {:?}", err);
            match err {
                crate::services::message_service::MessageServiceError::SelfMessage => {
                    Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "You cannot message yourself"
                    })))
                }
                crate::services::message_service::MessageServiceError::ValidationError(msg) => {
                    Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "error": msg
                    })))
                }
                _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to create chat room"
                }))),
            }
        }
    }
}

pub async fn get_messages(
    claims: Claims,
    path: web::Path<Uuid>,
    message_service: web::Data<MessageService>,
) -> Result<HttpResponse> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID"))?;
    let chat_room_id = path.into_inner();

    match message_service.get_messages(user_id, chat_room_id).await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(err) => {
            tracing::error!("Failed to get messages: {:?}", err);
            match err {
                crate::services::message_service::MessageServiceError::ChatRoomNotFound => {
                    Ok(HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Chat room not found"
                    })))
                }
                crate::services::message_service::MessageServiceError::Unauthorized => {
                    Ok(HttpResponse::Forbidden().json(serde_json::json!({
                        "error": "You don't have access to this chat room"
                    })))
                }
                _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to get messages"
                }))),
            }
        }
    }
}

pub async fn send_message(
    claims: Claims,
    path: web::Path<Uuid>,
    new_message: web::Json<NewMessage>,
    message_service: web::Data<MessageService>,
) -> Result<HttpResponse> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID"))?;
    let chat_room_id = path.into_inner();

    match message_service
        .send_message(user_id, chat_room_id, new_message.into_inner())
        .await
    {
        Ok(message) => Ok(HttpResponse::Created().json(message)),
        Err(err) => {
            tracing::error!("Failed to send message: {:?}", err);
            match err {
                crate::services::message_service::MessageServiceError::ChatRoomNotFound => {
                    Ok(HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Chat room not found"
                    })))
                }
                crate::services::message_service::MessageServiceError::Unauthorized => {
                    Ok(HttpResponse::Forbidden().json(serde_json::json!({
                        "error": "You don't have access to this chat room"
                    })))
                }
                crate::services::message_service::MessageServiceError::ValidationError(msg) => {
                    Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "error": msg
                    })))
                }
                _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to send message"
                }))),
            }
        }
    }
}

pub async fn delete_message(
    claims: Claims,
    path: web::Path<(Uuid, Uuid)>,
    message_service: web::Data<MessageService>,
) -> Result<HttpResponse> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID"))?;
    let (chat_room_id, message_id) = path.into_inner();

    match message_service
        .delete_message(user_id, chat_room_id, message_id)
        .await
    {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(err) => {
            tracing::error!("Failed to delete message: {:?}", err);
            match err {
                crate::services::message_service::MessageServiceError::ChatRoomNotFound => {
                    Ok(HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Chat room not found"
                    })))
                }
                crate::services::message_service::MessageServiceError::MessageNotFound => {
                    Ok(HttpResponse::NotFound().json(serde_json::json!({
                        "error": "Message not found or you don't have permission to delete it"
                    })))
                }
                crate::services::message_service::MessageServiceError::Unauthorized => {
                    Ok(HttpResponse::Forbidden().json(serde_json::json!({
                        "error": "You don't have access to this chat room"
                    })))
                }
                _ => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to delete message"
                }))),
            }
        }
    }
}

pub async fn get_connections(
    claims: Claims,
    message_service: web::Data<MessageService>,
) -> Result<HttpResponse> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid user ID"))?;

    match message_service.get_connections(user_id).await {
        Ok(connections) => Ok(HttpResponse::Ok().json(connections)),
        Err(err) => {
            tracing::error!("Failed to get connections: {:?}", err);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get connections"
            })))
        }
    }
}
