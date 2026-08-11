// src/jwt_middleware.rs
pub use crate::routes::login::Claims;
use actix_web::{Error, FromRequest, HttpRequest, dev::Payload};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::future::{Ready, ready};

pub struct AuthenticatedUser {
    pub user_id: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        match token {
            Some(token) => {
                let secret = b"opekepescam";
                let validation = Validation::default();
                match decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation) {
                    Ok(token_data) => ready(Ok(AuthenticatedUser {
                        user_id: token_data.claims.sub,
                    })),
                    Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
                }
            }
            None => ready(Err(actix_web::error::ErrorUnauthorized("Missing token"))),
        }
    }
}

impl FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "));

        match token {
            Some(token) => {
                let secret = b"opekepescam";
                let validation = Validation::default();
                match decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation) {
                    Ok(token_data) => ready(Ok(token_data.claims)),
                    Err(_) => ready(Err(actix_web::error::ErrorUnauthorized("Invalid token"))),
                }
            }
            None => ready(Err(actix_web::error::ErrorUnauthorized("Missing token"))),
        }
    }
}
