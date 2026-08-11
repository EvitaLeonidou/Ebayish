//! src/routes/cart.rs
use crate::define_route_error;
use crate::jwt_middleware::Claims;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use reqwest::StatusCode;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

define_route_error! {
    CartError {
        ItemNotFound => (StatusCode::NOT_FOUND, "Item not found"),
        ItemAlreadyInCart => (StatusCode::CONFLICT, "Item already in cart"),
        // Added error for a user trying to add their own item.
        CannotAddToCartOwnItem => (StatusCode::FORBIDDEN, "You cannot add your own item to the cart"),
        ItemNotInCart => (StatusCode::NOT_FOUND, "Item not in cart"),
        AuthenticationError => (StatusCode::UNAUTHORIZED, "Authentication required"),
    }
}

#[derive(Serialize)]
pub struct CartItem {
    pub item_id: String,
    pub name: String,
    pub currently: bigdecimal::BigDecimal,
    pub buy_price: Option<bigdecimal::BigDecimal>,
    pub images: Vec<String>,
    pub listing_type: String,
}

#[tracing::instrument(
    name = "Add item to cart",
    skip(claims, pool, path),
    fields(user_id = %claims.sub, item_id = %path.as_ref())
)]
pub async fn add_to_cart(
    claims: Claims,
    path: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CartError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| CartError::AuthenticationError)?;
    let item_id = path.into_inner();

    let record = sqlx::query!(
        "SELECT seller_user_id FROM items WHERE item_id = $1",
        item_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch item's seller")?;

    let seller_id = match record {
        Some(rec) => rec.seller_user_id,
        None => return Err(CartError::ItemNotFound),
    };

    if seller_id == Some(user_id) {
        return Err(CartError::CannotAddToCartOwnItem);
    }

    let already_in_cart = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM cart WHERE user_id = $1 AND item_id = $2)",
        user_id,
        item_id
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to check if item is in cart")?
    .unwrap_or(false);

    if already_in_cart {
        return Err(CartError::ItemAlreadyInCart);
    }

    sqlx::query!(
        "INSERT INTO cart (user_id, item_id) VALUES ($1, $2)",
        user_id,
        item_id
    )
    .execute(pool.get_ref())
    .await
    .context("Failed to add item to cart")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Remove item from cart",
    skip(claims, pool, path),
    fields(user_id = %claims.sub, item_id = %path.as_ref())
)]
pub async fn remove_from_cart(
    claims: Claims,
    path: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CartError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| CartError::AuthenticationError)?;
    let item_id = path.into_inner();

    let result = sqlx::query!(
        "DELETE FROM cart WHERE user_id = $1 AND item_id = $2",
        user_id,
        item_id
    )
    .execute(pool.get_ref())
    .await
    .context("Failed to remove item from cart")?;

    if result.rows_affected() == 0 {
        return Err(CartError::ItemNotInCart);
    }

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Get user cart",
    skip(claims, pool),
    fields(user_id = %claims.sub)
)]
pub async fn get_cart(claims: Claims, pool: web::Data<PgPool>) -> Result<HttpResponse, CartError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| CartError::AuthenticationError)?;

    let items = sqlx::query!(
        r#"
        SELECT
            i.item_id,
            i.name,
            i.currently,
            i.buy_price,
            i.listing_type,
            COALESCE(
                array_agg(
                    CONCAT('/api/uploads/items/', i.item_id, '/', img.filename)
                    ORDER BY img.display_order
                ) FILTER (WHERE img.filename IS NOT NULL),
                '{}'
            ) as images
        FROM cart c
        JOIN items i ON c.item_id = i.item_id
        LEFT JOIN item_images img ON i.item_id = img.item_id
        WHERE c.user_id = $1
        GROUP BY i.item_id, i.name, i.currently, i.buy_price, i.listing_type
        ORDER BY i.name
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch cart items")?
    .into_iter()
    .map(|row| CartItem {
        item_id: row.item_id,
        name: row.name,
        currently: row.currently.unwrap_or_default(),
        buy_price: row.buy_price,
        images: row.images.unwrap_or_default(),
        listing_type: row.listing_type,
    })
    .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(items))
}

#[tracing::instrument(
    name = "Clear user cart",
    skip(claims, pool),
    fields(user_id = %claims.sub)
)]
pub async fn clear_cart(
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CartError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| CartError::AuthenticationError)?;

    sqlx::query!("DELETE FROM cart WHERE user_id = $1", user_id)
        .execute(pool.get_ref())
        .await
        .context("Failed to clear cart")?;

    Ok(HttpResponse::Ok().finish())
}
