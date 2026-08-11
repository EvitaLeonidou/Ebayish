use crate::define_route_error;
use teddy_services::bid_service::BidServiceError;
use reqwest::StatusCode;

define_route_error! {
    BidError {
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid bid data provided"),
        NotFound => (StatusCode::NOT_FOUND, "Bid not found"),
        ItemNotFound => (StatusCode::NOT_FOUND, "Item not found"),
        BidTooLow => (StatusCode::BAD_REQUEST, "Bid amount must be higher than current price"),
        AuctionEnded => (StatusCode::BAD_REQUEST, "Auction has ended"),
        SelfBidding => (StatusCode::BAD_REQUEST, "Cannot bid on your own item"),
        BuyItNowTriggered => (StatusCode::OK, "Buy-it-now triggered, auction ended"),
        ServiceError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal service error"),
    }
}

impl From<BidServiceError> for BidError {
    fn from(err: BidServiceError) -> Self {
        match err {
            BidServiceError::ItemNotFound => BidError::ItemNotFound,
            BidServiceError::BidTooLow { .. } => BidError::BidTooLow,
            BidServiceError::AuctionEnded => BidError::AuctionEnded,
            BidServiceError::SelfBidding => BidError::SelfBidding,
            BidServiceError::BuyItNowTriggered => BidError::BuyItNowTriggered,
            _ => BidError::ServiceError,
        }
    }
}