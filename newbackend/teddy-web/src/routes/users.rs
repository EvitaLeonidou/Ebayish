use actix_web::{web, Scope};
use crate::handlers::users::{
    registration::create_user,
    profile::get_user_by_id,
};

pub fn configure() -> Scope {
    web::scope("")
        .route("/users", web::post().to(create_user))
        .route("/users/{user_id}", web::get().to(get_user_by_id))
}