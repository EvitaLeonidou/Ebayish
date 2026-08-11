// tests/api/auth.rs

use crate::helpers::spawn_app;
use serde_json::Value;

#[tokio::test]
async fn login_with_correct_credentials_returns_jwt() {
    // Arrange
    let app = spawn_app().await;
    let client = app.client();

    // Create and verify a user
    let create_user_body = serde_json::json!({
        "username": "testuser",
        "email": "test@example.com",
        "password_hash": "password123",
        "first_name": "Test",
        "last_name": "User",
        "phone": "1234567890",
        "date_of_birth": "2000-01-01"
    });
    app.create_and_verify_user(create_user_body).await;

    // Act
    let login_body = serde_json::json!({
        "username": "testuser",
        "password": "password123"
    });
    let response = client
        .post(&format!("{}/login", &app.address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let json: Value = response.json().await.expect("Failed to parse json body");
    assert!(json["token"].is_string());
}

#[tokio::test]
async fn login_with_incorrect_credentials_returns_401() {
    // Arrange
    let app = spawn_app().await;
    let client = app.client();

    // Create and verify a user
    let create_user_body = serde_json::json!({
        "username": "testuser",
        "email": "test@example.com",
        "password_hash": "password123",
        "first_name": "Test",
        "last_name": "User",
        "phone": "1234567890",
        "date_of_birth": "2000-01-01"
    });
    app.create_and_verify_user(create_user_body).await;

    // Act
    let login_body = serde_json::json!({
        "username": "testuser",
        "password": "wrongpassword"
    });
    let response = client
        .post(&format!("{}/login", &app.address))
        .json(&login_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}
