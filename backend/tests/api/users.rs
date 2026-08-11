//! tests/api/users.rs
use crate::helpers::spawn_app;

//run user test invalid
#[tokio::test]
async fn user_returns_400_missing_data() {
    let app = spawn_app().await;

    let test_cases = vec![
        (
            serde_json::json!(
            {
              "email": "deathwish.tonpaitnei@tsimpouki.com",
              "password_hash": "themosagapw",
              "first_name": "marios",
              "last_name": "deathwish",
              "phone": "+696969696969",
              "date_of_birth": "1821-05-15"
            }),
            "missing username",
        ),
        (
            serde_json::json!(
            {
              "username": "deathwish",
              "password_hash": "themosagapw",
              "first_name": "marios",
              "last_name": "deathwish",
              "phone": "+696969696969",
              "date_of_birth": "1821-05-15"
            }),
            "missing email",
        ),
        (
            serde_json::json!(
            {
              "username": "deathwish",
              "email": "deathwish.tonpaitnei@tsimpouki.com",
              "first_name": "marios",
              "last_name": "deathwish",
              "phone": "+696969696969",
              "date_of_birth": "1821-05-15"
            }),
            "missing password",
        ),
        (
            serde_json::json!(
            {
              "username": "deathwish",
              "email": "deathwish.tonpaitnei@tsimpouki.com",
              "password_hash": "themosagapw",
              "last_name": "deathwish",
              "phone": "+696969696969",
              "date_of_birth": "1821-05-15"
            }),
            "missing first name",
        ),
        (
            serde_json::json!(
             {
               "username": "deathwish",
               "email": "deathwish.tonpaitnei@tsimpouki.com",
               "password_hash": "themosagapw",
               "first_name": "marios",
               "phone": "+696969696969",
               "date_of_birth": "1821-05-15"
            }),
            "last name",
        ),
        (
            serde_json::json!(
            {
              "username": "deathwish",
              "email": "deathwish.tonpaitnei@tsimpouki.com",
              "password_hash": "themosagapw",
              "first_name": "marios",
              "last_name": "deathwish",
              "date_of_birth": "1821-05-15"
            }),
            "missing phone",
        ),
        (
            serde_json::json!(
            {
              "username": "deathwish",
              "email": "deathwish.tonpaitnei@tsimpouki.com",
              "password_hash": "themosagapw",
              "first_name": "marios",
              "last_name": "deathwish",
              "phone": "+696969696969",
            }),
            "missing date of birth",
        ),
        (
            serde_json::json!(
            {
              "first_name": "marios",
              "last_name": "deathwish",
              "phone": "+696969696969",
            }),
            "missing username email password",
        ),
        (serde_json::json!({}), "missing everything"),
    ];

    for (invalid_json, error_message) in test_cases {
        let response = app.post_users(invalid_json).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 when payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn user_returns_400_fields_present_but_empty() {
    let app = spawn_app().await;
    let _client = app.client();
    let test_cases = vec![
        (
            serde_json::json!(
            {
              "username": "",
              "email": "deathwish@tsimpouki.com",
              "password_hash": "themosagapw",
              "first_name": "marios",
              "last_name": "deathwish",
              "phone": "+696969696969",
              "date_of_birth": "1821-05-15"
            }),
            "empty username",
        ),
        (
            serde_json::json!(
            {
              "username": "deathwadish",
              "email": "",
              "password_hash": "themosagapw",
              "first_name": "marios",
              "last_name": "deathwish",
              "phone": "+696969696969",
              "date_of_birth": "1821-05-15"
            }),
            "empty email",
        ),
        (
            serde_json::json!(
            {
              "username": "deathwishidgda",
              "email": "invalid-email-most-def",
              "password_hash": "themosagapw",
              "first_name": "marios",
              "last_name": "deathwish",
              "phone": "+696969696969",
              "date_of_birth": "1821-05-15"
            }),
            "invalid email",
        ),
        (
            serde_json::json!(
            {
              "username": "deathwishidgda",
              "email": "invalid-email-most-def",
              "password_hash": "themosagapw",
              "first_name": "",
              "last_name": "deathwish",
              "phone": "+696969696969",
              "date_of_birth": "1821-05-15"
            }),
            "empty fist_name",
        ),
        (
            serde_json::json!(
            {
              "username": "deathwishidgda",
              "email": "invalid-email-most-def",
              "password_hash": "themosagapw",
              "first_name": "marios",
              "last_name": "",
              "phone": "+696969696969",
              "date_of_birth": "1821-05-15"
            }),
            "empty last_name",
        ),
        //TODO: make tests for empty passwords
        //validate phone number and data of birth
    ];

    for (invalid_json, description) in test_cases {
        let response = app.post_users(invalid_json).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn create_user_fails_if_there_is_a_fatal_database_error() {
    let app = spawn_app().await;
    let test = serde_json::json!(
        {
          "username": "deathwish",
          "email": "deathwish@tsimpouki.com",
          "password_hash": "themosagapw",
          "first_name": "marios",
          "last_name": "deathwish",
          "phone": "+696969696969",
          "date_of_birth": "1821-05-15"
        }
    );

    // Break the users table by dropping a required column
    sqlx::query!("ALTER TABLE users DROP COLUMN email;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app.post_users(test).await;

    assert_eq!(response.status().as_u16(), 500);
}
