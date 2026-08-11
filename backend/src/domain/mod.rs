// src/domain/mod.rs

mod bid;
mod category;
mod image;
mod item;
mod message;
mod new_user;
mod notification;
mod user_email;
mod username;

pub use bid::NewBid;
pub use category::{CategoryName, NewCategory};
pub use image::{DisplayOrderManager, FileValidator, ItemImage, NewItemImage};
pub use item::NewItem;
pub use message::{
    ChatRoom, ConnectedUser, Connection, CreateChatRoomRequest, CreateChatRoomResponse, Message,
    MessageUser, MessageWithUser, MessagesResponse, NewMessage, ValidatedMessage,
};
pub use new_user::NewUser;
pub use notification::{
    Notification, NotificationFilters, NotificationSummary, NotificationType, NotificationTypeCount,
};
pub use user_email::UserEmail;
pub use username::Username;

use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use rand::rngs::OsRng;
use secrecy::{ExposeSecret, Secret};

pub fn hash_password(password: Secret<String>) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.expose_secret().as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}
