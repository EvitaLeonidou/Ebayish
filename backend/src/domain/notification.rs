use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub item_id: Option<String>,
    pub related_user_id: Option<Uuid>, // For bidder/buyer info
    pub amount: Option<BigDecimal>,    // For bid/sale amounts
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    ItemSold,
    NewBid,
    AuctionEnded,

    BidOutbid,
    AuctionWon,
    AuctionLost,

    NewChatRoom,
    NewMessage,
}

impl Notification {
    pub fn new_item_sold(
        seller_id: Uuid,
        item_id: String,
        buyer_username: &str,
        amount: BigDecimal,
        item_name: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: seller_id,
            notification_type: NotificationType::ItemSold,
            title: "Item Sold!".to_string(),
            message: format!(
                "Your item '{}' has been sold to {} for {}",
                item_name,
                buyer_username,
                format_currency(&amount)
            ),
            item_id: Some(item_id),
            related_user_id: None,
            amount: Some(amount),
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn new_bid_received(
        seller_id: Uuid,
        item_id: String,
        bidder_username: &str,
        bid_amount: BigDecimal,
        item_name: &str,
        total_bids: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: seller_id,
            notification_type: NotificationType::NewBid,
            title: "New Bid Received!".to_string(),
            message: format!(
                "{} placed a bid of {} on your item '{}' (Total bids: {})",
                bidder_username,
                format_currency(&bid_amount),
                item_name,
                total_bids
            ),
            item_id: Some(item_id),
            related_user_id: None,
            amount: Some(bid_amount),
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn new_auction_ended_seller(
        seller_id: Uuid,
        item_id: String,
        item_name: &str,
        final_amount: BigDecimal,
        winner_username: Option<&str>,
        total_bids: i32,
    ) -> Self {
        let message = match winner_username {
            Some(winner) => format!(
                "Your auction for '{}' has ended! Sold to {} for {} ({} total bids)",
                item_name,
                winner,
                format_currency(&final_amount),
                total_bids
            ),
            None => format!(
                "Your auction for '{}' has ended with no winner ({} total bids)",
                item_name, total_bids
            ),
        };

        Self {
            id: Uuid::new_v4(),
            user_id: seller_id,
            notification_type: NotificationType::AuctionEnded,
            title: "Auction Ended".to_string(),
            message,
            item_id: Some(item_id),
            related_user_id: None,
            amount: Some(final_amount),
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn new_bid_outbid(
        bidder_id: Uuid,
        item_id: String,
        item_name: &str,
        their_bid: BigDecimal,
        new_bid: BigDecimal,
        outbidder_username: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: bidder_id,
            notification_type: NotificationType::BidOutbid,
            title: "You've Been Outbid!".to_string(),
            message: format!(
                "Your bid of {} on '{}' has been outbid by {} with a bid of {}",
                format_currency(&their_bid),
                item_name,
                outbidder_username,
                format_currency(&new_bid)
            ),
            item_id: Some(item_id),
            related_user_id: None,
            amount: Some(new_bid),
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn new_auction_won(
        winner_id: Uuid,
        item_id: String,
        item_name: &str,
        winning_bid: BigDecimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: winner_id,
            notification_type: NotificationType::AuctionWon,
            title: "Auction Won!".to_string(),
            message: format!(
                "Congratulations! You won the auction for '{}' with your bid of {}",
                item_name,
                format_currency(&winning_bid)
            ),
            item_id: Some(item_id),
            related_user_id: None,
            amount: Some(winning_bid),
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn new_auction_lost(
        bidder_id: Uuid,
        item_id: String,
        item_name: &str,
        their_final_bid: BigDecimal,
        winning_bid: BigDecimal,
        _winner_username: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: bidder_id,
            notification_type: NotificationType::AuctionLost,
            title: "Auction Lost".to_string(),
            message: format!(
                "You didn't win the auction for '{}'. Your final bid was {}, but another user won with {}",
                item_name,
                format_currency(&their_final_bid),
                format_currency(&winning_bid)
            ),
            item_id: Some(item_id),
            related_user_id: None,
            amount: Some(winning_bid),
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn new_chat_room(
        recipient_id: Uuid,
        initiator_username: &str,
        initiator_id: Uuid,
        _chat_room_id: Uuid,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: recipient_id,
            notification_type: NotificationType::NewChatRoom,
            title: "New Conversation".to_string(),
            message: format!("{} started a conversation with you", initiator_username),
            item_id: None,
            related_user_id: Some(initiator_id),
            amount: None,
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn new_message(
        recipient_id: Uuid,
        sender_username: &str,
        sender_id: Uuid,
        _chat_room_id: Uuid,
        message_preview: &str,
    ) -> Self {
        let preview = if message_preview.len() > 50 {
            format!("{}...", &message_preview[..50])
        } else {
            message_preview.to_string()
        };

        Self {
            id: Uuid::new_v4(),
            user_id: recipient_id,
            notification_type: NotificationType::NewMessage,
            title: format!("New message from {}", sender_username),
            message: format!("{}: {}", sender_username, preview),
            item_id: None,
            related_user_id: Some(sender_id),
            amount: None,
            is_read: false,
            created_at: Utc::now(),
            read_at: None,
        }
    }

    pub fn mark_as_read(&mut self) {
        self.is_read = true;
        self.read_at = Some(Utc::now());
    }
}

fn format_currency(amount: &BigDecimal) -> String {
    format!("${:.2}", amount)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationSummary {
    pub total_count: i64,
    pub unread_count: i64,
    pub notification_types: Vec<NotificationTypeCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationTypeCount {
    pub notification_type: NotificationType,
    pub count: i64,
    pub unread_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct NotificationFilters {
    pub notification_type: Option<NotificationType>,
    pub is_read: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
