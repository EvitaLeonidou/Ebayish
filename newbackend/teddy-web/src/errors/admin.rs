// Admin specific errors
use crate::define_route_error;
use reqwest::StatusCode;

define_route_error! {
    DashboardError {
        DataFetchFailed => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch dashboard data"),
    }
}

define_route_error! {
    AuctionError {
        NotFound => (StatusCode::NOT_FOUND, "Auction not found"),
        AlreadyEnded => (StatusCode::BAD_REQUEST, "Auction has already ended"),
    }
}