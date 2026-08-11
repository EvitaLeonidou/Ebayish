use actix_web::{web, Scope};
use crate::handlers::auctions::{
    lifecycle::force_end_auction,
    stats::{get_auction_result, get_auction_results, get_auction_stats},
};

pub fn configure() -> Scope {
    web::scope("")
        // Auction routes
        .route("/auctions/stats", web::get().to(get_auction_stats))
        .route("/auctions/results", web::get().to(get_auction_results))
        .route("/auctions/results/{item_id}", web::get().to(get_auction_result))
        .service(
            web::scope("/admin")
                .route("/auctions/{item_id}/end", web::post().to(force_end_auction))
        )
}