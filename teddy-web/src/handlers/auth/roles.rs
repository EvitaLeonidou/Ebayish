use crate::define_route_error;
use crate::middleware::jwt::AuthenticatedUser;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Serialize)]
struct UserRole {
    role: String,
}

define_route_error! {
    UserRoleError {
        InvalidUserId => (StatusCode::BAD_REQUEST, "Invalid user ID format"),
        UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
        DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database operation failed"),
    }
}

#[tracing::instrument(name = "Get user role", skip(user, pool))]
pub async fn user_role(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, UserRoleError> {
    let user_id = Uuid::parse_str(&user.user_id).map_err(|_| UserRoleError::InvalidUserId)?;

    let role = sqlx::query_as!(UserRole, "SELECT role FROM users WHERE id = $1", user_id)
        .fetch_optional(pool.get_ref())
        .await
        .context("Failed to fetch user role")?
        .ok_or(UserRoleError::UserNotFound)?;

    Ok(HttpResponse::Ok().json(role))
}