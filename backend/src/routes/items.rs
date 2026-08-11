#![allow(clippy::collapsible_if)]

use crate::define_route_error;
use crate::domain::{ItemImage, NewItem};
use crate::jwt_middleware::Claims;
use crate::routes::images::ItemImageResponse;
use crate::services::{ImageService, PurchaseService, WebSocketService};
use actix_web::{HttpResponse, web};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ItemRequest {
    item_id: Option<String>,
    listing_type: String,
    name: String,
    price: BigDecimal,
    currently: Option<BigDecimal>,
    buy_price: Option<BigDecimal>,
    number_of_bids: Option<i32>,
    location: Option<String>,
    country: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    started: DateTime<Utc>,
    ends: Option<DateTime<Utc>>,
    description: Option<String>,
    seller_user_id: Uuid,
    condition: Option<String>,
    shipping_cost: Option<BigDecimal>,
    categories: Vec<String>,
}

#[derive(Serialize)]
pub struct Item {
    item_id: String,
    listing_type: String,
    name: String,
    price: BigDecimal,
    currently: Option<BigDecimal>,
    buy_price: Option<BigDecimal>,
    number_of_bids: Option<i32>,
    location: Option<String>,
    country: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    started: DateTime<Utc>,
    ends: Option<DateTime<Utc>>,
    description: Option<String>,
    seller_user_id: Uuid,
    seller_rating: Option<BigDecimal>,
    condition: Option<String>,
    shipping_cost: BigDecimal,
    categories: Vec<String>,
    images: Vec<ItemImageResponse>,
    status: Option<String>,
}

impl ItemRequest {
    fn into_new_item_with_seller_rating(
        self,
        seller_rating: Option<BigDecimal>,
    ) -> Result<NewItem, String> {
        let item = NewItem {
            item_id: self.item_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            listing_type: self.listing_type,
            name: self.name,
            price: self.price,
            currently: self.currently,
            buy_price: self.buy_price,
            number_of_bids: self.number_of_bids,
            location: self.location,
            country: self.country,
            latitude: self.latitude,
            longitude: self.longitude,
            started: self.started,
            ends: self.ends,
            description: self.description,
            seller_user_id: self.seller_user_id,
            seller_rating,
            condition: self.condition,
            shipping_cost: self.shipping_cost.unwrap_or_else(|| BigDecimal::from(0)),
            categories: self.categories,
        };

        item.validate()?;
        Ok(item)
    }
}

define_route_error! {
    ItemError {
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid item data provided"),
        NotFound => (StatusCode::NOT_FOUND, "Item not found"),
        CategoriesNotFound => (StatusCode::BAD_REQUEST, "One or more categories not found"),
    }
}

