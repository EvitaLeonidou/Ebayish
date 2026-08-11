use actix_web::{web, Scope};
use crate::handlers::catalog::{
    categories::{create_category, delete_category, get_categories, get_category, update_category},
    images::{delete_image, get_item_images, serve_image, upload_images},
    items_crud::{create_item, delete_item, update_item},
    items_query::{get_item, get_items},
};

pub fn configure() -> Scope {
    web::scope("")
        // Category routes
        .route("/categories", web::post().to(create_category))
        .route("/categories", web::get().to(get_categories))
        .route("/categories/{id}", web::get().to(get_category))
        .route("/categories/{id}", web::put().to(update_category))
        .route("/categories/{id}", web::delete().to(delete_category))
        // Item routes
        .route("/items", web::post().to(create_item))
        .route("/items", web::get().to(get_items))
        .route("/items/{item_id}", web::get().to(get_item))
        .route("/items/{item_id}", web::put().to(update_item))
        .route("/items/{item_id}", web::delete().to(delete_item))
        // Image routes
        .route("/items/{item_id}/images", web::post().to(upload_images))
        .route("/items/{item_id}/images", web::get().to(get_item_images))
        .route("/items/{item_id}/images/{image_id}", web::delete().to(delete_image))
        .route("/uploads/items/{item_id}/{filename}", web::get().to(serve_image))
}