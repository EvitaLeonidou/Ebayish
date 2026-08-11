use actix_web::{web, Scope};

pub mod admin;
pub mod auth;
pub mod auctions;
pub mod bidding;
pub mod catalog;
pub mod health;
pub mod realtime;
pub mod users;

pub fn configure_routes() -> Scope {
    web::scope("")
        .service(health::configure())
        .service(auth::configure())
        .service(users::configure())
        .service(admin::configure())
        .service(catalog::configure())
        .service(bidding::configure())
        .service(auctions::configure())
        .service(realtime::configure())
}