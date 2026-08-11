use crate::error_handling::error_chain_fmt;
use anyhow::Context;
use serde::Serialize;
use sqlx::PgPool;

#[derive(thiserror::Error)]
pub enum CategoryServiceError {
    #[error("Category not found")]
    NotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for CategoryServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(Serialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

pub struct CategoryService;

impl CategoryService {
    #[tracing::instrument(name = "Get all categories", skip(pool))]
    pub async fn get_all_categories(pool: &PgPool) -> Result<Vec<Category>, CategoryServiceError> {
        let categories =
            sqlx::query_as!(Category, r#"SELECT id, name FROM categories ORDER BY name"#)
                .fetch_all(pool)
                .await
                .context("Failed to fetch categories")?;

        Ok(categories)
    }

    #[tracing::instrument(name = "Get category by ID", skip(pool))]
    pub async fn get_category_by_id(
        id: &i32,
        pool: &PgPool,
    ) -> Result<Category, CategoryServiceError> {
        let category = sqlx::query_as!(
            Category,
            r#"SELECT id, name FROM categories WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
        .context("Failed to fetch category")?
        .ok_or(CategoryServiceError::NotFound)?;

        Ok(category)
    }

    #[tracing::instrument(name = "Create category", skip(pool))]
    pub async fn create_category(
        name: &str,
        pool: &PgPool,
    ) -> Result<Category, CategoryServiceError> {
        let category = sqlx::query_as!(
            Category,
            r#"INSERT INTO categories (name) VALUES ($1) RETURNING id, name"#,
            name
        )
        .fetch_one(pool)
        .await
        .context("Failed to create category")?;

        Ok(category)
    }

    #[tracing::instrument(name = "Update category", skip(pool))]
    pub async fn update_category(
        id: &i32,
        name: &str,
        pool: &PgPool,
    ) -> Result<Category, CategoryServiceError> {
        let rows_affected =
            sqlx::query!(r#"UPDATE categories SET name = $1 WHERE id = $2"#, name, id)
                .execute(pool)
                .await
                .context("Failed to update category")?
                .rows_affected();

        if rows_affected == 0 {
            return Err(CategoryServiceError::NotFound);
        }

        Ok(Category {
            id: *id,
            name: name.to_string(),
        })
    }

    #[tracing::instrument(name = "Delete category", skip(pool))]
    pub async fn delete_category(id: &i32, pool: &PgPool) -> Result<(), CategoryServiceError> {
        let rows_affected = sqlx::query!(r#"DELETE FROM categories WHERE id = $1"#, id)
            .execute(pool)
            .await
            .context("Failed to delete category")?
            .rows_affected();

        if rows_affected == 0 {
            return Err(CategoryServiceError::NotFound);
        }

        Ok(())
    }
}