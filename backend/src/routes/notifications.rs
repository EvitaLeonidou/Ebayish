use actix_web::{HttpResponse, web};
use anyhow::Context;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::define_route_error;
use crate::domain::{NotificationFilters, NotificationType};
use crate::jwt_middleware::Claims;
use crate::services::NotificationService;
use sqlx::PgPool;

define_route_error! {
    NotificationError {
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid notification data provided"),
        NotFound => (StatusCode::NOT_FOUND, "Notification not found"),
        Unauthorized => (StatusCode::FORBIDDEN, "Not authorized to access this notification"),
    }
}

#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    #[serde(rename = "type")]
    notification_type: Option<NotificationType>,
    is_read: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct MarkAsReadResponse {
    marked_count: usize,
    success: bool,
}

#[tracing::instrument(name = "Get user notifications", skip(pool, claims))]
pub async fn get_notifications(
    query: web::Query<NotificationQuery>,
    pool: web::Data<PgPool>,
    claims: Claims,
) -> Result<HttpResponse, NotificationError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| NotificationError::ValidationError)?;

    let filters = NotificationFilters {
        notification_type: query.notification_type.clone(),
        is_read: query.is_read,
        limit: query.limit,
        offset: query.offset,
    };

    let notifications =
        NotificationService::get_user_notifications(pool.get_ref(), user_id, filters)
            .await
            .context("Failed to fetch notifications")
            .map_err(|_| NotificationError::ValidationError)?;

    Ok(HttpResponse::Ok().json(notifications))
}

#[tracing::instrument(name = "Get notification summary", skip(pool, claims))]
pub async fn get_notification_summary(
    pool: web::Data<PgPool>,
    claims: Claims,
) -> Result<HttpResponse, NotificationError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| NotificationError::ValidationError)?;

    let summary = NotificationService::get_notification_summary(pool.get_ref(), user_id)
        .await
        .context("Failed to fetch notification summary")
        .map_err(|_| NotificationError::ValidationError)?;

    Ok(HttpResponse::Ok().json(summary))
}

#[tracing::instrument(name = "Mark notification as read", skip(pool, claims))]
pub async fn mark_as_read(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    claims: Claims,
) -> Result<HttpResponse, NotificationError> {
    let notification_id = path.into_inner();
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| NotificationError::ValidationError)?;

    NotificationService::mark_as_read(pool.get_ref(), notification_id, user_id)
        .await
        .context("Failed to mark notification as read")
        .map_err(|e| {
            if e.to_string().contains("not found") {
                NotificationError::NotFound
            } else {
                NotificationError::ValidationError
            }
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Notification marked as read"
    })))
}

#[tracing::instrument(name = "Mark all notifications as read", skip(pool, claims))]
pub async fn mark_all_as_read(
    pool: web::Data<PgPool>,
    claims: Claims,
) -> Result<HttpResponse, NotificationError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| NotificationError::ValidationError)?;

    let marked_count = NotificationService::mark_all_as_read(pool.get_ref(), user_id)
        .await
        .context("Failed to mark all notifications as read")
        .map_err(|_| NotificationError::ValidationError)?;

    Ok(HttpResponse::Ok().json(MarkAsReadResponse {
        marked_count: marked_count as usize,
        success: true,
    }))
}

#[tracing::instrument(name = "Delete notification", skip(pool, claims))]
pub async fn delete_notification(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
    claims: Claims,
) -> Result<HttpResponse, NotificationError> {
    let notification_id = path.into_inner();
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| NotificationError::ValidationError)?;

    NotificationService::delete_notification(pool.get_ref(), notification_id, user_id)
        .await
        .context("Failed to delete notification")
        .map_err(|e| {
            if e.to_string().contains("not found") {
                NotificationError::NotFound
            } else {
                NotificationError::ValidationError
            }
        })?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Notification deleted"
    })))
}

#[tracing::instrument(name = "Delete all notifications", skip(pool, claims))]
pub async fn delete_all_notifications(
    pool: web::Data<PgPool>,
    claims: Claims,
) -> Result<HttpResponse, NotificationError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| NotificationError::ValidationError)?;

    let deleted_count = NotificationService::delete_all_notifications(pool.get_ref(), user_id)
        .await
        .context("Failed to delete all notifications")
        .map_err(|_| NotificationError::ValidationError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "All notifications deleted",
        "deleted_count": deleted_count
    })))
}
