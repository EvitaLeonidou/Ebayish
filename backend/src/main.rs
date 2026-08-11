// src/main.rs

use backend::configuration::get_configuration;
use backend::domain::hash_password;
use backend::services::{AuctionService, ImageService, SeedingService};
use backend::startup::Application;
use backend::telemetry::{get_subscriber, init_subscriber};
use secrecy::Secret;
use sqlx::{Error, PgPool};
use uuid::Uuid;

async fn create_admin_if_not_exists(pool: &PgPool) {
    let admin_user = {
        let mut last_result = Err(Error::PoolTimedOut);
        for i in 0..5 {
            // Retry up to 5 times
            match sqlx::query!("SELECT id FROM users WHERE username = 'admin'")
                .fetch_optional(pool)
                .await
            {
                Ok(user_option) => {
                    last_result = Ok(user_option);
                    break;
                }
                Err(e) => {
                    eprintln!("Attempt {} to query database failed: {:?}", i + 1, e);
                    last_result = Err(e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
        last_result.expect("Failed to query database after multiple retries")
    };

    if admin_user.is_none() {
        let password_hash =
            hash_password(Secret::new("admin".to_string())).expect("Failed to hash password");
        let user_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, first_name, last_name, phone, date_of_birth, status, role, seller_rating)
            VALUES ($1, 'admin', 'admin@example.com', $2, 'Admin', 'User', '1234567890', '2000-01-01', 'confirmed', 'admin', NULL)
            "#,
            user_id,
            password_hash
        )
        .execute(pool)
        .await
        .expect("Failed to create admin user");
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("backend".into(), "trace".into(), std::io::stdout);
    init_subscriber(subscriber);
    //panic if config not read
    let configuration = get_configuration().expect("Failed to read config file.");

    let connection_pool = backend::startup::get_connection_pool(&configuration.database);

    println!("Running database migrations...");
    let mut last_error = None;
    for i in 1..=5 {
        match sqlx::migrate!("./migrations").run(&connection_pool).await {
            Ok(_) => {
                println!("Database migrations completed successfully.");
                last_error = None;
                break;
            }
            Err(e) => {
                eprintln!("Attempt {i} failed: {e}. Retrying in 2 seconds...");
                last_error = Some(e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    }
    if let Some(e) = last_error {
        panic!("Failed to run database migrations: {e}");
    }

    //seed database
    if configuration.seeding.should_run_seeding() {
        tracing::info!("Starting database seeding process...");

        //seed item images
        let image_service = ImageService::from_settings(&configuration);

        let mut seeding_attempts = 0;
        let max_attempts = configuration.seeding.retry_attempts;
        let retry_delay =
            tokio::time::Duration::from_secs(configuration.seeding.retry_delay_seconds);

        while seeding_attempts < max_attempts {
            seeding_attempts += 1;

            match SeedingService::seed_database(
                &connection_pool,
                &configuration.seeding,
                &image_service,
            )
            .await
            {
                Ok(()) => {
                    tracing::info!("Database seeding completed successfully");
                    break;
                }
                Err(backend::services::seeding_service::SeedingError::AlreadyCompleted) => {
                    tracing::info!("Database seeding already completed, skipping");
                    break;
                }
                Err(e) => {
                    if seeding_attempts >= max_attempts {
                        tracing::error!(
                            "Database seeding failed after {} attempts: {:?}",
                            max_attempts,
                            e
                        );
                        panic!("Failed to seed database: {}", e);
                    } else {
                        tracing::warn!(
                            "Database seeding attempt {} failed: {:?}. Retrying in {:?}...",
                            seeding_attempts,
                            e,
                            retry_delay
                        );
                        tokio::time::sleep(retry_delay).await;
                    }
                }
            }
        }
    } else {
        tracing::info!("Database seeding is disabled or not configured for current environment");
    }

    create_admin_if_not_exists(&connection_pool).await;

    let auction_pool = connection_pool.clone();
    tokio::spawn(async move {
        AuctionService::start_auction_monitor(auction_pool).await;
    });

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
