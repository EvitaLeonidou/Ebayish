use teddy_domain::ItemImage;
use crate::handlers::catalog::images::ItemImageResponse;
use teddy_services::ImageService;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;
use crate::errors::catalog::ItemError;

#[derive(Serialize)]
pub struct Item {
    item_id: String,
    name: String,
    first_bid: BigDecimal,
    currently: BigDecimal,
    buy_price: Option<BigDecimal>,
    number_of_bids: i32,
    location: Option<String>,
    country: Option<String>,
    started: DateTime<Utc>,
    ends: DateTime<Utc>,
    description: Option<String>,
    seller_user_id: Uuid,
    seller_rating: Option<BigDecimal>,
    condition: Option<String>,
    shipping_cost: BigDecimal,
    categories: Vec<String>,
    images: Vec<ItemImageResponse>,
}


#[tracing::instrument(name = "Get all items", skip(pool))]
pub async fn get_items(
    pool: web::Data<PgPool>,
    _image_service: web::Data<ImageService>,
) -> Result<HttpResponse, ItemError> {
    let items = sqlx::query!(
        r#"SELECT item_id, name, first_bid, currently, buy_price, number_of_bids,
           location, country, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost
           FROM items ORDER BY created_at DESC"#,
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch items")?;

    // Collect all item IDs for batch image loading
    let item_ids: Vec<String> = items.iter().map(|item| item.item_id.clone()).collect();

    // Batch load all images
    let all_images = get_images_for_items(&item_ids, pool.get_ref())
        .await
        .context("Failed to batch load item images")?;

    let mut result_items = Vec::new();

    for item_row in items {
        let categories = get_item_categories(&item_row.item_id, &pool).await?;

        // Get images from the batch-loaded map
        let images = all_images
            .get(&item_row.item_id)
            .cloned()
            .unwrap_or_default();

        result_items.push(Item {
            item_id: item_row.item_id,
            name: item_row.name,
            first_bid: item_row.first_bid,
            currently: item_row.currently,
            buy_price: item_row.buy_price,
            number_of_bids: item_row.number_of_bids.unwrap_or(0),
            location: item_row.location,
            country: item_row.country,
            started: item_row.started.and_utc(),
            ends: item_row.ends.and_utc(),
            description: item_row.description,
            seller_user_id: item_row.seller_user_id.unwrap_or_default(),
            seller_rating: item_row.seller_rating,
            condition: item_row.condition,
            shipping_cost: item_row.shipping_cost,
            categories,
            images,
        });
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

/// Load images for multiple items efficiently
async fn get_images_for_items(
    item_ids: &[String],
    pool: &PgPool,
) -> Result<HashMap<String, Vec<ItemImageResponse>>, anyhow::Error> {
    if item_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Use a single query to get all images for all items
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

    // Group images by item_id
    let mut item_images = HashMap::new();
    for row in rows {
        #[allow(clippy::redundant_closure)]
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
                .unwrap_or_else(|| chrono::Utc::now()),
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

pub async fn get_item_categories(item_id: &str, pool: &PgPool) -> Result<Vec<String>, ItemError> {
    let categories = sqlx::query!(
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
    .collect();

    Ok(categories)
}

pub async fn get_item_with_categories_and_images(
    item_id: &str,
    pool: &PgPool,
    image_service: &ImageService,
) -> Result<Item, ItemError> {
    let item_row = sqlx::query!(
        r#"SELECT item_id, name, first_bid, currently, buy_price, number_of_bids,
           location, country, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost
           FROM items WHERE item_id = $1"#,
        item_id
    )
    .fetch_optional(pool)
    .await
    .context("Failed to fetch item")?
    .ok_or(ItemError::NotFound)?;

    let categories = get_item_categories(item_id, pool).await?;

    // Get images for the item
    let images = image_service
        .get_item_images(item_id, pool)
        .await
        .context("Failed to fetch item images")?
        .into_iter()
        .map(ItemImageResponse::from)
        .collect();

    Ok(Item {
        item_id: item_row.item_id,
        name: item_row.name,
        first_bid: item_row.first_bid,
        currently: item_row.currently,
        buy_price: item_row.buy_price,
        number_of_bids: item_row.number_of_bids.unwrap_or(0),
        location: item_row.location,
        country: item_row.country,
        started: item_row.started.and_utc(),
        ends: item_row.ends.and_utc(),
        description: item_row.description,
        seller_user_id: item_row.seller_user_id.unwrap_or_default(),
        seller_rating: item_row.seller_rating,
        condition: item_row.condition,
        shipping_cost: item_row.shipping_cost,
        categories,
        images,
    })
}

// Keep the old function for backwards compatibility during transition
pub async fn get_item_with_categories(item_id: &str, pool: &PgPool) -> Result<Item, ItemError> {
    let item_row = sqlx::query!(
        r#"SELECT item_id, name, first_bid, currently, buy_price, number_of_bids,
           location, country, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost
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
        name: item_row.name,
        first_bid: item_row.first_bid,
        currently: item_row.currently,
        buy_price: item_row.buy_price,
        number_of_bids: item_row.number_of_bids.unwrap_or(0),
        location: item_row.location,
        country: item_row.country,
        started: item_row.started.and_utc(),
        ends: item_row.ends.and_utc(),
        description: item_row.description,
        seller_user_id: item_row.seller_user_id.unwrap_or_default(),
        seller_rating: item_row.seller_rating,
        condition: item_row.condition,
        shipping_cost: item_row.shipping_cost,
        categories,
        images: Vec::new(), // Empty images for backwards compatibility
    })
}