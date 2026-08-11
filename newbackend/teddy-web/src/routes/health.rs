use actix_web::{web, Scope};
use crate::handlers::health::healthcheck;

pub fn configure() -> Scope {
    web::scope("")
        .route("/healthcheck", web::get().to(healthcheck))
}