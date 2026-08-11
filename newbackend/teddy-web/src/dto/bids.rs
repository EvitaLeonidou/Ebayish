use bigdecimal::{BigDecimal, FromPrimitive};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct BidRequest {
    pub bidder_user_id: Uuid,
    pub bidder_rating: Option<i32>,
    pub time: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_amount")]
    pub amount: BigDecimal,
    pub bidder_location: Option<String>,
    pub bidder_country: Option<String>,
}

fn deserialize_amount<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;

    match value {
        serde_json::Value::String(s) => {
            tracing::debug!("Deserializing amount from string: {}", s);
            BigDecimal::from_str(&s)
                .map_err(|e| D::Error::custom(format!("Invalid decimal string: {}", e)))
        }
        serde_json::Value::Number(n) => {
            tracing::debug!("Deserializing amount from number: {:?}", n);
            if let Some(i) = n.as_i64() {
                Ok(BigDecimal::from(i))
            } else if let Some(f) = n.as_f64() {
                Ok(BigDecimal::from_f64(f).unwrap_or_else(|| BigDecimal::from(0)))
            } else {
                Err(D::Error::custom("Number too large for BigDecimal"))
            }
        }
        _ => Err(D::Error::custom("Amount must be a number or string")),
    }
}

#[derive(Serialize)]
pub struct Bid {
    pub id: Uuid,
    pub item_id: String,
    pub bidder_user_id: Uuid,
    pub bidder_rating: Option<i32>,
    pub time: DateTime<Utc>,
    pub amount: BigDecimal,
    pub bidder_location: Option<String>,
    pub bidder_country: Option<String>,
}

#[derive(Serialize)]
pub struct CreateBidResponse {
    pub bid: Bid,
    pub is_buy_it_now: bool,
    pub auction_ended: bool,
}

#[derive(Serialize)]
pub struct BidListResponse {
    pub bids: Vec<Bid>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct BidDetailResponse {
    pub bid: Bid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_amount_integer() {
        let json_data = json!({
            "bidder_user_id": "550e8400-e29b-41d4-a716-446655440000",
            "time": "2023-01-01T00:00:00Z",
            "amount": 15
        });

        let bid_request: BidRequest =
            serde_json::from_value(json_data).expect("Failed to deserialize");
        assert_eq!(bid_request.amount, BigDecimal::from(15));
    }

    #[test]
    fn test_deserialize_amount_string() {
        let json_data = json!({
            "bidder_user_id": "550e8400-e29b-41d4-a716-446655440000",
            "time": "2023-01-01T00:00:00Z",
            "amount": "15.50"
        });

        let bid_request: BidRequest =
            serde_json::from_value(json_data).expect("Failed to deserialize");
        assert_eq!(bid_request.amount, BigDecimal::from_str("15.50").unwrap());
    }
}