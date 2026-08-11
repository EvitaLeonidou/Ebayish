use actix_web::{web, Scope};
use crate::handlers::bidding::{
    bid_crud::{create_bid, delete_bid, update_bid},
    bid_query::{get_bid, get_bids, get_bids_for_item},
};

pub fn configure() -> Scope {
    web::scope("")
        // Bid routes
        .route("/items/{item_id}/bids", web::post().to(create_bid))
        .route("/items/{item_id}/bids", web::get().to(get_bids_for_item))
        .route("/bids", web::get().to(get_bids))
        .route("/bids/{bid_id}", web::get().to(get_bid))
        .route("/bids/{bid_id}", web::put().to(update_bid))
        .route("/bids/{bid_id}", web::delete().to(delete_bid))
}