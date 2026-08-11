// teddy-web/src/handlers/users/profile.rs

use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::users::PublicUserInfo;
use crate::errors::users::GetUserError;

// --- FUNCTION TO HANDLE GET /users/{user_id} ---
#[tracing::instrument(name = "Get user by ID", skip(pool, path))]
pub async fn get_user_by_id(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, GetUserError> {
    let user_id = path.into_inner();

    let user_info = sqlx::query_as!(
        PublicUserInfo,
        "SELECT id, username, seller_rating, created_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .context("Failed to fetch user by ID")?
    .ok_or(GetUserError::NotFound)?;

    Ok(HttpResponse::Ok().json(user_info))
}