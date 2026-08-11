//! tests/api/login.rs

use crate::helpers::spawn_app;

#[tokio::test]
async fn login_fails_with_non_existent_user() {
    let app = spawn_app().await;

    let response = app
        .client()
        .post(&format!("{}/login", &app.address))
        .json(&serde_json::json!({
            "username": "random-user",
            "password": "random-password"
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(401, response.status().as_u16());
}
