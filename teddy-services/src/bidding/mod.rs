// Bidding services module

pub mod bids;
pub mod validation;

// Re-export key types and services
pub use bids::{BidService, BidServiceError, BidResult, ItemInfo, NewBid};