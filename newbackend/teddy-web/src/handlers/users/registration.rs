// teddy-web/src/handlers/users/registration.rs

use actix_web::{HttpResponse, web};
use anyhow::Context;
use secrecy::Secret;
use sqlx::PgPool;

use teddy_domain::{NewUser, UserEmail, Username, hash_password};
use teddy_services::UserService;
use crate::dto::users::User;
use crate::errors::users::CreateUserError;

// Helper function to convert User DTO to domain NewUser
fn convert_user_dto_to_domain(user: User) -> Result<NewUser, String> {
    let username = Username::parse(user.username)?;
    let email = UserEmail::parse(user.email)?;
    let first_name = Username::parse(user.first_name)?;
    let last_name = Username::parse(user.last_name)?;
    Ok(NewUser {
        username,
        email,
        password_hash: user.password_hash,
        first_name,
        last_name,
        phone: user.phone,
        date_of_birth: user.date_of_birth,
        seller_rating: user.seller_rating,
    })
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
    let mut new_user: NewUser = convert_user_dto_to_domain(json.into_inner())
        .map_err(|_| CreateUserError::ValidationError)?;

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