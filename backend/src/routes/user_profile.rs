//! src/routes/user_profile.rs
use crate::define_route_error;
use crate::jwt_middleware::Claims;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use reqwest::StatusCode;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UserStats {
    items_sold: i64,
    active_auctions: i64,
    active_fixed_price: i64,
    successful_bids: i64,
}

#[derive(Serialize)]
pub struct PurchasedItem {
    item_id: String,
    name: String,
    price: bigdecimal::BigDecimal,
    currently: Option<bigdecimal::BigDecimal>,
    images: serde_json::Value,
    purchase_date: chrono::NaiveDateTime,
    purchase_price: Option<bigdecimal::BigDecimal>,
    seller_username: String,
    seller_user_id: Uuid,
}

#[derive(Serialize)]
pub struct SoldItem {
    item_id: String,
    name: String,
    price: bigdecimal::BigDecimal,
    final_price: Option<bigdecimal::BigDecimal>,
    images: serde_json::Value,
    sold_date: chrono::NaiveDateTime,
    buyer_username: String,
    buyer_id: Uuid,
    total_bids: Option<i32>,
}

#[derive(Serialize)]
pub struct ActiveListing {
    item_id: String,
    name: String,
    price: bigdecimal::BigDecimal,
    currently: Option<bigdecimal::BigDecimal>,
    number_of_bids: Option<i32>,
    images: serde_json::Value,
    ends: Option<chrono::NaiveDateTime>,
    status: Option<String>,
    listing_type: Option<String>,
}

#[derive(Serialize)]
pub struct BidHistoryItem {
    bid_id: Uuid,
    item_id: Option<String>,
    item_title: Option<String>,
    amount: bigdecimal::BigDecimal,
    status: Option<String>,
    created_at: chrono::NaiveDateTime,
    current_price: Option<bigdecimal::BigDecimal>,
    auction_ends: Option<chrono::NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct PasswordChangeRequest {
    current_password: String,
    new_password: String,
}

define_route_error! {
    UserProfileError {
        NotFound => (StatusCode::NOT_FOUND, "User not found"),
        Unauthorized => (StatusCode::UNAUTHORIZED, "Authentication required"),
        Forbidden => (StatusCode::FORBIDDEN, "Access denied"),
        InvalidPassword => (StatusCode::BAD_REQUEST, "Invalid current password"),
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid input data"),
    }
}

#[tracing::instrument(name = "Get user statistics", skip(pool, path, claims))]
pub async fn get_user_stats(
    path: web::Path<Uuid>,
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, UserProfileError> {
    let user_id = path.into_inner();

    let requesting_user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| UserProfileError::Unauthorized)?;

    if requesting_user_id != user_id && claims.role != "admin" {
        return Err(UserProfileError::Forbidden);
    }

    let user_exists =
        sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)", user_id)
            .fetch_one(pool.get_ref())
            .await
            .context("Failed to check user existence")?
            .unwrap_or(false);

    if !user_exists {
        return Err(UserProfileError::NotFound);
    }

    let auction_sales_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint
        FROM items i
        JOIN auction_results ar ON i.item_id = ar.item_id
        WHERE i.seller_user_id = $1 AND ar.winner_user_id IS NOT NULL
        "#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to fetch auction sales count")?
    .unwrap_or(0);

    let direct_sales_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint
        FROM purchases p
        JOIN items i ON p.item_id = i.item_id
        WHERE i.seller_user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to fetch direct sales count")?
    .unwrap_or(0);

    let items_sold = auction_sales_count + direct_sales_count;

    let active_auctions = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint
        FROM items i
        WHERE i.seller_user_id = $1
        AND i.listing_type = 'auction'
        AND i.ends > NOW()
        AND NOT EXISTS (
            SELECT 1 FROM auction_results ar
            WHERE ar.item_id = i.item_id AND ar.winner_user_id IS NOT NULL
        )
        "#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to fetch active auctions count")?
    .unwrap_or(0);

    let active_fixed_price = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint
        FROM items i
        WHERE i.seller_user_id = $1
        AND i.listing_type = 'fixed_price'
        AND NOT EXISTS (
            SELECT 1 FROM purchases p WHERE p.item_id = i.item_id
        )
        "#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to fetch active fixed price items count")?
    .unwrap_or(0);

    let auction_wins_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint
        FROM auction_results
        WHERE winner_user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to fetch auction wins count")?
    .unwrap_or(0);

    let direct_purchases_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint
        FROM purchases
        WHERE buyer_user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to fetch direct purchases count")?
    .unwrap_or(0);

    let successful_bids = auction_wins_count + direct_purchases_count;

    let stats = UserStats {
        items_sold,
        active_auctions,
        active_fixed_price,
        successful_bids,
    };

    Ok(HttpResponse::Ok().json(stats))
}

