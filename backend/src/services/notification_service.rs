use anyhow::Context;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::domain::{
    Notification, NotificationFilters, NotificationSummary, NotificationType, NotificationTypeCount,
};
use crate::services::WebSocketService;

#[derive(thiserror::Error, Debug)]
pub enum NotificationServiceError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Notification not found")]
    NotFound,
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct NotificationService;

impl NotificationService {
    pub async fn create_notification(
        pool: &PgPool,
        notification: &Notification,
        websocket_service: &WebSocketService,
    ) -> Result<(), NotificationServiceError> {
        sqlx::query!(
            r#"
            INSERT INTO notifications (
                id, user_id, notification_type, title, message,
                item_id, related_user_id, amount, is_read, created_at, read_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            notification.id,
            notification.user_id,
            serde_json::to_string(&notification.notification_type)?,
            notification.title,
            notification.message,
            notification.item_id,
            notification.related_user_id,
            notification.amount,
            notification.is_read,
            notification.created_at,
            notification.read_at
        )
        .execute(pool)
        .await
        .context("Failed to insert notification")?;

        //broadcast notification via websocket
        let notification_type_str = serde_json::to_string(&notification.notification_type)?;
        tracing::info!(
            "Broadcasting notification to user {}: {} (type: {})",
            notification.user_id,
            notification.title,
            notification_type_str
        );
        tracing::info!("Using WebSocket service: {:p}", websocket_service);

        match websocket_service.broadcast_notification_to_user(
            notification.user_id,
            notification.id,
            notification.title.clone(),
            notification.message.clone(),
            notification_type_str,
            notification.item_id.clone(),
        ) {
            Ok(_) => {
                tracing::info!("Successfully broadcasted notification via WebSocket");
            }
            Err(e) => {
                tracing::error!("Failed to broadcast notification via WebSocket: {:?}", e);
            }
        }

        Ok(())
    }

    pub async fn get_user_notifications(
        pool: &PgPool,
        user_id: Uuid,
        filters: NotificationFilters,
    ) -> Result<Vec<Notification>, NotificationServiceError> {
        let limit = filters.limit.unwrap_or(50).min(100);
        let offset = filters.offset.unwrap_or(0);

        let mut query = sqlx::QueryBuilder::new(
            "SELECT id, user_id, notification_type, title, message, item_id,
             related_user_id, amount, is_read, created_at, read_at FROM notifications WHERE user_id = "
        );
        query.push_bind(user_id);

        if let Some(notification_type) = filters.notification_type {
            query.push(" AND notification_type = ");
            query.push_bind(serde_json::to_string(&notification_type)?);
        }

        if let Some(is_read) = filters.is_read {
            query.push(" AND is_read = ");
            query.push_bind(is_read);
        }

        query.push(" ORDER BY created_at DESC LIMIT ");
        query.push_bind(limit);
        query.push(" OFFSET ");
        query.push_bind(offset);

        let rows = query.build().fetch_all(pool).await?;

        let mut notifications = Vec::new();
        for row in rows {
            let notification_type: NotificationType =
                serde_json::from_str(&row.get::<String, _>("notification_type"))?;

            let notification = Notification {
                id: row.get("id"),
                user_id: row.get("user_id"),
                notification_type,
                title: row.get("title"),
                message: row.get("message"),
                item_id: row.get("item_id"),
                related_user_id: row.get("related_user_id"),
                amount: row.get("amount"),
                is_read: row.get("is_read"),
                created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
                read_at: row.get("read_at"),
            };
            notifications.push(notification);
        }

        Ok(notifications)
    }

    pub async fn mark_as_read(
        pool: &PgPool,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), NotificationServiceError> {
        let result = sqlx::query!(
            "UPDATE notifications SET is_read = true, read_at = NOW()
             WHERE id = $1 AND user_id = $2 AND is_read = false",
            notification_id,
            user_id
        )
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(NotificationServiceError::NotFound);
        }

        Ok(())
    }

    pub async fn mark_all_as_read(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<u64, NotificationServiceError> {
        let result = sqlx::query!(
            "UPDATE notifications SET is_read = true, read_at = NOW()
             WHERE user_id = $1 AND is_read = false",
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn get_notification_summary(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<NotificationSummary, NotificationServiceError> {
        let counts = sqlx::query!(
            "SELECT
                COUNT(*) as total_count,
                COUNT(*) FILTER (WHERE is_read = false) as unread_count
             FROM notifications WHERE user_id = $1",
            user_id
        )
        .fetch_one(pool)
        .await?;

        let type_counts = sqlx::query!(
            "SELECT
                notification_type,
                COUNT(*) as count,
                COUNT(*) FILTER (WHERE is_read = false) as unread_count
             FROM notifications
             WHERE user_id = $1
             GROUP BY notification_type",
            user_id
        )
        .fetch_all(pool)
        .await?;

        let mut notification_types = Vec::new();
        for row in type_counts {
            let notification_type: NotificationType = serde_json::from_str(&row.notification_type)?;

            notification_types.push(NotificationTypeCount {
                notification_type,
                count: row.count.unwrap_or(0),
                unread_count: row.unread_count.unwrap_or(0),
            });
        }

        Ok(NotificationSummary {
            total_count: counts.total_count.unwrap_or(0),
            unread_count: counts.unread_count.unwrap_or(0),
            notification_types,
        })
    }

    pub async fn cleanup_old_notifications(
        pool: &PgPool,
        days_to_keep: i32,
    ) -> Result<u64, NotificationServiceError> {
        let result = sqlx::query!(
            "DELETE FROM notifications
             WHERE created_at < NOW() - make_interval(days => $1)",
            days_to_keep
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn delete_notification(
        pool: &PgPool,
        notification_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), NotificationServiceError> {
        let result = sqlx::query!(
            "DELETE FROM notifications WHERE id = $1 AND user_id = $2",
            notification_id,
            user_id
        )
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(NotificationServiceError::NotFound);
        }

        Ok(())
    }

    pub async fn delete_all_notifications(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<u64, NotificationServiceError> {
        let result = sqlx::query!("DELETE FROM notifications WHERE user_id = $1", user_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }
}
