// teddy-services: Business operations and database interactions

pub mod catalog;
pub mod bidding;
pub mod users;
pub mod auctions;
pub mod media;
pub mod realtime;
pub mod admin;
pub mod error_handling;

// Re-export commonly used services and types for convenient access
pub use catalog::{Category, CategoryService, CategoryServiceError, Item, ItemService, ItemServiceError};
pub use users::{UserService, UserServiceError, PendingUser, UserCredentials};
pub use auctions::{AuctionService, AuctionServiceError, AuctionInfo, AuctionStats, CountdownService, AuctionEvent, WebSocketNotifier};
pub use bidding::{BidService, BidServiceError, BidResult, ItemInfo, NewBid};
pub use admin::{SeedingService, SeedingError};

// Temporary placeholders for services not yet migrated
pub use crate::media::storage::ImageService;
pub use crate::realtime::events::WebSocketService;

// Re-export submodules for easier access
pub mod bid_service {
    pub use crate::bidding::BidServiceError;
}

pub mod websocket_service {
    pub use crate::auctions::AuctionEvent;
}