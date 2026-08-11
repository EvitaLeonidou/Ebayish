//! Login endpoint implementation
use crate::define_route_error;
use teddy_domain::Username;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use reqwest::StatusCode;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;
use uuid::Uuid;

define_route_error! {
    LoginError {
        InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid username or password"),
        ValidationError => (StatusCode::BAD_REQUEST, "The username or password is in an invalid format"),
        AccountPending => (StatusCode::FORBIDDEN, "Account is pending approval"),
    }
}

#[derive(serde::Deserialize)]
pub struct Credentials {
    username: String,
    password: Secret<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

#[tracing::instrument(name = "User logging in", skip(credentials, pool))]
pub async fn login(
    credentials: web::Json<Credentials>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, LoginError> {
    let Credentials { username, password } = credentials.0;
    let username = Username::parse(username).map_err(|_| LoginError::ValidationError)?;
    let user_data = get_user_data_from_credentials(&username, &password, &pool)
        .await
        .context("Failed to retrieve user data from credentials")?;
    match user_data {
        None => Err(LoginError::InvalidCredentials),
        Some((user_id, username, role, status)) => {
            if status == "pending" {
                return Err(LoginError::AccountPending);
            }
            // TODO: Load the secret from configuration
            let secret = b"supersecret";
            let claims = Claims {
                sub: user_id.to_string(),
                username: username.to_string(),
                role,
                exp: (Utc::now() + Duration::days(1)).timestamp() as usize,
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(secret),
            )
            .map_err(|e| LoginError::UnexpectedError(e.into()))?;
            Ok(HttpResponse::Ok().json(serde_json::json!({ "token": token })))
        }
    }
}

#[tracing::instrument(
    name = "Get user data from credentials",
    skip(username, password, pool)
)]
pub async fn get_user_data_from_credentials(
    username: &Username,
    password: &Secret<String>,
    pool: &PgPool,
) -> Result<Option<(Uuid, String, String, String)>, anyhow::Error> {
    let user = sqlx::query!(
        r#"SELECT id, username, password_hash, role, status FROM users WHERE username = $1"#,
        username.as_ref()
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials")?;

    match user {
        Some(user) => {
            if verify_password_hash(user.password_hash.into(), password.clone()) {
                Ok(Some((user.id, user.username, user.role, user.status)))
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}

fn verify_password_hash(
    expected_password_hash: Secret<String>,
    password_candidate: Secret<String>,
) -> bool {
    let expected_password_hash = match PasswordHash::new(expected_password_hash.expose_secret()) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(),
            &expected_password_hash,
        )
        .is_ok()
}