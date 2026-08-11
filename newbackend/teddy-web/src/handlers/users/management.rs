// teddy-web/src/handlers/users/management.rs

use actix_web::{HttpResponse, web};
use sqlx::PgPool;
use uuid::Uuid;

use teddy_services::UserService;
use teddy_services::UserServiceError;
use crate::dto::users::UserResponse;
use crate::errors::users::{SuspendUserError, ActivateUserError, GetAllUsersError};

#[tracing::instrument(name = "Get all users", skip(pool))]
pub async fn get_all_users(pool: web::Data<PgPool>) -> Result<HttpResponse, GetAllUsersError> {
    let users = sqlx::query_as!(
        UserResponse,
        "SELECT id, username, email, first_name, last_name, phone, date_of_birth, status, role, created_at, seller_rating FROM users"
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch all users: {:?}", e);
        GetAllUsersError::DatabaseError
    })?;

    Ok(HttpResponse::Ok().json(users))
}

#[tracing::instrument(
    name = "Suspend user",
    skip(pool, path),
    fields(user_id = %path.as_ref())
)]
pub async fn suspend_user(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SuspendUserError> {
    let user_id = path.into_inner();

    UserService::suspend_user(&user_id, pool.get_ref())
        .await
        .map_err(|e| match e {
            UserServiceError::NotFound => SuspendUserError::UserNotFound,
            _ => SuspendUserError::UnexpectedError(anyhow::anyhow!("Service error: {}", e)),
        })?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Activate suspended user",
    skip(pool, path),
    fields(user_id = %path.as_ref())
)]
pub async fn activate_user(
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ActivateUserError> {
    let user_id = path.into_inner();

    UserService::activate_user(&user_id, pool.get_ref())
        .await
        .map_err(|e| match e {
            UserServiceError::NotFound => ActivateUserError::UserNotFound,
            _ => ActivateUserError::UnexpectedError(anyhow::anyhow!("Service error: {}", e)),
        })?;

    Ok(HttpResponse::Ok().finish())
}