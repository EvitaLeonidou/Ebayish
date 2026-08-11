//! backend/src/routes/admin.rs

//for linting issues
#![allow(clippy::wildcard_in_or_patterns)]
#![allow(clippy::useless_format)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::collapsible_if)]

use actix_web::{HttpResponse, web};
use anyhow::Context;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::define_route_error;
use crate::jwt_middleware::Claims;

define_route_error! {
    DashboardError {
        DataFetchFailed => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch dashboard data"),
        AuthenticationError => (StatusCode::FORBIDDEN, "Admin access required"),
    }
}

#[derive(Serialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub pending_users: i64,
    pub active_auctions: i64,
    pub active_fixed_price: i64,
    pub total_revenue: bigdecimal::BigDecimal,
    pub items_sold: i64,
}

#[derive(Serialize)]
pub struct ActivityItem {
    id: Uuid,
    activity_type: String,
    message: String,
    timestamp: DateTime<Utc>,
    user_id: Option<Uuid>,
    target_id: Option<String>,
}

#[derive(Serialize)]
pub struct RecentActivityResponse {
    activities: Vec<ActivityItem>,
}

#[tracing::instrument(name = "Get admin dashboard stats", skip(pool))]
pub async fn get_dashboard_stats(
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, DashboardError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(DashboardError::AuthenticationError);
    }
    //dashboard info queries
    let (total_users_res, pending_users_res, active_auctions_res, active_fixed_price_res, total_revenue_res, items_sold_res) =
        tokio::try_join!(
            sqlx::query_scalar!("SELECT COUNT(*) FROM users").fetch_one(pool.get_ref()),
            sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE status = 'pending'")
                .fetch_one(pool.get_ref()),
            sqlx::query_scalar!("SELECT COUNT(*) FROM items WHERE listing_type = 'auction' AND status = 'active' AND ends > NOW()")
                .fetch_one(pool.get_ref()),
            sqlx::query_scalar!("SELECT COUNT(*) FROM items WHERE listing_type = 'fixed_price' AND status = 'active'")
                .fetch_one(pool.get_ref()),
            sqlx::query_scalar!(
                "SELECT COALESCE(
                    (SELECT SUM(winning_amount) FROM auction_results) +
                    (SELECT SUM(purchase_price) FROM purchases),
                    0
                ) as total"
            )
            .fetch_one(pool.get_ref()),
            sqlx::query_scalar!(
                "SELECT COALESCE(
                    (SELECT COUNT(*) FROM auction_results) +
                    (SELECT COUNT(*) FROM purchases),
                    0
                ) as total_sold"
            )
            .fetch_one(pool.get_ref())
        )
        .context("Failed to execute one or more dashboard queries")?;

    let stats = DashboardStats {
        total_users: total_users_res.unwrap_or(0),
        pending_users: pending_users_res.unwrap_or(0),
        active_auctions: active_auctions_res.unwrap_or(0),
        active_fixed_price: active_fixed_price_res.unwrap_or(0),
        total_revenue: total_revenue_res.unwrap_or_default(),
        items_sold: items_sold_res.unwrap_or(0),
    };

    Ok(HttpResponse::Ok().json(stats))
}

