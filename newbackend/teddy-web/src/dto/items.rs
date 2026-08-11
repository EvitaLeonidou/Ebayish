use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::handlers::catalog::images::ItemImageResponse;

#[derive(Debug, Deserialize)]
pub struct ItemRequest {
    pub item_id: Option<String>,
    pub name: String,
    pub first_bid: BigDecimal,
    pub currently: BigDecimal,
    pub buy_price: Option<BigDecimal>,
    pub number_of_bids: Option<i32>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub started: DateTime<Utc>,
    pub ends: DateTime<Utc>,
    pub description: Option<String>,
    pub seller_user_id: Uuid,
    pub condition: Option<String>,
    pub shipping_cost: Option<BigDecimal>,
    pub categories: Vec<String>,
}

#[derive(Serialize)]
pub struct Item {
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
    pub images: Vec<ItemImageResponse>,
}

#[derive(Serialize)]
pub struct ItemListResponse {
    pub items: Vec<Item>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct ItemDetailResponse {
    pub item: Item,
}