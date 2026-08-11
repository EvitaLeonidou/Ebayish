//! tests/api/healthcheck.rs

use crate::helpers::spawn_app;

//run healthcheck test
#[tokio::test]
async fn healthcheck_works() {
    let app = spawn_app().await;
    let client = app.client();

    let response = client
        .get(&format!("{}/healthcheck", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
