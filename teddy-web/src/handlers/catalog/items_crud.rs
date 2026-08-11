use teddy_domain::NewItem;
use teddy_services::ImageService;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use crate::errors::catalog::ItemError;

#[derive(Debug, Deserialize)]
pub struct ItemRequest {
    item_id: Option<String>,
    name: String,
    first_bid: BigDecimal,
    currently: BigDecimal,
    buy_price: Option<BigDecimal>,
    number_of_bids: Option<i32>,
    location: Option<String>,
    country: Option<String>,
    started: DateTime<Utc>,
    ends: DateTime<Utc>,
    description: Option<String>,
    seller_user_id: Uuid,
    condition: Option<String>,
    shipping_cost: Option<BigDecimal>,
    categories: Vec<String>,
}

impl ItemRequest {
    fn into_new_item_with_seller_rating(
        self,
        seller_rating: Option<BigDecimal>,
    ) -> Result<NewItem, String> {
        let item = NewItem {
            item_id: self.item_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: self.name,
            first_bid: self.first_bid,
            currently: self.currently,
            buy_price: self.buy_price,
            number_of_bids: self.number_of_bids.unwrap_or(0),
            location: self.location,
            country: self.country,
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


#[tracing::instrument(name = "Create item", skip(json, pool))]
pub async fn create_item(
    json: web::Json<ItemRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ItemError> {
    // Fetch the seller's rating from the users table
    let seller_rating = sqlx::query_scalar!(
        "SELECT seller_rating FROM users WHERE id = $1",
        json.seller_user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch seller rating")?
    .flatten(); // This handles the Option<Option<BigDecimal>> -> Option<BigDecimal>

    let new_item: NewItem = json
        .0
        .into_new_item_with_seller_rating(seller_rating)
        .map_err(|_| ItemError::ValidationError)?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;

    insert_item(&new_item, &mut transaction).await?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction for new item.")?;

    let item = crate::handlers::catalog::items_query::get_item_with_categories(&new_item.item_id, &pool).await?;
    Ok(HttpResponse::Created().json(item))
}

#[tracing::instrument(name = "Update item", skip(json, pool, image_service))]
pub async fn update_item(
    path: web::Path<String>,
    json: web::Json<ItemRequest>,
    pool: web::Data<PgPool>,
    image_service: web::Data<ImageService>,
) -> Result<HttpResponse, ItemError> {
    let item_id = path.into_inner();

    // Fetch the seller's rating from the users table
    let seller_rating = sqlx::query_scalar!(
        "SELECT seller_rating FROM users WHERE id = $1",
        json.seller_user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch seller rating")?
    .flatten(); // This handles the Option<Option<BigDecimal>> -> Option<BigDecimal>

    let updated_item: NewItem = json
        .0
        .into_new_item_with_seller_rating(seller_rating)
        .map_err(|_| ItemError::ValidationError)?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;

    let rows_affected = sqlx::query!(
        r#"UPDATE items SET name = $1, first_bid = $2, currently = $3, buy_price = $4,
           number_of_bids = $5, location = $6, country = $7, started = $8, ends = $9,
           description = $10, seller_user_id = $11, seller_rating = $12, condition = $13
           WHERE item_id = $14"#,
        updated_item.name,
        updated_item.first_bid,
        updated_item.currently,
        updated_item.buy_price,
        updated_item.number_of_bids,
        updated_item.location,
        updated_item.country,
        updated_item.started.naive_utc(),
        updated_item.ends.naive_utc(),
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

    // Update categories
    sqlx::query!(r#"DELETE FROM item_categories WHERE item_id = $1"#, item_id)
        .execute(&mut *transaction)
        .await
        .context("Failed to delete existing item categories")?;

    insert_item_categories(&item_id, &updated_item.categories, &mut transaction).await?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction for item update.")?;

    let item = crate::handlers::catalog::items_query::get_item_with_categories_and_images(&item_id, pool.get_ref(), image_service.get_ref())
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

    // Start a transaction for consistency
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;

    // Check if item exists before deletion
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

    // Delete the item (images will be deleted by CASCADE)
    sqlx::query!(r#"DELETE FROM items WHERE item_id = $1"#, item_id)
        .execute(&mut *transaction)
        .await
        .context("Failed to delete item")?;

    // Commit database transaction
    transaction
        .commit()
        .await
        .context("Failed to commit item deletion transaction")?;

    // Clean up image files from filesystem
    // Note: This happens after database commit to avoid inconsistency
    // if the filesystem cleanup fails
    if let Err(e) = image_service
        .delete_all_item_images(&item_id, pool.get_ref())
        .await
    {
        tracing::warn!(
            "Failed to clean up image files for item {}: {:?}",
            item_id,
            e
        );
        // Don't fail the request since the database deletion succeeded
    }

    Ok(HttpResponse::NoContent().finish())
}

async fn insert_item(
    new_item: &NewItem,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), ItemError> {
    sqlx::query!(
        r#"INSERT INTO items (item_id, name, first_bid, currently, buy_price, number_of_bids,
           location, country, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"#,
        new_item.item_id,
        new_item.name,
        new_item.first_bid,
        new_item.currently,
        new_item.buy_price,
        new_item.number_of_bids,
        new_item.location,
        new_item.country,
        new_item.started.naive_utc(),
        new_item.ends.naive_utc(),
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