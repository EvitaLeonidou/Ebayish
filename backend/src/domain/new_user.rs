//! src/domain/new_user.rs

use crate::domain::user_email::UserEmail;
use crate::domain::username::Username;
use chrono::NaiveDate;

pub struct NewUser {
    pub username: Username,
    pub email: UserEmail,
    pub password_hash: String,
    pub first_name: Username,
    pub last_name: Username,
    pub phone: String,
    pub date_of_birth: NaiveDate,
    pub seller_rating: Option<bigdecimal::BigDecimal>,
    pub tax_id: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
}
