use actix_web::{web, Scope};
use crate::handlers::admin::{
    dashboard::{get_dashboard_stats, get_recent_activity},
    users::{activate_user, get_all_users, get_pending_users, suspend_user, verify_user},
};

pub fn configure() -> Scope {
    web::scope("/admin")
        // User Management
        .route("/users", web::get().to(get_all_users))
        .route("/users/pending", web::get().to(get_pending_users))
        .route("/users/{user_id}/verify", web::put().to(verify_user))
        .route("/users/{user_id}/suspend", web::put().to(suspend_user))
        .route("/users/{user_id}/activate", web::put().to(activate_user))
        // Dashboard Routes
        .route("/dashboard/stats", web::get().to(get_dashboard_stats))
        .route("/dashboard/activity", web::get().to(get_recent_activity))
}