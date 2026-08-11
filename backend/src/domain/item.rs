use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct NewItem {
    pub item_id: String,
    pub listing_type: String,
    pub name: String,
    pub price: BigDecimal,
    pub currently: Option<BigDecimal>,
    pub buy_price: Option<BigDecimal>,
    pub number_of_bids: Option<i32>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub started: DateTime<Utc>,
    pub ends: Option<DateTime<Utc>>,
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
        if self.price <= BigDecimal::from(0) {
            return Err("Price must be greater than zero".to_string());
        }

        match self.listing_type.as_str() {
            "auction" => {
                if self.ends.is_none() {
                    return Err("Auction items must have an end date".to_string());
                }
                #[allow(clippy::collapsible_if)]
                if let Some(ends) = self.ends {
                    if self.started >= ends {
                        return Err("Start time must be before end time".to_string());
                    }
                }
                if self.currently.is_none() {
                    return Err("Auction items must have a current price".to_string());
                }
                if self.currently.as_ref().unwrap() < &self.price {
                    return Err("Current price cannot be less than starting price".to_string());
                }
            }
            "fixed_price" => {
                if self.ends.is_some() || self.currently.is_some() || self.number_of_bids.is_some()
                {
                    return Err("Fixed price items should not have 'ends', 'currently', or 'number_of_bids' fields"
                        .to_string());
                }
            }
            _ => return Err("Invalid listing type specified".to_string()),
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
