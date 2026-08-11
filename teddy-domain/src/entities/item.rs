use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct NewItem {
    pub item_id: String,
    pub name: String,
    pub first_bid: BigDecimal,
    pub currently: BigDecimal,
    pub buy_price: Option<BigDecimal>,
    pub number_of_bids: i32,
    pub location: Option<String>,
    pub country: Option<String>,
    pub started: DateTime<Utc>,
    pub ends: DateTime<Utc>,
    pub description: Option<String>,
    pub seller_user_id: Uuid,
    pub seller_rating: Option<BigDecimal>,
    pub condition: Option<String>,
    pub shipping_cost: BigDecimal,
    pub categories: Vec<String>,
}

impl NewItem {
    pub fn validate(&self) -> Result<(), String> {
        if self.item_id.trim().is_empty() {
            return Err("Item ID cannot be empty".to_string());
        }

        if self.name.trim().is_empty() {
            return Err("Item name cannot be empty".to_string());
        }

        if self.name.len() > 255 {
            return Err("Item name cannot exceed 255 characters".to_string());
        }

        if self.first_bid <= BigDecimal::from(0) {
            return Err("First bid must be greater than zero".to_string());
        }

        if self.currently < self.first_bid {
            return Err("Current price cannot be less than first bid".to_string());
        }

        #[allow(clippy::collapsible_if)]
        if let Some(ref buy_price) = self.buy_price {
            if *buy_price <= self.first_bid {
                return Err("Buy price must be greater than first bid".to_string());
            }
        }

        if self.started >= self.ends {
            return Err("Start time must be before end time".to_string());
        }

        if self.shipping_cost < BigDecimal::from(0) {
            return Err("Shipping cost cannot be negative".to_string());
        }

        if self.categories.is_empty() {
            return Err("Item must have at least one category".to_string());
        }

        Ok(())
    }
}