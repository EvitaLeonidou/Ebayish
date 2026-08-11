use crate::helpers::spawn_app;
use serde_json::{Value, json};

#[tokio::test]
async fn create_category_works() {
    let app = spawn_app().await;

    let body = json!({
        "name": "Test Category"
    });

    let response = app
        .client()
        .post(&format!("{}/categories", &app.address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 201);

    let category: Value = response.json().await.expect("Failed to parse response");
    assert!(category["id"].is_number());
    assert_eq!(category["name"], "Test Category");
}

#[tokio::test]
async fn create_category_with_empty_name_fails() {
    let app = spawn_app().await;

    let body = json!({
        "name": ""
    });

    let response = app
        .client()
        .post(&format!("{}/categories", &app.address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn create_category_with_long_name_fails() {
    let app = spawn_app().await;

    let body = json!({
        "name": "a".repeat(101)
    });

    let response = app
        .client()
        .post(&format!("{}/categories", &app.address))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn get_categories_works() {
    let app = spawn_app().await;

    let response = app
        .client()
        .get(&format!("{}/categories", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let categories: Vec<Value> = response.json().await.expect("Failed to parse response");
    // Should have some default categories from migration
    assert!(categories.len() >= 1);
}

#[tokio::test]
async fn get_category_by_id_works() {
    let app = spawn_app().await;

    // First, get a category ID from the list
    let list_response = app
        .client()
        .get(&format!("{}/categories", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    let categories: Vec<Value> = list_response
        .json()
        .await
        .expect("Failed to parse response");
    let category_id = categories[0]["id"].as_i64().unwrap();

    let response = app
        .client()
        .get(&format!("{}/categories/{}", &app.address, category_id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let category: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(category["id"], category_id);
}

#[tokio::test]
async fn get_category_with_nonexistent_id_fails() {
    let app = spawn_app().await;

    let response = app
        .client()
        .get(&format!("{}/categories/99999", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn update_category_works() {
    let app = spawn_app().await;

    // First create a category
    let create_body = json!({
        "name": "Original Name"
    });

    let create_response = app
        .client()
        .post(&format!("{}/categories", &app.address))
        .header("Content-Type", "application/json")
        .json(&create_body)
        .send()
        .await
        .expect("Failed to execute request.");

    let created_category: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let category_id = created_category["id"].as_i64().unwrap();

    // Update the category
    let update_body = json!({
        "name": "Updated Name"
    });

    let response = app
        .client()
        .put(&format!("{}/categories/{}", &app.address, category_id))
        .header("Content-Type", "application/json")
        .json(&update_body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let updated_category: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(updated_category["name"], "Updated Name");
    assert_eq!(updated_category["id"], category_id);
}

#[tokio::test]
async fn delete_category_works() {
    let app = spawn_app().await;

    // First create a category
    let create_body = json!({
        "name": "To Be Deleted"
    });

    let create_response = app
        .client()
        .post(&format!("{}/categories", &app.address))
        .header("Content-Type", "application/json")
        .json(&create_body)
        .send()
        .await
        .expect("Failed to execute request.");

    let created_category: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let category_id = created_category["id"].as_i64().unwrap();

    // Delete the category
    let response = app
        .client()
        .delete(&format!("{}/categories/{}", &app.address, category_id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 204);

    // Verify it's deleted
    let get_response = app
        .client()
        .get(&format!("{}/categories/{}", &app.address, category_id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(get_response.status().as_u16(), 404);
}
