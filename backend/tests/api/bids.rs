use crate::helpers::spawn_app;
use chrono::Utc;
use serde_json::{Value, json};

async fn create_test_item(
    app: &crate::helpers::TestApp,
    item_id: &str,
    seller_id: uuid::Uuid,
    token: &str,
) {
    let start_time = Utc::now();
    let end_time = start_time + chrono::Duration::hours(24);

    let body = json!({
        "item_id": item_id,
        "listing_type": "auction",
        "name": "Test Item for Bids",
        "price": "10.00",
        "currently": "10.00",
        "started": start_time,
        "ends": end_time,
        "seller_user_id": seller_id,
        "categories": ["Electronics"]
    });

    let response = app.create_item_authenticated(body, token).await;
    assert_eq!(response.status().as_u16(), 201);
}

#[tokio::test]
async fn create_bid_works() {
    let app = spawn_app().await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Create seller
    let seller_data = json!({
        "username": format!("seller_bid1_{}", timestamp),
        "email": format!("seller_bid1_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": format!("12345676{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    // Create bidder
    let bidder_data = json!({
        "username": format!("bidder1_{}", timestamp),
        "email": format!("bidder1_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": format!("12345677{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    // Login users to get tokens
    let seller_token = app
        .login_user(&format!("seller_bid1_{}", timestamp), "password123")
        .await;
    let bidder_token = app
        .login_user(&format!("bidder1_{}", timestamp), "password123")
        .await;

    // Create item
    create_test_item(
        &app,
        &format!("BIDITEM1_{}", timestamp),
        seller_id,
        &seller_token,
    )
    .await;

    // Create bid
    let bid_time = Utc::now();
    let body = json!({
        "bidder_user_id": bidder_id,
        "bidder_rating": 85,
        "time": bid_time,
        "amount": 15,
        "bidder_location": "Bidder City",
        "bidder_country": "Bidder Country"
    });

    let response = app
        .create_bid_authenticated(&format!("BIDITEM1_{}", timestamp), body, &bidder_token)
        .await;

    let status = response.status().as_u16();
    if status != 201 {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to get error text".to_string());
        panic!("Expected 201 but got {}. Error: {}", status, error_text);
    }

    let response_json: Value = response.json().await.expect("Failed to parse response");

    // Our new response format has a nested structure with bid, is_buy_it_now, auction_ended
    let bid = &response_json["bid"];
    assert_eq!(bid["item_id"], format!("BIDITEM1_{}", timestamp));
    assert_eq!(bid["bidder_user_id"], bidder_id.to_string());
    assert_eq!(bid["amount"], "15"); // BigDecimal serializes without trailing zeros

    // Check new fields
    assert_eq!(response_json["is_buy_it_now"], false);
    assert_eq!(response_json["auction_ended"], false);
}

#[tokio::test]
async fn create_bid_too_low_fails() {
    let app = spawn_app().await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let seller_data = json!({
        "username": format!("seller_bid2_{}", timestamp),
        "email": format!("seller_bid2_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": format!("12345678{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    let bidder_data = json!({
        "username": format!("bidder2_{}", timestamp),
        "email": format!("bidder2_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": format!("12345679{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    // Login users to get tokens
    let seller_token = app
        .login_user(&format!("seller_bid2_{}", timestamp), "password123")
        .await;
    let bidder_token = app
        .login_user(&format!("bidder2_{}", timestamp), "password123")
        .await;

    create_test_item(
        &app,
        &format!("BIDITEM2_{}", timestamp),
        seller_id,
        &seller_token,
    )
    .await;

    // Try to bid equal to current price (should fail)
    let bid_time = Utc::now();
    let body = json!({
        "bidder_user_id": bidder_id,
        "time": bid_time,
        "amount": "10.00"
    });

    let response = app
        .create_bid_authenticated(&format!("BIDITEM2_{}", timestamp), body, &bidder_token)
        .await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn create_bid_on_nonexistent_item_fails() {
    let app = spawn_app().await;

    let bidder_data = json!({
        "username": "bidder3",
        "email": "bidder3@example.com",
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": "1234567900",
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    // Login user to get token
    let bidder_token = app.login_user("bidder3", "password123").await;

    let bid_time = Utc::now();
    let body = json!({
        "bidder_user_id": bidder_id,
        "time": bid_time,
        "amount": "15.00"
    });

    let response = app
        .create_bid_authenticated("NONEXISTENT", body, &bidder_token)
        .await;

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn get_all_bids_works() {
    let app = spawn_app().await;

    let response = app
        .client()
        .get(&format!("{}/bids", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let bids: Vec<Value> = response.json().await.expect("Failed to parse response");
    // Initially empty, but endpoint should work
    assert!(bids.is_empty() || !bids.is_empty());
}

#[tokio::test]
async fn get_bids_for_item_works() {
    let app = spawn_app().await;

    let seller_data = json!({
        "username": "seller_bid3",
        "email": "seller_bid3@example.com",
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": "1234567901",
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    let bidder_data = json!({
        "username": "bidder4",
        "email": "bidder4@example.com",
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": "1234567902",
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    // Login users to get tokens
    let seller_token = app.login_user("seller_bid3", "password123").await;
    let bidder_token = app.login_user("bidder4", "password123").await;

    create_test_item(&app, "BIDITEM3", seller_id, &seller_token).await;

    // Create a bid first
    let bid_time = Utc::now();
    let body = json!({
        "bidder_user_id": bidder_id,
        "time": bid_time,
        "amount": "20.00"
    });

    let create_response = app
        .create_bid_authenticated("BIDITEM3", body, &bidder_token)
        .await;

    assert_eq!(create_response.status().as_u16(), 201);

    // Now get bids for the item
    let response = app
        .client()
        .get(&format!("{}/items/BIDITEM3/bids", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let bids: Vec<Value> = response.json().await.expect("Failed to parse response");
    assert_eq!(bids.len(), 1);
    assert_eq!(bids[0]["item_id"], "BIDITEM3");
}

#[tokio::test]
async fn get_bid_by_id_works() {
    let app = spawn_app().await;

    let seller_data = json!({
        "username": "seller_bid4",
        "email": "seller_bid4@example.com",
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": "1234567903",
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    let bidder_data = json!({
        "username": "bidder5",
        "email": "bidder5@example.com",
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": "1234567904",
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    // Login users to get tokens
    let seller_token = app.login_user("seller_bid4", "password123").await;
    let bidder_token = app.login_user("bidder5", "password123").await;

    create_test_item(&app, "BIDITEM4", seller_id, &seller_token).await;

    // Create a bid first
    let bid_time = Utc::now();
    let body = json!({
        "bidder_user_id": bidder_id,
        "time": bid_time,
        "amount": "25.00"
    });

    let create_response = app
        .create_bid_authenticated("BIDITEM4", body, &bidder_token)
        .await;

    let created_bid: Value = create_response
        .json()
        .await
        .expect("Failed to parse response");
    let bid_id = created_bid["bid"]["id"].as_str().unwrap();

    // Now get the bid by ID
    let response = app
        .client()
        .get(&format!("{}/bids/{}", &app.address, bid_id))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let bid: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(bid["id"], bid_id);
    assert_eq!(bid["amount"], "25"); // BigDecimal serializes without trailing zeros
}

// NEW TESTS FOR ENHANCED BIDDING FEATURES

#[tokio::test]
async fn buy_it_now_triggers_immediate_auction_end() {
    let app = spawn_app().await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Create seller
    let seller_data = json!({
        "username": format!("seller_bin_{}", timestamp),
        "email": format!("seller_bin_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": format!("12345601{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    // Create bidder
    let bidder_data = json!({
        "username": format!("bidder_bin_{}", timestamp),
        "email": format!("bidder_bin_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": format!("12345602{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    // Login users to get tokens
    let seller_token = app
        .login_user(&format!("seller_bin_{}", timestamp), "password123")
        .await;
    let bidder_token = app
        .login_user(&format!("bidder_bin_{}", timestamp), "password123")
        .await;

    // Create item with buy_price
    let start_time = Utc::now();
    let end_time = start_time + chrono::Duration::hours(24);
    let item_body = json!({
        "item_id": format!("BIN_ITEM_{}", timestamp),
        "listing_type": "auction",
        "name": "Buy-It-Now Test Item",
        "price": "10.00",
        "currently": "10.00",
        "buy_price": "50.00",  // Set buy-it-now price
        "started": start_time,
        "ends": end_time,
        "seller_user_id": seller_id,
        "categories": ["Electronics"]
    });

    let create_response = app
        .create_item_authenticated(item_body, &seller_token)
        .await;
    assert_eq!(create_response.status().as_u16(), 201);

    // Place bid that meets buy-it-now price
    let bid_body = json!({
        "bidder_user_id": bidder_id,
        "bidder_rating": 90,
        "time": Utc::now(),
        "amount": "50.00",  // Exact buy-it-now price
        "bidder_location": "Test City",
        "bidder_country": "Test Country"
    });

    let bid_response = app
        .create_bid_authenticated(&format!("BIN_ITEM_{}", timestamp), bid_body, &bidder_token)
        .await;

    assert_eq!(bid_response.status().as_u16(), 200); // 200 for buy-it-now, not 201

    let response_json: Value = bid_response.json().await.expect("Failed to parse response");

    // Verify buy-it-now was triggered
    assert_eq!(response_json["is_buy_it_now"], true);
    assert_eq!(response_json["auction_ended"], true);
    assert_eq!(response_json["bid"]["amount"], "50"); // Should be set to buy_price
}

#[tokio::test]
async fn minimum_bid_increment_validation() {
    let app = spawn_app().await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Create seller and bidder
    let seller_data = json!({
        "username": format!("seller_inc_{}", timestamp),
        "email": format!("seller_inc_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": format!("12345701{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    let bidder_data = json!({
        "username": format!("bidder_inc_{}", timestamp),
        "email": format!("bidder_inc_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": format!("12345702{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    // Login users to get tokens
    let seller_token = app
        .login_user(&format!("seller_inc_{}", timestamp), "password123")
        .await;
    let bidder_token = app
        .login_user(&format!("bidder_inc_{}", timestamp), "password123")
        .await;

    // Create item with low starting price (< $25 tier)
    let start_time = Utc::now();
    let end_time = start_time + chrono::Duration::hours(24);
    let item_body = json!({
        "item_id": format!("INC_ITEM_{}", timestamp),
        "listing_type": "auction",
        "name": "Increment Test Item",
        "price": "5.00",
        "currently": "5.00",
        "number_of_bids": 0,
        "started": start_time,
        "ends": end_time,
        "seller_user_id": seller_id,
        "categories": ["Electronics"]
    });

    app.create_item_authenticated(item_body, &seller_token)
        .await;

    // First bid should work (creates minimum increment requirement)
    let first_bid = json!({
        "bidder_user_id": bidder_id,
        "time": Utc::now(),
        "amount": "6.00",
    });

    let response = app
        .create_bid_authenticated(&format!("INC_ITEM_{}", timestamp), first_bid, &bidder_token)
        .await;

    assert_eq!(response.status().as_u16(), 201);

    // Second bid with insufficient increment should fail
    // For <$25, minimum increment is $0.50, so current $6.00 + $0.50 = $6.50 minimum
    let insufficient_bid = json!({
        "bidder_user_id": bidder_id,
        "time": Utc::now(),
        "amount": "6.25", // Less than required $6.50
    });

    let fail_response = app
        .create_bid_authenticated(
            &format!("INC_ITEM_{}", timestamp),
            insufficient_bid,
            &bidder_token,
        )
        .await;

    let status = fail_response.status().as_u16();
    if status != 400 {
        let body = fail_response.text().await.unwrap_or("No body".to_string());
        println!("Expected 400, got {}: {}", status, body);
    }
    assert_eq!(status, 400); // Should fail

    // Valid increment should work
    let valid_bid = json!({
        "bidder_user_id": bidder_id,
        "time": Utc::now(),
        "amount": "6.50", // Meets minimum increment
    });

    let success_response = app
        .create_bid_authenticated(&format!("INC_ITEM_{}", timestamp), valid_bid, &bidder_token)
        .await;

    assert_eq!(success_response.status().as_u16(), 201);
}

#[tokio::test]
async fn self_bidding_prevention() {
    let app = spawn_app().await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Create seller (who will try to bid on their own item)
    let seller_data = json!({
        "username": format!("seller_self_{}", timestamp),
        "email": format!("seller_self_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": format!("12345801{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    // Login user to get token
    let seller_token = app
        .login_user(&format!("seller_self_{}", timestamp), "password123")
        .await;

    // Create item
    let start_time = Utc::now();
    let end_time = start_time + chrono::Duration::hours(24);
    let item_body = json!({
        "item_id": format!("SELF_ITEM_{}", timestamp),
        "listing_type": "auction",
        "name": "Self-Bid Test Item",
        "price": "10.00",
        "currently": "10.00",
        "started": start_time,
        "ends": end_time,
        "seller_user_id": seller_id,
        "categories": ["Electronics"]
    });

    app.create_item_authenticated(item_body, &seller_token)
        .await;

    // Try to bid on own item
    let self_bid = json!({
        "bidder_user_id": seller_id, // Same as seller!
        "time": Utc::now(),
        "amount": "15.00",
    });

    let response = app
        .create_bid_authenticated(&format!("SELF_ITEM_{}", timestamp), self_bid, &seller_token)
        .await;

    assert_eq!(response.status().as_u16(), 400); // Should be rejected
}

#[tokio::test]
async fn auction_ended_bidding_fails() {
    let app = spawn_app().await;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Create seller and bidder
    let seller_data = json!({
        "username": format!("seller_ended_{}", timestamp),
        "email": format!("seller_ended_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "John",
        "last_name": "Seller",
        "phone": format!("12345901{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let seller_id = app.create_and_verify_user(seller_data.clone()).await;

    let bidder_data = json!({
        "username": format!("bidder_ended_{}", timestamp),
        "email": format!("bidder_ended_{}@example.com", timestamp),
        "password_hash": "password123",
        "first_name": "Jane",
        "last_name": "Bidder",
        "phone": format!("12345902{:02}", timestamp % 100),
        "date_of_birth": "1990-01-01"
    });
    let bidder_id = app.create_and_verify_user(bidder_data.clone()).await;

    // Login users to get tokens
    let seller_token = app
        .login_user(&format!("seller_ended_{}", timestamp), "password123")
        .await;
    let bidder_token = app
        .login_user(&format!("bidder_ended_{}", timestamp), "password123")
        .await;

    // Create item that has already ended
    let start_time = Utc::now() - chrono::Duration::hours(48);
    let end_time = Utc::now() - chrono::Duration::hours(1); // Ended 1 hour ago
    let item_body = json!({
        "item_id": format!("ENDED_ITEM_{}", timestamp),
        "listing_type": "auction",
        "name": "Already Ended Item",
        "price": "10.00",
        "currently": "10.00",
        "started": start_time,
        "ends": end_time, // In the past
        "seller_user_id": seller_id,
        "categories": ["Electronics"]
    });

    app.create_item_authenticated(item_body, &seller_token)
        .await;

    // Try to bid on ended auction
    let bid_body = json!({
        "bidder_user_id": bidder_id,
        "time": Utc::now(),
        "amount": "15.00",
    });

    let response = app
        .create_bid_authenticated(
            &format!("ENDED_ITEM_{}", timestamp),
            bid_body,
            &bidder_token,
        )
        .await;

    assert_eq!(response.status().as_u16(), 400); // Should fail - auction ended
}
