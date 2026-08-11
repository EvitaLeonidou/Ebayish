// teddy-web/src/handlers/admin/dashboard.rs

use actix_web::{HttpResponse, web};
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::dto::admin::{DashboardStats, ActivityItem, RecentActivityResponse};
use crate::errors::admin::DashboardError;

#[tracing::instrument(name = "Get admin dashboard stats", skip(pool))]
pub async fn get_dashboard_stats(pool: web::Data<PgPool>) -> Result<HttpResponse, DashboardError> {
    // We run all queries concurrently for maximum efficiency
    let (total_users_res, pending_users_res, active_listings_res, total_revenue_res) =
        tokio::try_join!(
            sqlx::query_scalar!("SELECT COUNT(*) FROM users").fetch_one(pool.get_ref()),
            sqlx::query_scalar!("SELECT COUNT(*) FROM users WHERE status = 'pending'")
                .fetch_one(pool.get_ref()),
            sqlx::query_scalar!("SELECT COUNT(*) FROM items WHERE ends > NOW()")
                .fetch_one(pool.get_ref()),
            sqlx::query_scalar!(
                "SELECT COALESCE(SUM(winning_amount), 0) as total FROM auction_results"
            )
            .fetch_one(pool.get_ref())
        )
        .context("Failed to execute one or more dashboard queries")
        .map_err(DashboardError::UnexpectedError)?;

    // Safely unwrap the query results, providing a default of 0 if the tables are empty
    let stats = DashboardStats {
        total_users: total_users_res.unwrap_or(0),
        pending_users: pending_users_res.unwrap_or(0),
        active_listings: active_listings_res.unwrap_or(0),
        total_revenue: total_revenue_res.unwrap_or_default(),
    };

    Ok(HttpResponse::Ok().json(stats))
}

#[tracing::instrument(name = "Get recent admin activity", skip(pool))]
pub async fn get_recent_activity(pool: web::Data<PgPool>) -> Result<HttpResponse, DashboardError> {
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
        ORDER BY timestamp DESC
        LIMIT 10;
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    .context("Failed to fetch recent activity")
    .map_err(DashboardError::UnexpectedError)?;

    // Manually map the mixed rows from the UNION query to our strongly-typed struct
    let activities: Vec<ActivityItem> = rows
        .into_iter()
        .map(|row| {
            let naive_dt: chrono::NaiveDateTime = row.get("timestamp");
            ActivityItem {
                id: Uuid::new_v4(), // Generate a unique ID for the React key
                activity_type: row.get("activity_type"),
                message: row.get("message"),
                timestamp: DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc),
                user_id: row.try_get("user_id").ok(),
                target_id: row.try_get("target_id").ok(),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(RecentActivityResponse { activities }))
}