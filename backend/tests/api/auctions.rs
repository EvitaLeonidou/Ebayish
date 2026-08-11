use crate::helpers::spawn_app;
use chrono::Utc;
use serde_json::{Value, json};

async fn create_test_item_with_end_time(
    app: &crate::helpers::TestApp,
    item_id: &str,
    seller_id: uuid::Uuid,
    end_time: chrono::DateTime<Utc>,
    token: &str,
) {
    let start_time = Utc::now() - chrono::Duration::hours(2);

    let body = json!({
        "item_id": item_id,
        "listing_type": "auction",
        "name": "Test Auction Item",
        "price": "10.00",
        "currently": "10.00",
        "number_of_bids": 0,
        "started": start_time,
        "ends": end_time,
        "seller_user_id": seller_id,
        "categories": ["Electronics"]
    });

    let response = app.create_item_authenticated(body, token).await;
    assert_eq!(response.status().as_u16(), 201);
}

async fn place_test_bid(
    app: &crate::helpers::TestApp,
    item_id: &str,
    bidder_id: uuid::Uuid,
    amount: &str,
    token: &str,
) {
    let bid_body = json!({
        "bidder_user_id": bidder_id,
        "time": Utc::now(),
        "amount": amount,
    });

    let response = app.create_bid_authenticated(item_id, bid_body, token).await;

    let status = response.status().as_u16();
    if status != 201 && status != 200 {
        let body = response.text().await.unwrap_or("No body".to_string());
        panic!("Expected 201 or 200, got {}: {}", status, body);
    }
    println!(
        "Placed bid of {} for item {} with bidder {}",
        amount, item_id, bidder_id
    );
}

#[tokio::test]
async fn get_auction_stats_works() {
    let app = spawn_app().await;

    let response = app
        .client()
        .get(&format!("{}/auctions/stats", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let stats: Value = response.json().await.expect("Failed to parse response");

    assert!(stats["active_auctions"].is_number());
    assert!(stats["ended_today"].is_number());
    assert!(stats["total_bids_today"].is_number());
}

#[tokio::test]
async fn get_auction_results_works() {
    let app = spawn_app().await;

    let response = app
        .client()
        .get(&format!("{}/auctions/results", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let results: Value = response.json().await.expect("Failed to parse response");
    assert!(results.is_array());
}

#[tokio::test]
async fn force_end_auction_works() {
    let app = spawn_app().await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let seller_data = json!({
        "username": format!("seller_force_{}", timestamp),
        "email": format!("seller_force_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": format!("12346001{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    let bidder_data = json!({
        "username": format!("bidder_force_{}", timestamp),
        "email": format!("bidder_force_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": format!("12346002{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    let seller_token = app
        .login_user(&format!("seller_force_{}", timestamp), "password123")
        .await;
    let bidder_token = app
        .login_user(&format!("bidder_force_{}", timestamp), "password123")
        .await;

    let item_id = format!("FORCE_ITEM_{}", timestamp);

    let end_time = Utc::now() + chrono::Duration::hours(24);
    create_test_item_with_end_time(&app, &item_id, seller_id, end_time, &seller_token).await;

    place_test_bid(&app, &item_id, bidder_id, "15.00", &bidder_token).await;

    let admin_token = app.create_admin_and_login().await;
    let response = app
        .client()
        .post(&format!("{}/admin/auctions/{}/end", &app.address, item_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let result: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(result["item_id"], item_id);
    assert_eq!(result["winner_user_id"], bidder_id.to_string());
    assert_eq!(result["winning_amount"], "15");
}

#[tokio::test]
async fn get_specific_auction_result_works() {
    let app = spawn_app().await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let seller_data = json!({
        "username": format!("seller_result_{}", timestamp),
        "email": format!("seller_result_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": format!("12346101{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    let bidder_data = json!({
        "username": format!("bidder_result_{}", timestamp),
        "email": format!("bidder_result_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": format!("12346102{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    let seller_token = app
        .login_user(&format!("seller_result_{}", timestamp), "password123")
        .await;
    let bidder_token = app
        .login_user(&format!("bidder_result_{}", timestamp), "password123")
        .await;

    let item_id = format!("RESULT_ITEM_{}", timestamp);

    let end_time = Utc::now() + chrono::Duration::hours(24);
    create_test_item_with_end_time(&app, &item_id, seller_id, end_time, &seller_token).await;
    place_test_bid(&app, &item_id, bidder_id, "20.00", &bidder_token).await;

    let admin_token = app.create_admin_and_login().await;
    app.client()
        .post(&format!("{}/admin/auctions/{}/end", &app.address, item_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request.");

    let response = app
        .client()
        .get(&format!("{}/auctions/results/{}", &app.address, item_id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let result: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(result["item_id"], item_id);
    assert_eq!(result["winner_user_id"], bidder_id.to_string());
    assert_eq!(result["winning_amount"], "20");
}

#[tokio::test]
async fn get_nonexistent_auction_result_returns_404() {
    let app = spawn_app().await;

    let response = app
        .client()
        .get(&format!(
            "{}/auctions/results/NONEXISTENT_ITEM",
            &app.address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn auction_with_multiple_bids_determines_correct_winner() {
    let app = spawn_app().await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let seller_data = json!({
        "username": format!("seller_multi_{}", timestamp),
        "email": format!("seller_multi_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": format!("12346201{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    let bidder1_data = json!({
        "username": format!("bidder1_multi_{}", timestamp),
        "email": format!("bidder1_multi_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder1",
        "phone": format!("12346202{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let bidder1_id = app.create_and_verify_user(bidder1_data.clone()).await;

    let bidder2_data = json!({
        "username": format!("bidder2_multi_{}", timestamp),
        "email": format!("bidder2_multi_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "Bob",
        "last_name": "Bidder2",
        "phone": format!("12346203{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let bidder2_id = app.create_and_verify_user(bidder2_data.clone()).await;

    let seller_token = app
        .login_user(&format!("seller_multi_{}", timestamp), "password123")
        .await;
    let bidder1_token = app
        .login_user(&format!("bidder1_multi_{}", timestamp), "password123")
        .await;
    let bidder2_token = app
        .login_user(&format!("bidder2_multi_{}", timestamp), "password123")
        .await;

    let item_id = format!("MULTI_ITEM_{}", timestamp);

    let end_time = Utc::now() + chrono::Duration::hours(24);
    create_test_item_with_end_time(&app, &item_id, seller_id, end_time, &seller_token).await;

    place_test_bid(&app, &item_id, bidder1_id, "15.00", &bidder1_token).await;
    place_test_bid(&app, &item_id, bidder2_id, "20.00", &bidder2_token).await; // Higher bid
    place_test_bid(&app, &item_id, bidder1_id, "25.00", &bidder1_token).await; // Even higher bid

    let admin_token = app.create_admin_and_login().await;
    app.client()
        .post(&format!("{}/admin/auctions/{}/end", &app.address, item_id))
        .header("Authorization", format!("Bearer {}", admin_token))
        .send()
        .await
        .expect("Failed to execute request.");

    let response = app
        .client()
        .get(&format!("{}/auctions/results/{}", &app.address, item_id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let result: Value = response.json().await.expect("Failed to parse response");
    println!(
        "Auction result response: {}",
        serde_json::to_string_pretty(&result).unwrap()
    );
    assert_eq!(result["winner_user_id"], bidder1_id.to_string());
    assert_eq!(result["winning_amount"], "25");
    assert_eq!(result["total_bids"], 3);
}
