use crate::domain::NewUser;
use crate::error_handling::error_chain_fmt;
use anyhow::Context;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum UserServiceError {
    #[error("User not found")]
    NotFound,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Unexpected error: {0}")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for UserServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub struct UserService;

impl UserService {
    #[tracing::instrument(
        name = "Saving new subscriber details in the database",
        skip(new_user, transaction)
    )]
    pub async fn insert_user(
        new_user: &NewUser,
        transaction: &mut Transaction<'_, Postgres>,
    ) -> Result<Uuid, UserServiceError> {
        let user_id = Uuid::new_v4();
        //if seeded users activate on insert
        let status = if new_user.username.as_ref() == "o paliatzis"
            || new_user.username.as_ref() == "o psaras"
        {
            "confirmed"
        } else {
            "pending"
        };
        let query = sqlx::query!(
            r#"
        INSERT INTO users (id, username, email, password_hash,
             first_name, last_name, phone, date_of_birth, status, seller_rating,
             tax_id, location, country)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            user_id,
            new_user.username.as_ref(),
            new_user.email.as_ref(),
            new_user.password_hash,
            new_user.first_name.as_ref(),
            new_user.last_name.as_ref(),
            new_user.phone,
            new_user.date_of_birth,
            status,
            new_user.seller_rating,
            new_user.tax_id,
            new_user.location,
            new_user.country
        );
        transaction.execute(query).await?;
        Ok(user_id)
    }

    #[tracing::instrument(name = "Verify user", skip(pool))]
    pub async fn verify_user(user_id: &Uuid, pool: &PgPool) -> Result<(), UserServiceError> {
        let rows_affected = sqlx::query!(
            r#"UPDATE users SET status = 'confirmed' WHERE id = $1 AND status = 'pending'"#,
            user_id
        )
        .execute(pool)
        .await
        .context("Failed to update user status")?
        .rows_affected();

        if rows_affected == 0 {
            return Err(UserServiceError::NotFound);
        }

        Ok(())
    }

    #[tracing::instrument(name = "Get pending users", skip(pool))]
    pub async fn get_pending_users(pool: &PgPool) -> Result<Vec<PendingUser>, UserServiceError> {
        let users = sqlx::query_as!(
            PendingUser,
            r#"
            SELECT id, username, email, first_name, last_name, phone, date_of_birth, created_at
            FROM users 
            WHERE status = 'pending'
            ORDER BY id ASC
            "#
        )
        .fetch_all(pool)
        .await
        .context("Failed to fetch pending users")?;

        Ok(users)
    }

    #[tracing::instrument(name = "Suspend user", skip(pool))]
    pub async fn suspend_user(user_id: &Uuid, pool: &PgPool) -> Result<(), UserServiceError> {
        let rows_affected = sqlx::query!(
            r#"UPDATE users SET status = 'suspended' WHERE id = $1 AND (status = 'confirmed' OR status = 'pending')"#,
            user_id
        )
        .execute(pool)
        .await
        .context("Failed to update user status to suspended")?
        .rows_affected();

        if rows_affected == 0 {
            return Err(UserServiceError::NotFound);
        }

        Ok(())
    }

    #[tracing::instrument(name = "Activate user", skip(pool))]
    pub async fn activate_user(user_id: &Uuid, pool: &PgPool) -> Result<(), UserServiceError> {
        let rows_affected = sqlx::query!(
            r#"UPDATE users SET status = 'confirmed' WHERE id = $1 AND status = 'suspended'"#,
            user_id
        )
        .execute(pool)
        .await
        .context("Failed to update user status to confirmed")?
        .rows_affected();

        if rows_affected == 0 {
            return Err(UserServiceError::NotFound);
        }

        Ok(())
    }

    #[tracing::instrument(name = "Get user by credentials", skip(pool))]
    pub async fn get_user_by_credentials(
        username: &str,
        password_hash: &str,
        pool: &PgPool,
    ) -> Result<Option<UserCredentials>, UserServiceError> {
        let user = sqlx::query_as!(
            UserCredentials,
            r#"SELECT id, username, password_hash, role, status FROM users WHERE username = $1"#,
            username
        )
        .fetch_optional(pool)
        .await
        .context("Failed to fetch user credentials")?;

        Ok(user)
    }
}

#[derive(serde::Serialize)]
pub struct PendingUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub date_of_birth: chrono::NaiveDate,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct UserCredentials {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub status: String,
}
