use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NewBid {
    pub item_id: String,
    pub bidder_user_id: Uuid,
    pub bidder_rating: Option<i32>,
    pub time: DateTime<Utc>,
    pub amount: BigDecimal,
    pub bidder_location: Option<String>,
    pub bidder_country: Option<String>,
}

impl NewBid {
    pub fn validate(&self) -> Result<(), String> {
        if self.item_id.trim().is_empty() {
            return Err("Item ID cannot be empty".to_string());
        }

        if self.amount <= BigDecimal::from(0) {
            return Err("Bid amount must be greater than zero".to_string());
        }

        Ok(())
    }
}
