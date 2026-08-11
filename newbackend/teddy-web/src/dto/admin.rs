// teddy-web/src/dto/admin.rs
// Admin-specific DTOs

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

// Struct for the dashboard stats
#[derive(Serialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub pending_users: i64,
    pub active_listings: i64,
    pub total_revenue: bigdecimal::BigDecimal,
}

// Define the structure for a single activity item in the feed
#[derive(Serialize)]
pub struct ActivityItem {
    pub id: Uuid, // A unique ID for React keys
    pub activity_type: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub target_id: Option<String>, // e.g., an item_id
}

// Define the response wrapper for the activity feed
#[derive(Serialize)]
pub struct RecentActivityResponse {
    pub activities: Vec<ActivityItem>,
}

// Admin system DTOs
#[derive(Serialize)]
pub struct AuctionResultResponse {
    pub item_id: String,
    pub seller_user_id: String,
    pub winner_user_id: Option<String>,
    pub winning_amount: Option<String>,
    pub ended_at: String,
    pub total_bids: i32,
}