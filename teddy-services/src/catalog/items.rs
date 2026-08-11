use teddy_domain::entities::NewItem;
use crate::error_handling::error_chain_fmt;
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum ItemServiceError {
    #[error("Item not found")]
    NotFound,
    #[error("Categories not found")]
    CategoriesNotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ItemServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Serialize)]
pub struct Item {
    pub item_id: String,
    pub name: String,
    pub first_bid: BigDecimal,
    pub currently: BigDecimal,
    pub buy_price: Option<BigDecimal>,
    pub number_of_bids: i32,
    pub location: Option<String>,
    pub country: Option<String>,
    pub started: DateTime<Utc>,
    pub ends: DateTime<Utc>,
    pub description: Option<String>,
    pub seller_user_id: Uuid,
    pub seller_rating: Option<BigDecimal>,
    pub condition: Option<String>,
    pub shipping_cost: BigDecimal,
    pub categories: Vec<String>,
}
#[derive(Debug)]
pub struct ItemService;

impl ItemService {
    #[tracing::instrument(name = "Insert item", skip(new_item, transaction))]
    pub async fn insert_item(
        new_item: &NewItem,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), ItemServiceError> {
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

        Self::insert_item_categories(&new_item.item_id, &new_item.categories, transaction).await?;

        Ok(())
    }

    #[tracing::instrument(name = "Get all items", skip(pool))]
    pub async fn get_all_items(pool: &PgPool) -> Result<Vec<Item>, ItemServiceError> {
        let items = sqlx::query!(
            r#"SELECT item_id, name, first_bid, currently, buy_price, number_of_bids,
               location, country, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost
               FROM items ORDER BY created_at DESC"#,
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch items")?;

        let mut result_items = Vec::new();

        for item_row in items {
            let categories = Self::get_item_categories(&item_row.item_id, pool).await?;

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
            });
        }

        Ok(result_items)
    }

    #[tracing::instrument(name = "Get item by ID", skip(pool))]
    pub async fn get_item_by_id(item_id: &str, pool: &PgPool) -> Result<Item, ItemServiceError> {
        let item_row = sqlx::query!(
            r#"SELECT item_id, name, first_bid, currently, buy_price, number_of_bids,
               location, country, started, ends, description, seller_user_id, seller_rating, condition, shipping_cost
               FROM items WHERE item_id = $1"#,
            item_id
        )
        .fetch_optional(pool)
        .await
        .context("Failed to fetch item")?
        .ok_or(ItemServiceError::NotFound)?;

        let categories = Self::get_item_categories(&item_row.item_id, pool).await?;

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
        })
    }

    #[tracing::instrument(name = "Update item", skip(updated_item, pool))]
    pub async fn update_item(
        item_id: &str,
        updated_item: &NewItem,
        pool: &PgPool,
    ) -> Result<Item, ItemServiceError> {
        let mut transaction = pool
            .begin()
            .await
            .context("Failed to acquire a Postgres connection from the pool.")?;

        let rows_affected = sqlx::query!(
            r#"UPDATE items SET name = $1, first_bid = $2, currently = $3, buy_price = $4,
               location = $5, country = $6, started = $7, ends = $8, description = $9,
               seller_user_id = $10, seller_rating = $11, condition = $12, shipping_cost = $13
               WHERE item_id = $14"#,
            updated_item.name,
            updated_item.first_bid,
            updated_item.currently,
            updated_item.buy_price,
            updated_item.location,
            updated_item.country,
            updated_item.started.naive_utc(),
            updated_item.ends.naive_utc(),
            updated_item.description,
            updated_item.seller_user_id,
            updated_item.seller_rating,
            updated_item.condition,
            updated_item.shipping_cost,
            item_id
        )
        .execute(&mut *transaction)
        .await
        .context("Failed to update item")?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ItemServiceError::NotFound);
        }

        // Update categories
        sqlx::query!(r#"DELETE FROM item_categories WHERE item_id = $1"#, item_id)
            .execute(&mut *transaction)
            .await
            .context("Failed to delete existing item categories")?;

        Self::insert_item_categories(item_id, &updated_item.categories, &mut transaction).await?;

        transaction
            .commit()
            .await
            .context("Failed to commit SQL transaction for item update.")?;

        Self::get_item_by_id(item_id, pool).await
    }

    #[tracing::instrument(name = "Delete item", skip(pool))]
    pub async fn delete_item(item_id: &str, pool: &PgPool) -> Result<(), ItemServiceError> {
        let rows_affected = sqlx::query!(r#"DELETE FROM items WHERE item_id = $1"#, item_id)
            .execute(pool)
            .await
            .context("Failed to delete item")?
            .rows_affected();

        if rows_affected == 0 {
            return Err(ItemServiceError::NotFound);
        }

        Ok(())
    }

    async fn insert_item_categories(
        item_id: &str,
        categories: &[String],
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<(), ItemServiceError> {
        for category_name in categories {
            let category_id = sqlx::query!(
                r#"SELECT id FROM categories WHERE name = $1"#,
                category_name
            )
            .fetch_optional(&mut **transaction)
            .await
            .context("Failed to fetch category")?
            .ok_or(ItemServiceError::CategoriesNotFound)?
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

    async fn get_item_categories(
        item_id: &str,
        pool: &PgPool,
    ) -> Result<Vec<String>, ItemServiceError> {
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
}