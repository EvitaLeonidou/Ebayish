use crate::domain::{
    ChatRoom, ConnectedUser, Connection, CreateChatRoomRequest, CreateChatRoomResponse,
    MessageWithUser, MessagesResponse, NewMessage, Notification,
};
use crate::error_handling::error_chain_fmt;
use crate::services::websocket_service::AuctionEvent;
use crate::services::{NotificationService, WebSocketService};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum MessageServiceError {
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Chat room not found")]
    ChatRoomNotFound,
    #[error("Message not found")]
    MessageNotFound,
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Users cannot message themselves")]
    SelfMessage,
    #[error("Message validation failed: {0}")]
    ValidationError(String),
    #[error("Unexpected error")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for MessageServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Clone)]
pub struct MessageService {
    pool: PgPool,
    websocket_service: WebSocketService,
}

impl MessageService {
    pub fn new(pool: PgPool, websocket_service: WebSocketService) -> Self {
        Self {
            pool,
            websocket_service,
        }
    }

    pub async fn create_or_get_chat_room(
        &self,
        current_user_id: Uuid,
        request: CreateChatRoomRequest,
    ) -> Result<CreateChatRoomResponse, MessageServiceError> {
        if current_user_id == request.other_user_id {
            return Err(MessageServiceError::SelfMessage);
        }

        let (user1_id, user2_id) = if current_user_id < request.other_user_id {
            (current_user_id, request.other_user_id)
        } else {
            (request.other_user_id, current_user_id)
        };

        let existing_room = sqlx::query_as!(
            ChatRoom,
            r#"
            SELECT id, user1_id, user2_id, created_at, updated_at
            FROM chat_rooms
            WHERE user1_id = $1 AND user2_id = $2
            "#,
            user1_id,
            user2_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(room) = existing_room {
            return Ok(CreateChatRoomResponse { room_id: room.id });
        }

        let users = sqlx::query!(
            "SELECT id, username FROM users WHERE id = $1 OR id = $2",
            current_user_id,
            request.other_user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let current_username = users
            .iter()
            .find(|u| u.id == current_user_id)
            .map(|u| u.username.as_str())
            .ok_or_else(|| {
                MessageServiceError::UnexpectedError(anyhow::anyhow!("Current user not found"))
            })?;

        let room_id = Uuid::new_v4();
        let now = Utc::now().naive_utc();

        sqlx::query!(
            r#"
            INSERT INTO chat_rooms (id, user1_id, user2_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            room_id,
            user1_id,
            user2_id,
            now,
            now
        )
        .execute(&self.pool)
        .await?;

        let notification = Notification::new_chat_room(
            request.other_user_id,
            current_username,
            current_user_id,
            room_id,
        );

        if let Err(e) = NotificationService::create_notification(
            &self.pool,
            &notification,
            &self.websocket_service,
        )
        .await
        {
            tracing::warn!("Failed to create chat room notification: {:?}", e);
        }

        let event = AuctionEvent::ChatRoomCreated {
            chat_room_id: room_id,
            other_user_id: current_user_id,
            other_username: current_username.to_string(),
            timestamp: DateTime::from_naive_utc_and_offset(now, Utc),
        };

        if let Err(e) = self.websocket_service.broadcast_event(event) {
            tracing::warn!("Failed to broadcast chat room creation event: {:?}", e);
        }

        Ok(CreateChatRoomResponse { room_id })
    }

    pub async fn get_messages(
        &self,
        current_user_id: Uuid,
        chat_room_id: Uuid,
    ) -> Result<MessagesResponse, MessageServiceError> {
        let chat_room = self.get_chat_room_by_id(chat_room_id).await?;
        if !chat_room.contains_user(current_user_id) {
            return Err(MessageServiceError::Unauthorized);
        }

        let messages = sqlx::query!(
            r#"
            SELECT m.id, m.chat_room_id, m.content, m.created_at, m.sender_id,
                   u.username
            FROM messages m
            JOIN users u ON m.sender_id = u.id
            WHERE m.chat_room_id = $1
            ORDER BY m.created_at ASC
            "#,
            chat_room_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| MessageWithUser {
            id: row.id,
            chat_room_id: row.chat_room_id,
            content: row.content,
            created_at: DateTime::from_naive_utc_and_offset(row.created_at.unwrap(), Utc),
            user: crate::domain::MessageUser {
                id: row.sender_id,
                username: row.username,
            },
        })
        .collect();

        Ok(MessagesResponse { messages })
    }

    pub async fn send_message(
        &self,
        current_user_id: Uuid,
        chat_room_id: Uuid,
        new_message: NewMessage,
    ) -> Result<MessageWithUser, MessageServiceError> {
        let validated_message = new_message
            .parse()
            .map_err(MessageServiceError::ValidationError)?;

        let chat_room = self.get_chat_room_by_id(chat_room_id).await?;
        if !chat_room.contains_user(current_user_id) {
            return Err(MessageServiceError::Unauthorized);
        }

        let username = sqlx::query!("SELECT username FROM users WHERE id = $1", current_user_id)
            .fetch_one(&self.pool)
            .await?
            .username;

        let message_id = Uuid::new_v4();
        let created_at_utc = Utc::now();
        let created_at = created_at_utc.naive_utc();

        sqlx::query!(
            r#"
            INSERT INTO messages (id, chat_room_id, sender_id, content, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            message_id,
            chat_room_id,
            current_user_id,
            validated_message.content,
            created_at
        )
        .execute(&self.pool)
        .await?;

        sqlx::query!(
            "UPDATE chat_rooms SET updated_at = $1 WHERE id = $2",
            created_at,
            chat_room_id
        )
        .execute(&self.pool)
        .await?;

        let other_user_id = chat_room.get_other_user_id(current_user_id);

        let notification = Notification::new_message(
            other_user_id,
            &username,
            current_user_id,
            chat_room_id,
            &validated_message.content,
        );

        //sends notification to database and websocket with special handling for message notifications
        if let Err(e) = self
            .create_message_notification(&notification, chat_room_id)
            .await
        {
            tracing::warn!("Failed to create message notification: {:?}", e);
        }

        //broadcast the new message via webocket
        let message_event = AuctionEvent::NewMessage {
            chat_room_id,
            message_id,
            sender_username: username.clone(),
            content: validated_message.content.clone(),
            timestamp: created_at_utc,
        };

        if let Err(e) = self.websocket_service.broadcast_event(message_event) {
            tracing::warn!("Failed to broadcast new message event: {:?}", e);
        }

        Ok(MessageWithUser {
            id: message_id,
            chat_room_id,
            content: validated_message.content,
            created_at: created_at_utc,
            user: crate::domain::MessageUser {
                id: current_user_id,
                username,
            },
        })
    }

    pub async fn delete_message(
        &self,
        current_user_id: Uuid,
        chat_room_id: Uuid,
        message_id: Uuid,
    ) -> Result<(), MessageServiceError> {
        let chat_room = self.get_chat_room_by_id(chat_room_id).await?;
        if !chat_room.contains_user(current_user_id) {
            return Err(MessageServiceError::Unauthorized);
        }

        //check i user deleting is author
        let result = sqlx::query!(
            "DELETE FROM messages WHERE id = $1 AND sender_id = $2 AND chat_room_id = $3",
            message_id,
            current_user_id,
            chat_room_id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(MessageServiceError::MessageNotFound);
        }

        let username = sqlx::query!("SELECT username FROM users WHERE id = $1", current_user_id)
            .fetch_one(&self.pool)
            .await?
            .username;

        //broadcast deletion via websocket
        let deletion_event = AuctionEvent::MessageDeleted {
            chat_room_id,
            message_id,
            deleted_by_username: username,
            timestamp: Utc::now(),
        };

        if let Err(e) = self.websocket_service.broadcast_event(deletion_event) {
            tracing::warn!("Failed to broadcast message deletion event: {:?}", e);
        }

        Ok(())
    }

    pub async fn get_connections(
        &self,
        current_user_id: Uuid,
    ) -> Result<Vec<Connection>, MessageServiceError> {
        //admin can message anyone at any time
        let username = sqlx::query!("SELECT username FROM users WHERE id = $1", current_user_id)
            .fetch_one(&self.pool)
            .await?
            .username;

        let is_admin = username == "admin";

        let connections = if is_admin {
            sqlx::query_as!(
                ConnectedUser,
                r#"
            SELECT id, username
            FROM users
            WHERE id != $1
            ORDER BY username
            "#,
                current_user_id
            )
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as!(
                ConnectedUser,
                r#"
            SELECT DISTINCT u.id, u.username
            FROM users u
            WHERE u.id != $1 AND (
                u.id IN (
                    SELECT p.buyer_user_id FROM purchases p
                    JOIN items i ON p.item_id = i.item_id
                    WHERE p.seller_user_id = $1
                )
                OR
                u.id IN (
                    SELECT p.seller_user_id FROM purchases p
                    JOIN items i ON p.item_id = i.item_id
                    WHERE p.buyer_user_id = $1
                )
            )
            ORDER BY u.username
            "#,
                current_user_id
            )
            .fetch_all(&self.pool)
            .await?
        };

        let connections = connections
            .into_iter()
            .map(|row| Connection {
                connected_user: ConnectedUser {
                    id: row.id,
                    username: row.username,
                },
            })
            .collect();

        Ok(connections)
    }

    async fn create_message_notification(
        &self,
        notification: &Notification,
        chat_room_id: Uuid,
    ) -> Result<(), MessageServiceError> {
        sqlx::query!(
            r#"
            INSERT INTO notifications (
                id, user_id, notification_type, title, message,
                item_id, related_user_id, amount, is_read, created_at, read_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            notification.id,
            notification.user_id,
            serde_json::to_string(&notification.notification_type).map_err(|e| {
                MessageServiceError::UnexpectedError(anyhow::anyhow!("Serialization error: {}", e))
            })?,
            notification.title,
            notification.message,
            notification.item_id,
            notification.related_user_id,
            notification.amount,
            notification.is_read,
            notification.created_at,
            notification.read_at
        )
        .execute(&self.pool)
        .await?;

        let notification_type_str = serde_json::to_string(&notification.notification_type)
            .map_err(|e| {
                MessageServiceError::UnexpectedError(anyhow::anyhow!("Serialization error: {}", e))
            })?;

        tracing::info!(
            "Broadcasting message notification to user {}: {}",
            notification.user_id,
            notification.title
        );

        if let Err(e) = self.websocket_service.broadcast_notification_to_user(
            notification.user_id,
            notification.id,
            notification.title.clone(),
            notification.message.clone(),
            notification_type_str,
            Some(chat_room_id.to_string()),
        ) {
            tracing::error!(
                "Failed to broadcast message notification via WebSocket: {:?}",
                e
            );
        }

        Ok(())
    }

    async fn get_chat_room_by_id(
        &self,
        chat_room_id: Uuid,
    ) -> Result<ChatRoom, MessageServiceError> {
        sqlx::query_as!(
            ChatRoom,
            "SELECT id, user1_id, user2_id, created_at, updated_at FROM chat_rooms WHERE id = $1",
            chat_room_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or(MessageServiceError::ChatRoomNotFound)
    }
}
