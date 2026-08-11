use crate::helpers::spawn_app;

#[tokio::test]
async fn item_websocket_stats_endpoint_requires_auth() {
    let app = spawn_app().await;

    let sample_uuid = "550e8400-e29b-41d4-a716-446655440000";

    //test without authentication - should return 401
    let response = app
        .client()
        .get(&format!(
            "{}/items/{}/websockets/stats",
            &app.address, sample_uuid
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 401);

}


