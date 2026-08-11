use crate::define_route_error;
use crate::domain::{CategoryName, NewCategory};
use actix_web::{HttpResponse, web};
use anyhow::Context;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct CategoryRequest {
    name: String,
}

#[derive(Serialize)]
pub struct Category {
    id: i32,
    name: String,
}

impl TryFrom<CategoryRequest> for NewCategory {
    type Error = String;

    fn try_from(value: CategoryRequest) -> Result<NewCategory, String> {
        let name = CategoryName::parse(value.name)?;
        Ok(Self { name })
    }
}

define_route_error! {
    CategoryError {
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid category data provided"),
        NotFound => (StatusCode::NOT_FOUND, "Category not found"),
    }
}

#[tracing::instrument(name = "Create category", skip(json, pool))]
pub async fn create_category(
    json: web::Json<CategoryRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CategoryError> {
    let new_category: NewCategory = json
        .0
        .try_into()
        .map_err(|_| CategoryError::ValidationError)?;

    let category_id = sqlx::query!(
        r#"INSERT INTO categories (name) VALUES ($1) RETURNING id"#,
        new_category.name.as_ref()
    )
    .fetch_one(pool.get_ref())
    .await
    .context("Failed to insert category")?
    .id;

    let category = Category {
        id: category_id,
        name: new_category.name.as_ref().to_string(),
    };

    Ok(HttpResponse::Created().json(category))
}

#[tracing::instrument(name = "Get all categories", skip(pool))]
pub async fn get_categories(pool: web::Data<PgPool>) -> Result<HttpResponse, CategoryError> {
    let categories = sqlx::query_as!(Category, r#"SELECT id, name FROM categories ORDER BY name"#,)
        .fetch_all(pool.get_ref())
        .await
        .context("Failed to fetch categories")?;

    Ok(HttpResponse::Ok().json(categories))
}

#[tracing::instrument(name = "Get category by ID", skip(pool))]
pub async fn get_category(
    path: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CategoryError> {
    let category_id = path.into_inner();

    let category = sqlx::query_as!(
        Category,
        r#"SELECT id, name FROM categories WHERE id = $1"#,
        category_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch category")?;

    match category {
        Some(category) => Ok(HttpResponse::Ok().json(category)),
        None => Err(CategoryError::NotFound),
    }
}

#[tracing::instrument(name = "Update category", skip(json, pool))]
pub async fn update_category(
    path: web::Path<i32>,
    json: web::Json<CategoryRequest>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CategoryError> {
    let category_id = path.into_inner();
    let updated_category: NewCategory = json
        .0
        .try_into()
        .map_err(|_| CategoryError::ValidationError)?;

    let rows_affected = sqlx::query!(
        r#"UPDATE categories SET name = $1 WHERE id = $2"#,
        updated_category.name.as_ref(),
        category_id
    )
    .execute(pool.get_ref())
    .await
    .context("Failed to update category")?
    .rows_affected();

    if rows_affected == 0 {
        return Err(CategoryError::NotFound);
    }

    let category = Category {
        id: category_id,
        name: updated_category.name.as_ref().to_string(),
    };

    Ok(HttpResponse::Ok().json(category))
}

#[tracing::instrument(name = "Delete category", skip(pool))]
pub async fn delete_category(
    path: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CategoryError> {
    let category_id = path.into_inner();

    let rows_affected = sqlx::query!(r#"DELETE FROM categories WHERE id = $1"#, category_id)
        .execute(pool.get_ref())
        .await
        .context("Failed to delete category")?
        .rows_affected();

    if rows_affected == 0 {
        return Err(CategoryError::NotFound);
    }

    Ok(HttpResponse::NoContent().finish())
}
