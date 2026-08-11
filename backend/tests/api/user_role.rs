// tests/api/user_role.rs

use crate::helpers::spawn_app;
use serde_json::Value;

#[tokio::test]
async fn get_user_role_with_invalid_token_returns_401() {
    let app = spawn_app().await;
    let client = app.client();

    let response = client
        .get(&format!("{}/user_role", &app.address))
        .bearer_auth("invalid-token")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn get_user_role_without_token_returns_401() {
    let app = spawn_app().await;
    let client = app.client();

    let response = client
        .get(&format!("{}/user_role", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 401);
}
