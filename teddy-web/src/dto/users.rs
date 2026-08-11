// teddy-web/src/dto/users.rs
// User-related DTOs

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub date_of_birth: NaiveDate,
    pub seller_rating: Option<bigdecimal::BigDecimal>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub date_of_birth: NaiveDate,
    pub status: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub seller_rating: Option<bigdecimal::BigDecimal>,
}

// Public user info (limited data)
#[derive(Serialize)]
pub struct PublicUserInfo {
    pub id: Uuid,
    pub username: String,
    pub seller_rating: Option<bigdecimal::BigDecimal>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}