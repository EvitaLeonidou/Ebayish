use actix_web::{web, Scope};
use crate::handlers::auth::{login::login, roles::user_role};

pub fn configure() -> Scope {
    web::scope("")
        .route("/login", web::post().to(login))
        .route("/user_role", web::get().to(user_role))
}