#[tracing::instrument(name = "Get user purchased items", skip(pool, path, claims))]
pub async fn get_purchased_items(
    path: web::Path<Uuid>,
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, UserProfileError> {
    let user_id = path.into_inner();

    let requesting_user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| UserProfileError::Unauthorized)?;

    if requesting_user_id != user_id && claims.role != "admin" {
        return Err(UserProfileError::Forbidden);
    }

    let auction_wins = sqlx::query_as!(
        PurchasedItem,
        r#"
        SELECT
            i.item_id,
            i.name,
            i.price,
            i.currently,
            COALESCE(
                (SELECT json_agg(json_build_object('url', '/uploads/items/' || i.item_id || '/' || filename))
                 FROM item_images WHERE item_id = i.item_id),
                '[]'::json
            ) as "images!",
            ar.ended_at as purchase_date,
            ar.winning_amount as purchase_price,
            u.username as seller_username,
            u.id as seller_user_id
        FROM items i
        JOIN auction_results ar ON i.item_id = ar.item_id
        JOIN users u ON i.seller_user_id = u.id
        WHERE ar.winner_user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch auction wins")?;

    let direct_purchases = sqlx::query_as!(
        PurchasedItem,
        r#"
        SELECT
            i.item_id,
            i.name,
            i.price,
            i.currently,
            COALESCE(
                (SELECT json_agg(json_build_object('url', '/uploads/items/' || i.item_id || '/' || filename))
                 FROM item_images WHERE item_id = i.item_id),
                '[]'::json
            ) as "images!",
            p.purchased_at as purchase_date,
            p.purchase_price as purchase_price,
            u.username as seller_username,
            u.id as seller_user_id
        FROM items i
        JOIN purchases p ON i.item_id = p.item_id
        JOIN users u ON i.seller_user_id = u.id
        WHERE p.buyer_user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch direct purchases")?;

    let mut purchased_items = auction_wins;
    purchased_items.extend(direct_purchases);
    purchased_items.sort_by(|a, b| b.purchase_date.cmp(&a.purchase_date));

    Ok(HttpResponse::Ok().json(purchased_items))
}

#[tracing::instrument(name = "Get user sold items", skip(pool, path, claims))]
pub async fn get_sold_items(
    path: web::Path<Uuid>,
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, UserProfileError> {
    let user_id = path.into_inner();

    let requesting_user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| UserProfileError::Unauthorized)?;

    if requesting_user_id != user_id && claims.role != "admin" {
        return Err(UserProfileError::Forbidden);
    }

    let auction_sales = sqlx::query_as!(
        SoldItem,
        r#"
        SELECT
            i.item_id,
            i.name,
            i.price,
            ar.winning_amount as final_price,
            COALESCE(
                (SELECT json_agg(json_build_object('url', '/uploads/items/' || i.item_id || '/' || filename))
                 FROM item_images WHERE item_id = i.item_id),
                '[]'::json
            ) as "images!",
            ar.ended_at as sold_date,
            u.username as buyer_username,
            u.id as buyer_id,
            ar.total_bids
        FROM items i
        JOIN auction_results ar ON i.item_id = ar.item_id
        JOIN users u ON ar.winner_user_id = u.id
        WHERE i.seller_user_id = $1 AND ar.winner_user_id IS NOT NULL
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch auction sales")?;

    let direct_sales = sqlx::query_as!(
        SoldItem,
        r#"
        SELECT
            i.item_id,
            i.name,
            i.price,
            p.purchase_price as final_price,
            COALESCE(
                (SELECT json_agg(json_build_object('url', '/uploads/items/' || i.item_id || '/' || filename))
                 FROM item_images WHERE item_id = i.item_id),
                '[]'::json
            ) as "images!",
            p.purchased_at as sold_date,
            u.username as buyer_username,
            u.id as buyer_id,
            NULL::integer as total_bids
        FROM items i
        JOIN purchases p ON i.item_id = p.item_id
        JOIN users u ON p.buyer_user_id = u.id
        WHERE i.seller_user_id = $1
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch direct sales")?;

    let mut sold_items = auction_sales;
    sold_items.extend(direct_sales);
    sold_items.sort_by(|a, b| b.sold_date.cmp(&a.sold_date));

    Ok(HttpResponse::Ok().json(sold_items))
}

#[derive(Deserialize)]
pub struct ActiveListingsQuery {
    #[serde(rename = "type")]
    listing_type: Option<String>,
}

#[tracing::instrument(name = "Get user active listings", skip(pool, path, claims, query))]
pub async fn get_active_listings(
    path: web::Path<Uuid>,
    query: web::Query<ActiveListingsQuery>,
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, UserProfileError> {
    let user_id = path.into_inner();

    let requesting_user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| UserProfileError::Unauthorized)?;

    if requesting_user_id != user_id && claims.role != "admin" {
        return Err(UserProfileError::Forbidden);
    }

    let active_listings = sqlx::query_as!(
        ActiveListing,
        r#"
        SELECT
            i.item_id,
            i.name,
            i.price,
            i.currently,
            i.number_of_bids,
            COALESCE(
                (SELECT json_agg(json_build_object('url', '/uploads/items/' || i.item_id || '/' || filename))
                 FROM item_images WHERE item_id = i.item_id),
                '[]'::json
            ) as "images!",
            i.ends,
            'active'::text as status,
            i.listing_type
        FROM items i
        WHERE i.seller_user_id = $1
        AND (
            (i.listing_type = 'auction' AND i.ends > NOW() AND NOT EXISTS (
                SELECT 1 FROM auction_results ar WHERE ar.item_id = i.item_id AND ar.winner_user_id IS NOT NULL
            )) OR
            (i.listing_type = 'fixed_price' AND NOT EXISTS (
                SELECT 1 FROM purchases p WHERE p.item_id = i.item_id
            ))
        )
        AND ($2::text IS NULL OR i.listing_type = $2)
        ORDER BY i.ends ASC
        "#,
        user_id,
        query.listing_type.as_deref()
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch active listings")?;

    Ok(HttpResponse::Ok().json(active_listings))
}

#[tracing::instrument(name = "Get user bid history", skip(pool, path, claims))]
pub async fn get_bid_history(
    path: web::Path<Uuid>,
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, UserProfileError> {
    let user_id = path.into_inner();

    let requesting_user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| UserProfileError::Unauthorized)?;

    if requesting_user_id != user_id && claims.role != "admin" {
        return Err(UserProfileError::Forbidden);
    }

    let bid_history = sqlx::query_as!(
        BidHistoryItem,
        r#"
        SELECT
            b.id as bid_id,
            b.item_id,
            i.name as item_title,
            b.amount,
            CASE
                WHEN ar.winner_user_id = $1 THEN 'won'::text
                WHEN ar.winner_user_id IS NOT NULL AND ar.winner_user_id != $1 THEN 'lost'::text
                WHEN i.ends <= NOW() THEN 'lost'::text
                WHEN b.amount = i.currently THEN 'winning'::text
                ELSE 'outbid'::text
            END as status,
            b.time as created_at,
            i.currently as current_price,
            i.ends as auction_ends
        FROM bids b
        JOIN items i ON b.item_id = i.item_id
        LEFT JOIN auction_results ar ON i.item_id = ar.item_id
        WHERE b.bidder_user_id = $1
        ORDER BY b.time DESC
        "#,
        user_id
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch bid history")?;

    Ok(HttpResponse::Ok().json(bid_history))
}

#[tracing::instrument(name = "Change user password", skip(pool, path, claims, json))]
pub async fn change_password(
    path: web::Path<Uuid>,
    claims: Claims,
    json: web::Json<PasswordChangeRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, UserProfileError> {
    let user_id = path.into_inner();

    let requesting_user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| UserProfileError::Unauthorized)?;

    if requesting_user_id != user_id && claims.role != "admin" {
        return Err(UserProfileError::Forbidden);
    }

    if claims.role != "admin" {
        use crate::domain::Username;
        use crate::routes::login::get_user_data_from_credentials;

        let username_str = sqlx::query_scalar!("SELECT username FROM users WHERE id = $1", user_id)
            .fetch_one(pool.get_ref())
            .await
            .context("Failed to fetch username")?;

        let username =
            Username::parse(username_str).map_err(|_| UserProfileError::ValidationError)?;

        let is_valid = get_user_data_from_credentials(
            &username,
            &Secret::new(json.current_password.clone()),
            pool.get_ref(),
        )
        .await
        .context("Failed to verify password")?
        .is_some();

        if !is_valid {
            return Err(UserProfileError::InvalidPassword);
        }
    }

    use crate::domain::hash_password;
    let new_hash = hash_password(Secret::new(json.new_password.clone()))
        .context("Failed to hash new password")?;

    sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        new_hash,
        user_id
    )
    .execute(pool.get_ref())
    .await
    .context("Failed to update password")?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password updated successfully"
    })))
}
