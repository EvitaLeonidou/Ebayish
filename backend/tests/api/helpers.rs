//! tests/api/helpers.rs

use backend::configuration::{DatabaseSettings, get_configuration};
use backend::startup::{Application, get_connection_pool};
use backend::telemetry::{get_subscriber, init_subscriber};
use chrono;
use once_cell::sync::Lazy;
use secrecy::Secret;
use serde_json::Value;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    #[allow(dead_code)]
    pub port: u16,
}

impl TestApp {
    pub fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }

    pub async fn post_users(&self, json: Value) -> reqwest::Response {
        self.client()
            .post(&format!("{}/users", &self.address))
            .header("Content-Type", "application/json")
            .json(&json)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_pending_users(&self) -> reqwest::Response {
        let admin_token = self.create_admin_and_login().await;
        self.client()
            .get(&format!("{}/admin/users/pending", &self.address))
            .header("Authorization", format!("Bearer {}", admin_token))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn verify_user(&self, user_id: &uuid::Uuid) -> reqwest::Response {
        let admin_token = self.create_admin_and_login().await;
        self.client()
            .put(&format!("{}/admin/users/{}/verify", &self.address, user_id))
            .header("Authorization", format!("Bearer {}", admin_token))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn suspend_user(&self, user_id: &uuid::Uuid) -> reqwest::Response {
        let admin_token = self.create_admin_and_login().await;
        self.client()
            .put(&format!(
                "{}/admin/users/{}/suspend",
                &self.address, user_id
            ))
            .header("Authorization", format!("Bearer {}", admin_token))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_and_verify_user(&self, user_data: Value) -> uuid::Uuid {
        // Create the user
        let response = self.post_users(user_data.clone()).await;
        assert_eq!(response.status().as_u16(), 200);

        // Get the user ID from database by username (which should be unique)
        let username = user_data["username"]
            .as_str()
            .expect("Username should be present");
        let user = sqlx::query!("SELECT id FROM users WHERE username = $1", username)
            .fetch_one(&self.db_pool)
            .await
            .expect("Failed to fetch created user");

        // Verify the user
        let verify_response = self.verify_user(&user.id).await;
        assert_eq!(verify_response.status().as_u16(), 200);

        user.id
    }

    pub async fn create_admin_and_login(&self) -> String {
        // Create an admin user directly in the database
        let admin_id = uuid::Uuid::new_v4();
        let admin_username = format!("admin_{}", admin_id.to_string()[..8].to_string());
        let admin_email = format!("admin_{}@test.com", admin_id.to_string()[..8].to_string());
        let password = "admin_password123";

        // Use domain function to hash password properly
        let password_hash = backend::domain::hash_password(Secret::new(password.to_string()))
            .expect("Failed to hash password");

        // Insert admin user directly into database
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, first_name, last_name, phone, date_of_birth, role, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            admin_id,
            admin_username,
            admin_email,
            password_hash,
            "Admin",
            "User",
            "1234567890",
            chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            "admin",
            "confirmed"
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to create admin user");

        // Login and get token
        let login_body = serde_json::json!({
            "username": admin_username,
            "password": password
        });
        let response = self
            .client()
            .post(&format!("{}/login", &self.address))
            .json(&login_body)
            .send()
            .await
            .expect("Failed to login admin user");

        assert_eq!(response.status().as_u16(), 200, "Admin login failed");

        let json: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse login response");
        json["token"]
            .as_str()
            .expect("No token in response")
            .to_string()
    }

    pub async fn login_user(&self, username: &str, password: &str) -> String {
        let login_body = serde_json::json!({
            "username": username,
            "password": password
        });
        let response = self
            .client()
            .post(&format!("{}/login", &self.address))
            .json(&login_body)
            .send()
            .await
            .expect("Failed to login user");

        assert_eq!(response.status().as_u16(), 200, "User login failed");

        let json: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse login response");
        json["token"]
            .as_str()
            .expect("No token in response")
            .to_string()
    }

    pub async fn create_item_authenticated(
        &self,
        item_data: Value,
        token: &str,
    ) -> reqwest::Response {
        self.client()
            .post(&format!("{}/items", &self.address))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&item_data)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_bid_authenticated(
        &self,
        item_id: &str,
        bid_data: Value,
        token: &str,
    ) -> reqwest::Response {
        self.client()
            .post(&format!("{}/items/{}/bids", &self.address, item_id))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&bid_data)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

//spawn server for tests
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");

        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        // Disable SSL for tests to avoid certificate issues
        if let Some(ref mut ssl) = c.application.ssl {
            ssl.enabled = false;
        }
        c
    };

    configure_database(&configuration.database).await;
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let address = if configuration
        .application
        .ssl
        .as_ref()
        .map_or(false, |ssl| ssl.enabled)
    {
        format!("https://127.0.0.1:{}", application.port())
    } else {
        format!("http://127.0.0.1:{}", application.port())
    };

    let _ = tokio::spawn(application.run_until_stopped());
    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        port: application_port,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to postgres.");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create testing database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to testing database.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate testing database.");

    connection_pool
}
