//! Auction-related DTOs

use serde::Serialize;

#[derive(Serialize)]
pub struct AuctionResultResponse {
    pub item_id: String,
    pub seller_user_id: String,
    pub winner_user_id: Option<String>,
    pub winning_amount: Option<String>,
    pub ended_at: String,
    pub total_bids: i32,
}

#[derive(Serialize)]
pub struct AuctionStatsResponse {
    pub active_auctions: i64,
    pub ended_today: i64,
    pub total_bids_today: i64,
}