#[tracing::instrument(name = "Create item", skip(json, pool))]
pub async fn create_item(
    json: web::Json<ItemRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ItemError> {
    let seller_rating = sqlx::query_scalar!(
        "SELECT seller_rating FROM users WHERE id = $1",
        json.seller_user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch seller rating")?
    .flatten();

    let new_item: NewItem = json
        .0
        .into_new_item_with_seller_rating(seller_rating)
        .map_err(|e| {
            tracing::error!("Item validation error: {}", e);
            ItemError::ValidationError
        })?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;

    insert_item(&new_item, &mut transaction).await?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction for new item.")?;

    let item = get_item_with_categories(&new_item.item_id, &pool).await?;
    Ok(HttpResponse::Created().json(item))
}

#[derive(Debug, Deserialize)]
pub struct ItemsQuery {
    category: Option<i32>,
    q: Option<String>,
    #[serde(rename = "minPrice")]
    min_price: Option<String>,
    #[serde(rename = "maxPrice")]
    max_price: Option<String>,
    #[serde(rename = "sortBy")]
    sort_by: Option<String>,
    #[serde(rename = "type")]
    listing_type: Option<String>,
    location: Option<String>,
}

#[tracing::instrument(name = "Get all items", skip(pool))]
pub async fn get_items(
    query: web::Query<ItemsQuery>,
    pool: web::Data<PgPool>,
    _image_service: web::Data<ImageService>,
) -> Result<HttpResponse, ItemError> {
    let items = sqlx::query!(
        r#"SELECT item_id, listing_type, name, price, currently, buy_price, number_of_bids,
           location, country, latitude, longitude, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost, status
           FROM items ORDER BY created_at DESC"#,
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch items")?;

    let item_ids: Vec<String> = items.iter().map(|item| item.item_id.clone()).collect();
    let all_images = get_images_for_items(&item_ids, pool.get_ref())
        .await
        .context("Failed to batch load item images")?;

    let mut result_items = Vec::new();

    for item_row in items {
        let categories = get_item_categories(&item_row.item_id, &pool).await?;
        let images = all_images
            .get(&item_row.item_id)
            .cloned()
            .unwrap_or_default();

        let item = Item {
            item_id: item_row.item_id,
            listing_type: item_row.listing_type,
            name: item_row.name,
            price: item_row.price,
            currently: item_row.currently,
            buy_price: item_row.buy_price,
            number_of_bids: item_row.number_of_bids,
            location: item_row.location,
            country: item_row.country,
            latitude: item_row.latitude,
            longitude: item_row.longitude,
            started: item_row.started.and_utc(),
            ends: item_row.ends.map(|dt| dt.and_utc()),
            description: item_row.description,
            seller_user_id: item_row.seller_user_id.unwrap_or_default(),
            seller_rating: item_row.seller_rating,
            condition: item_row.condition,
            shipping_cost: item_row.shipping_cost,
            categories,
            images,
            status: item_row.status,
        };

        let mut include_item = true;

        if let Some(filter_type) = &query.listing_type {
            if item.listing_type != *filter_type {
                include_item = false;
            }
        }

        if let Some(category_id) = query.category {
            let category_matches = sqlx::query_scalar!(
                "SELECT EXISTS(SELECT 1 FROM item_categories ic JOIN categories c ON ic.category_id = c.id WHERE ic.item_id = $1 AND c.id = $2)",
                item.item_id,
                category_id
            )
            .fetch_one(pool.get_ref())
            .await
            .context("Failed to check category match")?
            .unwrap_or(false);

            if !category_matches {
                include_item = false;
            }
        }

        if let Some(search_term) = &query.q {
            if !search_term.trim().is_empty() {
                let search_lower = search_term.to_lowercase();
                let name_match = item.name.to_lowercase().contains(&search_lower);
                let desc_match = item
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&search_lower))
                    .unwrap_or(false);

                if !name_match && !desc_match {
                    include_item = false;
                }
            }
        }

        if let Some(filter_location) = &query.location {
            if !filter_location.trim().is_empty() {
                let location_matches = item
                    .location
                    .as_ref()
                    .map(|loc| {
                        let city = loc.split(',').next().unwrap_or(loc).trim();
                        city.to_lowercase() == filter_location.to_lowercase()
                    })
                    .unwrap_or(false);

                if !location_matches {
                    include_item = false;
                }
            }
        }

        if let Some(min_price_str) = &query.min_price {
            if !min_price_str.is_empty() {
                if let Ok(min_price) = min_price_str.parse::<BigDecimal>() {
                    let item_price = if item.listing_type == "auction" {
                        item.currently.as_ref().unwrap_or(&item.price)
                    } else {
                        &item.price
                    };
                    if item_price < &min_price {
                        include_item = false;
                    }
                }
            }
        }

        if let Some(max_price_str) = &query.max_price {
            if !max_price_str.is_empty() {
                if let Ok(max_price) = max_price_str.parse::<BigDecimal>() {
                    let item_price = if item.listing_type == "auction" {
                        item.currently.as_ref().unwrap_or(&item.price)
                    } else {
                        &item.price
                    };
                    if item_price > &max_price {
                        include_item = false;
                    }
                }
            }
        }

        if include_item {
            result_items.push(item);
        }
    }

    match query.sort_by.as_deref() {
        Some("price_asc") => {
            result_items.sort_by(|a, b| {
                let a_price = if a.listing_type == "auction" {
                    a.currently.as_ref().unwrap_or(&a.price)
                } else {
                    &a.price
                };
                let b_price = if b.listing_type == "auction" {
                    b.currently.as_ref().unwrap_or(&b.price)
                } else {
                    &b.price
                };
                a_price.cmp(b_price)
            });
        }
        Some("price_desc") => {
            result_items.sort_by(|a, b| {
                let a_price = if a.listing_type == "auction" {
                    a.currently.as_ref().unwrap_or(&a.price)
                } else {
                    &a.price
                };
                let b_price = if b.listing_type == "auction" {
                    b.currently.as_ref().unwrap_or(&b.price)
                } else {
                    &b.price
                };
                b_price.cmp(a_price)
            });
        }
        Some("ending_soon") => {
            result_items.sort_by(|a, b| match (a.ends, b.ends) {
                (Some(a_end), Some(b_end)) => a_end.cmp(&b_end),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            });
        }
        _ => {}
    }

    Ok(HttpResponse::Ok().json(result_items))
}

#[tracing::instrument(name = "Get item by ID", skip(pool, image_service))]
pub async fn get_item(
    path: web::Path<String>,
    pool: web::Data<PgPool>,
    image_service: web::Data<ImageService>,
) -> Result<HttpResponse, ItemError> {
    let item_id = path.into_inner();
    let item =
        get_item_with_categories_and_images(&item_id, pool.get_ref(), image_service.get_ref())
            .await?;
    Ok(HttpResponse::Ok().json(item))
}

#[tracing::instrument(name = "Update item", skip(json, pool, image_service))]
pub async fn update_item(
    path: web::Path<String>,
    json: web::Json<ItemRequest>,
    claims: Claims,
    pool: web::Data<PgPool>,
    image_service: web::Data<ImageService>,
) -> Result<HttpResponse, ItemError> {
    let item_id = path.into_inner();

    //only the seller can edit their item and the admin
    let requesting_user_id =
        uuid::Uuid::parse_str(&claims.sub).map_err(|_| ItemError::ValidationError)?;

    let item_seller = sqlx::query_scalar!(
        "SELECT seller_user_id FROM items WHERE item_id = $1",
        item_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch item seller")?;

    let seller_user_id = match item_seller {
        Some(Some(seller_id)) => seller_id,
        _ => return Err(ItemError::NotFound),
    };

    if requesting_user_id != seller_user_id && claims.role != "admin" {
        return Err(ItemError::ValidationError);
    }

    let seller_rating = sqlx::query_scalar!(
        "SELECT seller_rating FROM users WHERE id = $1",
        json.seller_user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch seller rating")?
    .flatten();

    let updated_item: NewItem = json
        .0
        .into_new_item_with_seller_rating(seller_rating)
        .map_err(|_| ItemError::ValidationError)?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;

    let rows_affected = sqlx::query_unchecked!(
        r#"UPDATE items SET listing_type = $1, name = $2, price = $3, currently = $4, buy_price = $5,
           number_of_bids = $6, location = $7, country = $8, latitude = $9, longitude = $10, started = $11, ends = $12,
           description = $13, seller_user_id = $14, seller_rating = $15, condition = $16
           WHERE item_id = $17"#,
        updated_item.listing_type,
        updated_item.name,
        updated_item.price,
        updated_item.currently,
        updated_item.buy_price,
        updated_item.number_of_bids,
        updated_item.location,
        updated_item.country,
        updated_item.latitude,
        updated_item.longitude,
        updated_item.started.naive_utc(),
        updated_item.ends.map(|dt| dt.naive_utc()),
        updated_item.description,
        updated_item.seller_user_id,
        updated_item.seller_rating,
        updated_item.condition,
        item_id
    )
    .execute(&mut *transaction)
    .await
    .context("Failed to update item")?
    .rows_affected();

    if rows_affected == 0 {
        return Err(ItemError::NotFound);
    }

    sqlx::query!(r#"DELETE FROM item_categories WHERE item_id = $1"#, item_id)
        .execute(&mut *transaction)
        .await
        .context("Failed to delete existing item categories")?;

    insert_item_categories(&item_id, &updated_item.categories, &mut transaction).await?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction for item update.")?;

    let item =
        get_item_with_categories_and_images(&item_id, pool.get_ref(), image_service.get_ref())
            .await?;
    Ok(HttpResponse::Ok().json(item))
}

#[tracing::instrument(name = "Delete item", skip(pool, image_service))]
pub async fn delete_item(
    path: web::Path<String>,
    pool: web::Data<PgPool>,
    image_service: web::Data<ImageService>,
) -> Result<HttpResponse, ItemError> {
    let item_id = path.into_inner();
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;
    let exists = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM items WHERE item_id = $1)"#,
        item_id
    )
    .fetch_one(&mut *transaction)
    .await
    .context("Failed to check item existence")?
    .unwrap_or(false);

    if !exists {
        return Err(ItemError::NotFound);
    }
    sqlx::query!(r#"DELETE FROM items WHERE item_id = $1"#, item_id)
        .execute(&mut *transaction)
        .await
        .context("Failed to delete item")?;
    transaction
        .commit()
        .await
        .context("Failed to commit item deletion transaction")?;
    if let Err(e) = image_service
        .delete_all_item_images(&item_id, pool.get_ref())
        .await
    {
        tracing::warn!(
            "Failed to clean up image files for item {}: {:?}",
            item_id,
            e
        );
    }
    Ok(HttpResponse::NoContent().finish())
}

async fn insert_item(
    new_item: &NewItem,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), ItemError> {
    sqlx::query_unchecked!(
        r#"INSERT INTO items (item_id, listing_type, name, price, currently, buy_price, number_of_bids,
           location, country, latitude, longitude, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)"#,
        new_item.item_id,
        new_item.listing_type,
        new_item.name,
        new_item.price,
        new_item.currently,
        new_item.buy_price,
        new_item.number_of_bids,
        new_item.location,
        new_item.country,
        new_item.latitude,
        new_item.longitude,
        new_item.started.naive_utc(),
        new_item.ends.map(|dt| dt.naive_utc()),
        new_item.description,
        new_item.seller_user_id,
        new_item.seller_rating,
        new_item.condition,
        new_item.shipping_cost
    )
    .execute(&mut **transaction)
    .await
    .context("Failed to insert item")?;

    insert_item_categories(&new_item.item_id, &new_item.categories, transaction).await?;

    Ok(())
}

async fn insert_item_categories(
    item_id: &str,
    categories: &[String],
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), ItemError> {
    for category_name in categories {
        let category_id = sqlx::query!(
            r#"SELECT id FROM categories WHERE name = $1"#,
            category_name
        )
        .fetch_optional(&mut **transaction)
        .await
        .context("Failed to fetch category")?
        .ok_or(ItemError::CategoriesNotFound)?
        .id;

        sqlx::query!(
            r#"INSERT INTO item_categories (item_id, category_id) VALUES ($1, $2)"#,
            item_id,
            category_id
        )
        .execute(&mut **transaction)
        .await
        .context("Failed to insert item category")?;
    }
    Ok(())
}

async fn get_images_for_items(
    item_ids: &[String],
    pool: &PgPool,
) -> Result<HashMap<String, Vec<ItemImageResponse>>, anyhow::Error> {
    if item_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query!(
        r#"SELECT id, item_id, filename, original_name, display_order, file_size, mime_type, upload_timestamp
           FROM item_images 
           WHERE item_id = ANY($1)
           ORDER BY item_id, display_order"#,
        item_ids
    )
    .fetch_all(pool)
    .await
    .context("Failed to batch fetch item images")?;

    let mut item_images = HashMap::new();
    for row in rows {
        let item_image = ItemImage {
            id: row.id,
            item_id: row.item_id.unwrap_or_default(),
            filename: row.filename,
            original_name: row.original_name,
            display_order: row.display_order,
            file_size: row.file_size,
            mime_type: row.mime_type,
            upload_timestamp: row
                .upload_timestamp
                .map(|ts| ts.and_utc())
                .unwrap_or_else(chrono::Utc::now),
        };

        let item_id = item_image.item_id.clone();
        let response_image = ItemImageResponse::from(item_image);
        item_images
            .entry(item_id)
            .or_insert_with(Vec::new)
            .push(response_image);
    }
    Ok(item_images)
}

async fn get_item_categories(item_id: &str, pool: &PgPool) -> Result<Vec<String>, ItemError> {
    Ok(sqlx::query!(
        r#"SELECT c.name FROM categories c
           JOIN item_categories ic ON c.id = ic.category_id
           WHERE ic.item_id = $1
           ORDER BY c.name"#,
        item_id
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch item categories")?
    .into_iter()
    .map(|row| row.name)
    .collect())
}

async fn get_item_with_categories_and_images(
    item_id: &str,
    pool: &PgPool,
    image_service: &ImageService,
) -> Result<Item, ItemError> {
    let item_row = sqlx::query!(
        r#"SELECT item_id, listing_type, name, price, currently, buy_price, number_of_bids,
           location, country, latitude, longitude, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost, status
           FROM items WHERE item_id = $1"#,
        item_id
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch item")?
    .ok_or(ItemError::NotFound)?;

    let categories = get_item_categories(item_id, pool).await?;
    let images = image_service
        .get_item_images(item_id, pool)
        .await
        .context("Failed to fetch item images")?
        .into_iter()
        .map(ItemImageResponse::from)
        .collect();

    Ok(Item {
        item_id: item_row.item_id,
        listing_type: item_row.listing_type,
        name: item_row.name,
        price: item_row.price,
        currently: item_row.currently,
        buy_price: item_row.buy_price,
        number_of_bids: item_row.number_of_bids,
        location: item_row.location,
        country: item_row.country,
        latitude: item_row.latitude,
        longitude: item_row.longitude,
        started: item_row.started.and_utc(),
        ends: item_row.ends.map(|dt| dt.and_utc()),
        description: item_row.description,
        seller_user_id: item_row.seller_user_id.unwrap_or_default(),
        seller_rating: item_row.seller_rating,
        condition: item_row.condition,
        shipping_cost: item_row.shipping_cost,
        categories,
        images,
        status: item_row.status,
    })
}

async fn get_item_with_categories(item_id: &str, pool: &PgPool) -> Result<Item, ItemError> {
    let item_row = sqlx::query!(
        r#"SELECT item_id, listing_type, name, price, currently, buy_price, number_of_bids,
           location, country, latitude, longitude, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost, status
           FROM items WHERE item_id = $1"#,
        item_id
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch item")?
    .ok_or(ItemError::NotFound)?;

    let categories = get_item_categories(item_id, pool).await?;

    Ok(Item {
        item_id: item_row.item_id,
        listing_type: item_row.listing_type,
        name: item_row.name,
        price: item_row.price,
        currently: item_row.currently,
        buy_price: item_row.buy_price,
        number_of_bids: item_row.number_of_bids,
        location: item_row.location,
        country: item_row.country,
        latitude: item_row.latitude,
        longitude: item_row.longitude,
        started: item_row.started.and_utc(),
        ends: item_row.ends.map(|dt| dt.and_utc()),
        description: item_row.description,
        seller_user_id: item_row.seller_user_id.unwrap_or_default(),
        seller_rating: item_row.seller_rating,
        condition: item_row.condition,
        shipping_cost: item_row.shipping_cost,
        categories,
        images: Vec::new(),
        status: item_row.status,
    })
}

#[derive(Debug, Serialize)]
pub struct PurchaseResponse {
    success: bool,
    message: String,
    purchase_info: Option<PurchaseInfo>,
}

#[derive(Debug, Serialize)]
pub struct PurchaseInfo {
    item_id: String,
    buyer_user_id: Uuid,
    seller_user_id: Uuid,
    purchase_price: BigDecimal,
    purchased_at: DateTime<Utc>,
    item_name: String,
}

impl From<crate::services::purchase_service::PurchaseInfo> for PurchaseInfo {
    fn from(info: crate::services::purchase_service::PurchaseInfo) -> Self {
        Self {
            item_id: info.item_id,
            buyer_user_id: info.buyer_user_id,
            seller_user_id: info.seller_user_id,
            purchase_price: info.purchase_price,
            purchased_at: info.purchased_at,
            item_name: info.item_name,
        }
    }
}

define_route_error! {
    PurchaseError {
        NotFound => (StatusCode::NOT_FOUND, "Item not found"),
        NotAvailable => (StatusCode::BAD_REQUEST, "Item is not available for purchase"),
        AlreadySold => (StatusCode::CONFLICT, "Item has already been sold"),
        IsAuction => (StatusCode::BAD_REQUEST, "This item is an auction, use bidding instead"),
        UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
        CannotPurchaseOwn => (StatusCode::BAD_REQUEST, "Cannot purchase your own item"),
    }
}

#[tracing::instrument(name = "Purchase item", skip(pool, websocket_service, claims))]
pub async fn purchase_item(
    path: web::Path<String>,
    pool: web::Data<PgPool>,
    websocket_service: web::Data<WebSocketService>,
    claims: Claims,
) -> Result<HttpResponse, PurchaseError> {
    let item_id = path.into_inner();
    let buyer_user_id = Uuid::parse_str(&claims.sub).map_err(|_| PurchaseError::UserNotFound)?;

    match PurchaseService::purchase_item(&pool, &websocket_service, &item_id, buyer_user_id).await {
        Ok(purchase_info) => {
            let response = PurchaseResponse {
                success: true,
                message: format!("Successfully purchased item '{}'", purchase_info.item_name),
                purchase_info: Some(purchase_info.into()),
            };

            tracing::info!("Item {} purchased by user {}", item_id, buyer_user_id);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            tracing::warn!(
                "Purchase failed for item {} by user {}: {}",
                item_id,
                buyer_user_id,
                e
            );

            let (status_code, message) = match e {
                crate::services::purchase_service::PurchaseServiceError::ItemNotFound => {
                    (StatusCode::NOT_FOUND, "Item not found")
                }
                crate::services::purchase_service::PurchaseServiceError::ItemNotAvailable => (
                    StatusCode::BAD_REQUEST,
                    "Item is not available for purchase",
                ),
                crate::services::purchase_service::PurchaseServiceError::ItemAlreadySold => {
                    (StatusCode::CONFLICT, "Item has already been sold")
                }
                crate::services::purchase_service::PurchaseServiceError::ItemIsAuction => (
                    StatusCode::BAD_REQUEST,
                    "This item is an auction, use bidding instead",
                ),
                crate::services::purchase_service::PurchaseServiceError::UserNotFound => {
                    (StatusCode::NOT_FOUND, "User not found")
                }
                crate::services::purchase_service::PurchaseServiceError::CannotPurchaseOwnItem => {
                    (StatusCode::BAD_REQUEST, "Cannot purchase your own item")
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "Purchase failed"),
            };

            let response = PurchaseResponse {
                success: false,
                message: message.to_string(),
                purchase_info: None,
            };

            Ok(HttpResponse::build(status_code).json(response))
        }
    }
}
