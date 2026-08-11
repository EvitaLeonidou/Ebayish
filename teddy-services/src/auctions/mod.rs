// Auction services module

pub mod management;
pub mod countdown;

// Re-export key types and services
pub use management::{AuctionService, AuctionServiceError, AuctionInfo, AuctionStats};
pub use countdown::{CountdownService, AuctionEvent, WebSocketNotifier};