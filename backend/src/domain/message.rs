use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub chat_room_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageWithUser {
    pub id: Uuid,
    pub chat_room_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub user: MessageUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageUser {
    pub id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRoom {
    pub id: Uuid,
    pub user1_id: Uuid,
    pub user2_id: Uuid,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize)]
pub struct NewMessage {
    pub content: String,
}

impl NewMessage {
    pub fn parse(self) -> Result<ValidatedMessage, String> {
        let content = self.content.trim();

        if content.is_empty() {
            return Err("Message content cannot be empty".to_string());
        }

        if content.len() > 1000 {
            return Err("Message content cannot exceed 1000 characters".to_string());
        }

        Ok(ValidatedMessage {
            content: content.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct ValidatedMessage {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateChatRoomRequest {
    pub other_user_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct CreateChatRoomResponse {
    pub room_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<MessageWithUser>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct Connection {
    pub connected_user: ConnectedUser,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct ConnectedUser {
    pub id: Uuid,
    pub username: String,
}

impl ChatRoom {
    pub fn get_other_user_id(&self, current_user_id: Uuid) -> Uuid {
        if self.user1_id == current_user_id {
            self.user2_id
        } else {
            self.user1_id
        }
    }

    pub fn contains_user(&self, user_id: Uuid) -> bool {
        self.user1_id == user_id || self.user2_id == user_id
    }
}
