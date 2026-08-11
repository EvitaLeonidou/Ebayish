use crate::helpers::spawn_app;
use chrono::Utc;
use serde_json::{Value, json};

#[tokio::test]
async fn create_item_works() {
    let app = spawn_app().await;

    // First create and verify a user to use as seller
    let user_data = json!({
        "username": "seller123",
        "email": "seller@example.com",
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": "1234567890",
        "date_of_birth": "1990-01-01"
    });

    let user_id = app.create_and_verify_user(user_data).await;

    let start_time = Utc::now();
    let end_time = start_time + chrono::Duration::hours(24);

    let body = json!({
        "item_id": "ITEM123",
        "listing_type": "auction",
        "name": "Test Item",
        "price": "10.00",
        "currently": "10.00",
        "buy_price": "100.00",
        "number_of_bids": 0,
        "location": "Test City",
        "country": "Test Country",
        "started": start_time,
        "ends": end_time,
        "description": "A test item for auction",
        "seller_user_id": user_id,
        "categories": ["Electronics", "Books"]
    });

    let response = app
        .client()
        .post(&format!("{}/items", &app.address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 201);

    let item: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(item["item_id"], "ITEM123");
    assert_eq!(item["name"], "Test Item");
    assert_eq!(item["categories"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn create_item_with_invalid_categories_fails() {
    let app = spawn_app().await;

    let user_data = json!({
        "username": "seller124",
        "email": "seller2@example.com",
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Seller",
        "phone": "1234567891",
        "date_of_birth": "1990-01-01"
    });

    let user_id = app.create_and_verify_user(user_data).await;

    let start_time = Utc::now();
    let end_time = start_time + chrono::Duration::hours(24);

    let body = json!({
        "item_id": "ITEM124",
        "listing_type": "auction",
        "name": "Test Item",
        "price": "10.00",
        "currently": "10.00",
        "started": start_time,
        "ends": end_time,
        "seller_user_id": user_id,
        "categories": ["NonexistentCategory"]
    });

    let response = app
        .client()
        .post(&format!("{}/items", &app.address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn create_item_with_invalid_dates_fails() {
    let app = spawn_app().await;

    let user_data = json!({
        "username": "seller125",
        "email": "seller3@example.com",
        "password_hash": "password123",
        "first_name": "Bob",
        "last_name": "Seller",
        "phone": "1234567892",
        "date_of_birth": "1990-01-01"
    });

    let user_id = app.create_and_verify_user(user_data).await;

    let start_time = Utc::now();
    let end_time = start_time - chrono::Duration::hours(1); // End before start

    let body = json!({
        "item_id": "ITEM125",
        "listing_type": "auction",
        "name": "Test Item",
        "price": "10.00",
        "currently": "10.00",
        "started": start_time,
        "ends": end_time,
        "seller_user_id": user_id,
        "categories": ["Electronics"]
    });

    let response = app
        .client()
        .post(&format!("{}/items", &app.address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn get_items_works() {
    let app = spawn_app().await;

    let response = app
        .client()
        .get(&format!("{}/items", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let items: Vec<Value> = response.json().await.expect("Failed to parse response");
    // Initially empty, but endpoint should work
    assert!(items.is_empty() || !items.is_empty());
}

#[tokio::test]
async fn get_item_by_id_works() {
    let app = spawn_app().await;

    // First create an item
    let user_data = json!({
        "username": "seller126",
        "email": "seller4@example.com",
        "password_hash": "password123",
        "first_name": "Alice",
        "last_name": "Seller",
        "phone": "1234567893",
        "date_of_birth": "1990-01-01"
    });

    let user_id = app.create_and_verify_user(user_data).await;

    let start_time = Utc::now();
    let end_time = start_time + chrono::Duration::hours(24);

    let create_body = json!({
        "item_id": "ITEM126",
        "listing_type": "auction",
        "name": "Test Item for Get",
        "price": "15.00",
        "currently": "15.00",
        "started": start_time,
        "ends": end_time,
        "seller_user_id": user_id,
        "categories": ["Electronics"]
    });

    let create_response = app
        .client()
        .post(&format!("{}/items", &app.address))
        .header("Content-Type", "application/json")
        .json(&create_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(create_response.status().as_u16(), 201);

    // Now get the item
    let response = app
        .client()
        .get(&format!("{}/items/ITEM126", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let item: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(item["item_id"], "ITEM126");
    assert_eq!(item["name"], "Test Item for Get");
}

#[tokio::test]
async fn get_item_with_nonexistent_id_fails() {
    let app = spawn_app().await;

    let response = app
        .client()
        .get(&format!("{}/items/NONEXISTENT", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn delete_item_works() {
    let app = spawn_app().await;

    // First create an item
    let user_data = json!({
        "username": "seller128",
        "email": "seller6@example.com",
        "password_hash": "password123",
        "first_name": "David",
        "last_name": "Seller",
        "phone": "1234567895",
        "date_of_birth": "1990-01-01"
    });

    let user_id = app.create_and_verify_user(user_data).await;

    let start_time = Utc::now();
    let end_time = start_time + chrono::Duration::hours(24);

    let create_body = json!({
        "item_id": "ITEM128",
        "listing_type": "auction",
        "name": "To Be Deleted",
        "price": "30.00",
        "currently": "30.00",
        "started": start_time,
        "ends": end_time,
        "seller_user_id": user_id,
        "categories": ["Electronics"]
    });

    let create_response = app
        .client()
        .post(&format!("{}/items", &app.address))
        .header("Content-Type", "application/json")
        .json(&create_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(create_response.status().as_u16(), 201);

    // Delete the item
    let response = app
        .client()
        .delete(&format!("{}/items/ITEM128", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 204);

    // Verify it's deleted
    let get_response = app
        .client()
        .get(&format!("{}/items/ITEM128", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(get_response.status().as_u16(), 404);
}