#[tracing::instrument(name = "Get recent admin activity", skip(pool))]
pub async fn get_recent_activity(
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, DashboardError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(DashboardError::AuthenticationError);
    }
    let rows = sqlx::query(
        r#"
        (
            SELECT
                'user_registration' AS activity_type,
                u.created_at AS timestamp,
                CONCAT('New user ''', u.username, ''' has registered.') AS message,
                u.id AS user_id,
                NULL AS target_id
            FROM users u
        )
        UNION ALL
        (
            SELECT
                'new_listing' AS activity_type,
                i.created_at AS timestamp,
                CONCAT('Item ''', i.name, ''' was listed by ''', u.username, '''.') AS message,
                i.seller_user_id AS user_id,
                i.item_id AS target_id
            FROM items i
            JOIN users u ON i.seller_user_id = u.id
        )
        UNION ALL
        (
            SELECT
                'new_bid' AS activity_type,
                b.time AS timestamp,
                CONCAT(u.username, ' placed a bid of $', b.amount, ' on ''', i.name, '''.') AS message,
                b.bidder_user_id AS user_id,
                b.item_id AS target_id
            FROM bids b
            JOIN users u ON b.bidder_user_id = u.id
            JOIN items i ON b.item_id = i.item_id
        )
        UNION ALL
        (
            SELECT
                'purchase' AS activity_type,
                p.purchased_at AS timestamp,
                CONCAT(u.username, ' purchased ''', i.name, ''' for $', p.purchase_price, '.') AS message,
                p.buyer_user_id AS user_id,
                p.item_id AS target_id
            FROM purchases p
            JOIN users u ON p.buyer_user_id = u.id
            JOIN items i ON p.item_id = i.item_id
        )
        UNION ALL
        (
            SELECT
                'auction_win' AS activity_type,
                ar.ended_at AS timestamp,
                CONCAT(u.username, ' won auction for ''', i.name, ''' with bid of $', ar.winning_amount, '.') AS message,
                ar.winner_user_id AS user_id,
                ar.item_id AS target_id
            FROM auction_results ar
            JOIN users u ON ar.winner_user_id = u.id
            JOIN items i ON ar.item_id = i.item_id
        )
        ORDER BY timestamp DESC
        LIMIT 10;
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch recent activity")?;

    let activities: Result<Vec<ActivityItem>, sqlx::Error> = rows
        .into_iter()
        .map(|row| -> Result<ActivityItem, sqlx::Error> {
            let timestamp: DateTime<Utc> = row.try_get("timestamp")?;
            let user_id_str: Option<String> = row.try_get("user_id").ok();
            let user_id = user_id_str.and_then(|s| Uuid::parse_str(&s).ok());

            Ok(ActivityItem {
                id: Uuid::new_v4(),
                activity_type: row.try_get("activity_type")?,
                message: row.try_get("message")?,
                timestamp,
                user_id,
                target_id: row.try_get("target_id").ok(),
            })
        })
        .collect();

    let activities = activities.context("Failed to parse activity data")?;

    Ok(HttpResponse::Ok().json(RecentActivityResponse { activities }))
}

#[derive(Serialize)]
pub struct SaleDetails {
    pub id: Uuid,
    pub item_id: String,
    pub item_name: String,
    pub buyer_username: String,
    pub seller_username: String,
    pub sale_amount: bigdecimal::BigDecimal,
    pub sale_date: DateTime<Utc>,
    pub sale_type: String,
}

#[derive(Serialize)]
pub struct SalesResponse {
    pub sales: Vec<SaleDetails>,
    pub total_count: i64,
}

#[tracing::instrument(name = "Get all purchases", skip(pool))]
pub async fn get_all_purchases(
    claims: Claims,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, DashboardError> {
    // Only allow admin access
    if claims.role != "admin" {
        return Err(DashboardError::AuthenticationError);
    }

    let sales = sqlx::query!(
        r#"
        (
            SELECT
                p.id,
                p.item_id,
                i.name as item_name,
                buyer.username as buyer_username,
                seller.username as seller_username,
                p.purchase_price as sale_amount,
                p.purchased_at as sale_date,
                'purchase' as sale_type
            FROM purchases p
            JOIN items i ON p.item_id = i.item_id
            JOIN users buyer ON p.buyer_user_id = buyer.id
            JOIN users seller ON p.seller_user_id = seller.id
        )
        UNION ALL
        (
            SELECT
                gen_random_uuid() as id,
                ar.item_id,
                i.name as item_name,
                buyer.username as buyer_username,
                seller.username as seller_username,
                ar.winning_amount as sale_amount,
                ar.ended_at as sale_date,
                'auction' as sale_type
            FROM auction_results ar
            JOIN items i ON ar.item_id = i.item_id
            JOIN users buyer ON ar.winner_user_id = buyer.id
            JOIN users seller ON i.seller_user_id = seller.id
        )
        ORDER BY sale_date DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch purchases")?;

    let sale_details: Vec<SaleDetails> = sales
        .into_iter()
        .map(|row| SaleDetails {
            id: row.id.unwrap_or_default(),
            item_id: row.item_id.unwrap_or_default(),
            item_name: row.item_name.unwrap_or_default(),
            buyer_username: row.buyer_username.unwrap_or_default(),
            seller_username: row.seller_username.unwrap_or_default(),
            sale_amount: row.sale_amount.unwrap_or_default(),
            sale_date: row
                .sale_date
                .map(|d| d.and_utc())
                .unwrap_or_else(chrono::Utc::now),
            sale_type: row.sale_type.unwrap_or_default(),
        })
        .collect();

    let total_count = sale_details.len() as i64;

    Ok(HttpResponse::Ok().json(SalesResponse {
        sales: sale_details,
        total_count,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BidData {
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "TaxID")]
    pub tax_id: Option<String>,
    #[serde(rename = "City")]
    pub city: Option<String>,
    #[serde(rename = "Region")]
    pub region: Option<String>,
    #[serde(rename = "Country")]
    pub country: Option<String>,
    #[serde(rename = "Time")]
    pub time: DateTime<Utc>,
    #[serde(rename = "Amount")]
    pub amount: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct BuyerData {
    #[serde(rename = "BuyerUsername")]
    pub buyer_username: String,
    #[serde(rename = "BuyerTaxID")]
    pub buyer_tax_id: Option<String>,
    #[serde(rename = "BuyerCity")]
    pub buyer_city: Option<String>,
    #[serde(rename = "BuyerCountry")]
    pub buyer_country: Option<String>,
    #[serde(rename = "PurchasePrice")]
    pub purchase_price: BigDecimal,
    #[serde(rename = "PurchasedAt")]
    pub purchased_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ItemData {
    #[serde(rename = "ItemID")]
    pub item_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Category")]
    pub category: String,
    #[serde(rename = "ListingType")]
    pub listing_type: String,
    #[serde(rename = "Price")]
    pub price: BigDecimal,
    #[serde(rename = "CurrentPrice")]
    pub current_price: BigDecimal,
    #[serde(rename = "Location")]
    pub location: Option<String>,
    #[serde(rename = "StartTime")]
    pub start_time: DateTime<Utc>,
    #[serde(rename = "EndTime")]
    pub end_time: DateTime<Utc>,
    #[serde(rename = "SellerRating")]
    pub seller_rating: Option<BigDecimal>,
    #[serde(rename = "SellerUsername")]
    pub seller_username: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Bids", skip_serializing_if = "Vec::is_empty")]
    pub bids: Vec<BidData>,
    #[serde(rename = "Buyer", skip_serializing_if = "Option::is_none")]
    pub buyer: Option<BuyerData>,
}

#[derive(Debug, Serialize)]
pub struct ExportData {
    #[serde(rename = "Items")]
    pub items: Vec<ItemData>,
    #[serde(rename = "ExportedAt")]
    pub exported_at: DateTime<Utc>,
    #[serde(rename = "TotalItems")]
    pub total_items: usize,
}

#[tracing::instrument(name = "Export all listings data", skip(pool, claims))]
pub async fn export_listings(
    query: web::Query<ExportQuery>,
    pool: web::Data<PgPool>,
    claims: Claims,
) -> Result<HttpResponse, DashboardError> {
    // Check admin authorization
    if claims.role != "admin" {
        return Err(DashboardError::AuthenticationError);
    }

    let items_with_bids = sqlx::query!(
        r#"
        SELECT DISTINCT ON (i.item_id, COALESCE(b.id, p.id, gen_random_uuid()))
            i.item_id,
            i.name,
            i.listing_type,
            i.price,
            i.currently as current_price,
            i.location,
            i.started as start_time,
            i.ends as end_time,
            i.seller_rating,
            seller_users.username as "seller_username!",
            i.description,
            COALESCE(c.name, 'Uncategorized') as "category_name!",
            bidder_users.username as "bidder_username?",
            bidder_users.tax_id as "bidder_tax_id?",
            bidder_users.location as "bidder_location?",
            bidder_users.country as "bidder_country?",
            b.time as "bid_time?",
            b.amount as "bid_amount?",
            b.bidder_location as "bid_city?",
            b.bidder_country as "bid_country?",
            buyer_users.username as "buyer_username?",
            buyer_users.tax_id as "buyer_tax_id?",
            buyer_users.location as "buyer_user_location?",
            buyer_users.country as "buyer_user_country?",
            p.purchase_price as "purchase_price?",
            p.purchased_at as "purchased_at?"
        FROM items i
        LEFT JOIN users seller_users ON i.seller_user_id = seller_users.id
        LEFT JOIN item_categories ic ON i.item_id = ic.item_id
        LEFT JOIN categories c ON ic.category_id = c.id
        LEFT JOIN bids b ON i.item_id = b.item_id AND i.listing_type = 'auction'
        LEFT JOIN users bidder_users ON b.bidder_user_id = bidder_users.id
        LEFT JOIN purchases p ON i.item_id = p.item_id AND i.listing_type = 'fixed_price'
        LEFT JOIN users buyer_users ON p.buyer_user_id = buyer_users.id
        ORDER BY i.item_id, COALESCE(b.id, p.id, gen_random_uuid()), COALESCE(b.time, p.purchased_at, i.started) ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch items and bids data")
    .map_err(|e| {
        tracing::error!("Database query failed: {:?}", e);
        DashboardError::DataFetchFailed
    })?;

    let mut items_map: std::collections::HashMap<String, ItemData> =
        std::collections::HashMap::new();

    for row in items_with_bids {
        let item_id = row.item_id.clone();

        let item = items_map
            .entry(item_id.clone())
            .or_insert_with(|| ItemData {
                item_id: item_id.clone(),
                name: row.name.clone(),
                category: row.category_name.clone(),
                listing_type: row.listing_type.clone(),
                price: row.price,
                current_price: row.current_price.unwrap_or_else(|| BigDecimal::from(0)),
                location: row.location.clone(),
                start_time: row.start_time.and_utc(),
                end_time: row.end_time.map(|t| t.and_utc()).unwrap_or_else(Utc::now),
                seller_rating: row.seller_rating,
                seller_username: row.seller_username.clone(),
                description: row.description.clone(),
                bids: Vec::new(),
                buyer: None,
            });

        if let (Some(bidder_username), Some(bid_time), Some(bid_amount)) =
            (row.bidder_username, row.bid_time, row.bid_amount)
        {
            item.bids.push(BidData {
                username: bidder_username,
                tax_id: row.bidder_tax_id,
                city: row.bidder_location.or(row.bid_city),
                region: None,
                country: row.bidder_country.or(row.bid_country),
                time: bid_time.and_utc(),
                amount: bid_amount,
            });
        }

        if let (Some(buyer_username), Some(purchase_price), Some(purchased_at)) =
            (row.buyer_username, row.purchase_price, row.purchased_at)
        {
            item.buyer = Some(BuyerData {
                buyer_username,
                buyer_tax_id: row.buyer_tax_id,
                buyer_city: row.buyer_user_location,
                buyer_country: row.buyer_user_country,
                purchase_price,
                purchased_at: purchased_at.and_utc(),
            });
        }
    }

    let items: Vec<ItemData> = items_map.into_values().collect();
    let total_items = items.len();

    let export_data = ExportData {
        items,
        exported_at: Utc::now(),
        total_items,
    };

    let format = query.format.as_deref().unwrap_or("json");

    match format.to_lowercase().as_str() {
        "xml" => {
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            xml.push_str(&format!("<ExportData>\n"));
            xml.push_str(&format!(
                "  <ExportedAt>{}</ExportedAt>\n",
                export_data.exported_at.to_rfc3339()
            ));
            xml.push_str(&format!(
                "  <TotalItems>{}</TotalItems>\n",
                export_data.total_items
            ));
            xml.push_str("  <Items>\n");

            for item in &export_data.items {
                xml.push_str("    <Item>\n");
                xml.push_str(&format!(
                    "      <ItemID>{}</ItemID>\n",
                    escape_xml(&item.item_id)
                ));
                xml.push_str(&format!("      <Name>{}</Name>\n", escape_xml(&item.name)));
                xml.push_str(&format!(
                    "      <Category>{}</Category>\n",
                    escape_xml(&item.category)
                ));
                xml.push_str(&format!(
                    "      <ListingType>{}</ListingType>\n",
                    escape_xml(&item.listing_type)
                ));
                xml.push_str(&format!("      <Price>{}</Price>\n", item.price));
                xml.push_str(&format!(
                    "      <CurrentPrice>{}</CurrentPrice>\n",
                    item.current_price
                ));
                xml.push_str(&format!(
                    "      <Location>{}</Location>\n",
                    escape_xml(&item.location.as_deref().unwrap_or(""))
                ));
                xml.push_str(&format!(
                    "      <StartTime>{}</StartTime>\n",
                    item.start_time.to_rfc3339()
                ));
                xml.push_str(&format!(
                    "      <EndTime>{}</EndTime>\n",
                    item.end_time.to_rfc3339()
                ));
                xml.push_str(&format!(
                    "      <SellerRating>{}</SellerRating>\n",
                    item.seller_rating
                        .as_ref()
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| "0".to_string())
                ));
                xml.push_str(&format!(
                    "      <SellerUsername>{}</SellerUsername>\n",
                    escape_xml(&item.seller_username)
                ));
                xml.push_str(&format!(
                    "      <Description>{}</Description>\n",
                    escape_xml(&item.description.as_deref().unwrap_or(""))
                ));

                if item.listing_type == "auction" {
                    xml.push_str("      <Bids>\n");
                    for bid in &item.bids {
                        xml.push_str("        <Bid>\n");
                        xml.push_str(&format!(
                            "          <Username>{}</Username>\n",
                            escape_xml(&bid.username)
                        ));
                        xml.push_str(&format!(
                            "          <TaxID>{}</TaxID>\n",
                            escape_xml(&bid.tax_id.as_deref().unwrap_or(""))
                        ));
                        xml.push_str(&format!(
                            "          <City>{}</City>\n",
                            escape_xml(&bid.city.as_deref().unwrap_or(""))
                        ));
                        xml.push_str(&format!(
                            "          <Region>{}</Region>\n",
                            escape_xml(&bid.region.as_deref().unwrap_or(""))
                        ));
                        xml.push_str(&format!(
                            "          <Country>{}</Country>\n",
                            escape_xml(&bid.country.as_deref().unwrap_or(""))
                        ));
                        xml.push_str(&format!(
                            "          <Time>{}</Time>\n",
                            bid.time.to_rfc3339()
                        ));
                        xml.push_str(&format!("          <Amount>{}</Amount>\n", bid.amount));
                        xml.push_str("        </Bid>\n");
                    }
                    xml.push_str("      </Bids>\n");
                }

                if item.listing_type == "fixed_price" {
                    if let Some(buyer) = &item.buyer {
                        xml.push_str("      <Buyer>\n");
                        xml.push_str(&format!(
                            "        <BuyerUsername>{}</BuyerUsername>\n",
                            escape_xml(&buyer.buyer_username)
                        ));
                        xml.push_str(&format!(
                            "        <BuyerTaxID>{}</BuyerTaxID>\n",
                            escape_xml(&buyer.buyer_tax_id.as_deref().unwrap_or(""))
                        ));
                        xml.push_str(&format!(
                            "        <BuyerCity>{}</BuyerCity>\n",
                            escape_xml(&buyer.buyer_city.as_deref().unwrap_or(""))
                        ));
                        xml.push_str(&format!(
                            "        <BuyerCountry>{}</BuyerCountry>\n",
                            escape_xml(&buyer.buyer_country.as_deref().unwrap_or(""))
                        ));
                        xml.push_str(&format!(
                            "        <PurchasePrice>{}</PurchasePrice>\n",
                            buyer.purchase_price
                        ));
                        xml.push_str(&format!(
                            "        <PurchasedAt>{}</PurchasedAt>\n",
                            buyer.purchased_at.to_rfc3339()
                        ));
                        xml.push_str("      </Buyer>\n");
                    }
                }

                xml.push_str("    </Item>\n");
            }

            xml.push_str("  </Items>\n");
            xml.push_str("</ExportData>");

            Ok(HttpResponse::Ok()
                .content_type("application/xml")
                .insert_header((
                    "Content-Disposition",
                    "attachment; filename=\"listings_export.xml\"",
                ))
                .body(xml))
        }
        "json" | _ => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .insert_header((
                "Content-Disposition",
                "attachment; filename=\"listings_export.json\"",
            ))
            .json(export_data)),
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
