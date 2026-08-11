//! rc/routes/users.rs
use crate::define_route_error;
use crate::domain::{NewUser, UserEmail, Username, hash_password};
use crate::jwt_middleware::Claims;
use crate::services::user_service::{UserService, UserServiceError};
use actix_web::{HttpResponse, web};
use anyhow::Context;
use chrono::NaiveDate;
use reqwest::StatusCode;
use secrecy::Secret;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct User {
    username: String,
    email: String,
    password_hash: String,
    first_name: String,
    last_name: String,
    phone: String,
    date_of_birth: NaiveDate,
    seller_rating: Option<bigdecimal::BigDecimal>,
    tax_id: Option<String>,
    location: Option<String>,
    country: Option<String>,
}

impl TryFrom<web::Json<User>> for NewUser {
    type Error = String;

    fn try_from(value: web::Json<User>) -> Result<NewUser, String> {
        let username = Username::parse(value.0.username)?;
        let email = UserEmail::parse(value.0.email)?;
        let first_name = Username::parse(value.0.first_name)?;
        let last_name = Username::parse(value.0.last_name)?;
        Ok(Self {
            username,
            email,
            password_hash: value.0.password_hash,
            first_name,
            last_name,
            phone: value.0.phone,
            date_of_birth: value.0.date_of_birth,
            seller_rating: value.0.seller_rating,
            tax_id: value.0.tax_id,
            location: value.0.location,
            country: value.0.country,
        })
    }
}

define_route_error! {
    CreateUserError {
        ValidationError => (StatusCode::BAD_REQUEST, "Invalid user data provided"),
        UsernameTaken => (StatusCode::BAD_REQUEST, "Username is taken"),
    }
}

#[tracing::instrument(
    name = "Adding a new user",
    skip(json, pool),
    fields(
        user_email = %json.username,
        user_name = %json.email
    )
)]
pub async fn create_user(
    json: web::Json<User>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, CreateUserError> {
    let mut new_user: NewUser = json
        .try_into()
        .map_err(|_| CreateUserError::ValidationError)?;

    //necessary for unwrap_or_else(|| false)
    #[allow(clippy::unnecessary_lazy_evaluations)]
    let username_taken: bool = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)",
        new_user.username.as_ref()
    )
    .fetch_one(&**pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to check if username exists {:?}", e);
        CreateUserError::UsernameTaken
    })?
    .unwrap_or_else(|| false);

    if username_taken {
        return Err(CreateUserError::UsernameTaken);
    }
    let password_hash =
        hash_password(Secret::new(new_user.password_hash)).context("Failed to hash password")?;
    new_user.password_hash = password_hash;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")?;
    let _user_id = UserService::insert_user(&new_user, &mut transaction)
        .await
        .context("Failed to insert new user in the database.")?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction for new user.")?;

    Ok(HttpResponse::Ok().finish())
}

define_route_error! {
    VerifyUserError {
        UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
        AuthenticationError => (StatusCode::FORBIDDEN, "Admin access required"),
    }
}

#[tracing::instrument(
    name = "Verify pending user",
    skip(pool, path),
    fields(user_id = %path.as_ref())
)]
pub async fn verify_user(
    claims: Claims,
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, VerifyUserError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(VerifyUserError::AuthenticationError);
    }
    let user_id = path.into_inner();

    UserService::verify_user(&user_id, pool.get_ref())
        .await
        .map_err(|e| match e {
            UserServiceError::NotFound => VerifyUserError::UserNotFound,
            _ => VerifyUserError::UnexpectedError(anyhow::anyhow!("Service error: {}", e)),
        })?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get pending users", skip(pool))]
pub async fn get_pending_users(
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, VerifyUserError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(VerifyUserError::AuthenticationError);
    }
    let users = UserService::get_pending_users(pool.get_ref())
        .await
        .map_err(|e| match e {
            UserServiceError::NotFound => VerifyUserError::UserNotFound,
            _ => VerifyUserError::UnexpectedError(anyhow::anyhow!("Service error: {}", e)),
        })?;

    Ok(HttpResponse::Ok().json(users))
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub date_of_birth: NaiveDate,
    pub status: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub seller_rating: Option<bigdecimal::BigDecimal>,
}

define_route_error! {
    SuspendUserError {
        UserNotFound => (StatusCode::NOT_FOUND, "User not found or already suspended"),
        AuthenticationError => (StatusCode::FORBIDDEN, "Admin access required"),
    }
}

define_route_error! {
    ActivateUserError {
        UserNotFound => (StatusCode::NOT_FOUND, "User not found or not suspended"),
        AuthenticationError => (StatusCode::FORBIDDEN, "Admin access required"),
    }
}

define_route_error! {
    GetAllUsersError {
        DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred"),
        AuthenticationError => (StatusCode::FORBIDDEN, "Admin access required"),
    }
}

#[derive(Serialize)]
pub struct PublicUserInfo {
    id: Uuid,
    username: String,
    seller_rating: Option<bigdecimal::BigDecimal>,
    created_at: chrono::DateTime<chrono::Utc>,
}

define_route_error! {
    GetUserError {
        NotFound => (StatusCode::NOT_FOUND, "User not found"),
    }
}

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

#[tracing::instrument(name = "Get all users", skip(pool))]
pub async fn get_all_users(
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, GetAllUsersError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(GetAllUsersError::AuthenticationError);
    }
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
    claims: Claims,
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, SuspendUserError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(SuspendUserError::AuthenticationError);
    }
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
    claims: Claims,
    path: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ActivateUserError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(ActivateUserError::AuthenticationError);
    }
    let user_id = path.into_inner();

    UserService::activate_user(&user_id, pool.get_ref())
        .await
        .map_err(|e| match e {
            UserServiceError::NotFound => ActivateUserError::UserNotFound,
            _ => ActivateUserError::UnexpectedError(anyhow::anyhow!("Service error: {}", e)),
        })?;

    Ok(HttpResponse::Ok().finish())
}
