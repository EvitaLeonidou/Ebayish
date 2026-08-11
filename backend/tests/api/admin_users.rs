//! tests/api/admin_users.rs

use crate::helpers::spawn_app;
use serde_json::Value;
use uuid::Uuid;

#[tokio::test]
async fn get_pending_users_returns_empty_list_when_no_pending_users() {
    let app = spawn_app().await;

    let response = app.get_pending_users().await;

    assert_eq!(response.status().as_u16(), 200);
    let users: Vec<Value> = response.json().await.expect("Failed to parse JSON");
    assert_eq!(users.len(), 0);
}

#[tokio::test]
async fn get_pending_users_returns_pending_users() {
    let app = spawn_app().await;

    // Create a user (should be in pending status)
    let user_data = serde_json::json!({
        "username": "pendinguser",
        "email": "pending@example.com",
        "password_hash": "password123",
        "first_name": "Pending",
        "last_name": "User",
        "phone": "1234567890",
        "date_of_birth": "2000-01-01"
    });

    let create_response = app.post_users(user_data).await;
    assert_eq!(create_response.status().as_u16(), 200);

    // Get pending users
    let response = app.get_pending_users().await;
    assert_eq!(response.status().as_u16(), 200);

    let users: Vec<Value> = response.json().await.expect("Failed to parse JSON");
    assert_eq!(users.len(), 1);
    assert_eq!(users[0]["username"], "pendinguser");
    assert_eq!(users[0]["email"], "pending@example.com");
}

#[tokio::test]
async fn verify_user_changes_status_from_pending_to_confirmed() {
    let app = spawn_app().await;

    // Create a user
    let user_data = serde_json::json!({
        "username": "testuser",
        "email": "test@example.com",
        "password_hash": "password123",
        "first_name": "Test",
        "last_name": "User",
        "phone": "1234567890",
        "date_of_birth": "2000-01-01"
    });

    let create_response = app.post_users(user_data).await;
    assert_eq!(create_response.status().as_u16(), 200);

    // Get the user ID
    let user = sqlx::query!("SELECT id FROM users WHERE username = 'testuser'")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch user");

    // Verify user status is pending
    let user_status = sqlx::query!("SELECT status FROM users WHERE id = $1", user.id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch user status");
    assert_eq!(user_status.status, "pending");

    // Verify the user via admin endpoint
    let verify_response = app.verify_user(&user.id).await;
    assert_eq!(verify_response.status().as_u16(), 200);

    // Check that status is now confirmed
    let user_status = sqlx::query!("SELECT status FROM users WHERE id = $1", user.id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch user status");
    assert_eq!(user_status.status, "confirmed");
}

#[tokio::test]
async fn verify_user_removes_user_from_pending_list() {
    let app = spawn_app().await;

    let user_data = serde_json::json!({
        "username": "testuser",
        "email": "test@example.com",
        "password_hash": "password123",
        "first_name": "Test",
        "last_name": "User",
        "phone": "1234567890",
        "date_of_birth": "2000-01-01"
    });

    let create_response = app.post_users(user_data).await;
    assert_eq!(create_response.status().as_u16(), 200);

    let pending_response = app.get_pending_users().await;
    let pending_users: Vec<Value> = pending_response.json().await.expect("Failed to parse JSON");
    assert_eq!(pending_users.len(), 1);

    let user = sqlx::query!("SELECT id FROM users WHERE username = 'testuser'")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch user");

    let verify_response = app.verify_user(&user.id).await;
    assert_eq!(verify_response.status().as_u16(), 200);

    let pending_response = app.get_pending_users().await;
    let pending_users: Vec<Value> = pending_response.json().await.expect("Failed to parse JSON");
    assert_eq!(pending_users.len(), 0);
}

#[tokio::test]
async fn verify_nonexistent_user_returns_404() {
    let app = spawn_app().await;

    let fake_user_id = Uuid::new_v4();
    let verify_response = app.verify_user(&fake_user_id).await;

    assert_eq!(verify_response.status().as_u16(), 404);
}

#[tokio::test]
async fn verify_already_confirmed_user_returns_404() {
    let app = spawn_app().await;

    let user_data = serde_json::json!({
        "username": "testuser",
        "email": "test@example.com",
        "password_hash": "password123",
        "first_name": "Test",
        "last_name": "User",
        "phone": "1234567890",
        "date_of_birth": "2000-01-01"
    });

    let user_id = app.create_and_verify_user(user_data).await;

    let verify_response = app.verify_user(&user_id).await;
    assert_eq!(verify_response.status().as_u16(), 404);
}

#[tokio::test]
async fn suspend_confirmed_user_changes_status_to_suspended() {
    let app = spawn_app().await;

    let user_data = serde_json::json!({
        "username": "testuser",
        "email": "test@example.com",
        "password_hash": "password123",
        "first_name": "Test",
        "last_name": "User",
        "phone": "1234567890",
        "date_of_birth": "2000-01-01"
    });

    let user_id = app.create_and_verify_user(user_data).await;

    let user_status = sqlx::query!("SELECT status FROM users WHERE id = $1", user_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch user status");
    assert_eq!(user_status.status, "confirmed");

    let suspend_response = app.suspend_user(&user_id).await;
    assert_eq!(suspend_response.status().as_u16(), 200);

    let user_status = sqlx::query!("SELECT status FROM users WHERE id = $1", user_id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch user status");
    assert_eq!(user_status.status, "suspended");
}

#[tokio::test]
async fn suspend_pending_user_changes_status_to_suspended() {
    let app = spawn_app().await;

    let user_data = serde_json::json!({
        "username": "pendinguser",
        "email": "pending@example.com",
        "password_hash": "password123",
        "first_name": "Pending",
        "last_name": "User",
        "phone": "1234567890",
        "date_of_birth": "2000-01-01"
    });

    let create_response = app.post_users(user_data).await;
    assert_eq!(create_response.status().as_u16(), 200);

    let user = sqlx::query!("SELECT id FROM users WHERE username = 'pendinguser'")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch user");

    let user_status = sqlx::query!("SELECT status FROM users WHERE id = $1", user.id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch user status");
    assert_eq!(user_status.status, "pending");

    let suspend_response = app.suspend_user(&user.id).await;
    assert_eq!(suspend_response.status().as_u16(), 200);

    let user_status = sqlx::query!("SELECT status FROM users WHERE id = $1", user.id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch user status");
    assert_eq!(user_status.status, "suspended");
}

#[tokio::test]
async fn suspend_already_suspended_user_returns_404() {
    let app = spawn_app().await;

    let user_data = serde_json::json!({
        "username": "testuser",
        "email": "test@example.com",
        "password_hash": "password123",
        "first_name": "Test",
        "last_name": "User",
        "phone": "1234567890",
        "date_of_birth": "2000-01-01"
    });

    let user_id = app.create_and_verify_user(user_data).await;

    let first_suspend_response = app.suspend_user(&user_id).await;
    assert_eq!(first_suspend_response.status().as_u16(), 200);

    let second_suspend_response = app.suspend_user(&user_id).await;
    assert_eq!(second_suspend_response.status().as_u16(), 404);
}

#[tokio::test]
async fn suspend_nonexistent_user_returns_404() {
    let app = spawn_app().await;

    let fake_user_id = Uuid::new_v4();
    let suspend_response = app.suspend_user(&fake_user_id).await;

    assert_eq!(suspend_response.status().as_u16(), 404);
}
