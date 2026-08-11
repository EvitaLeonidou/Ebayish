//! src/startup.rs

use crate::configuration::{DatabaseSettings, Settings, SslSettings, UploadSettings};
use crate::routes::{
    activate_user, add_to_cart, change_password, clear_cart, create_bid, create_category,
    create_chat_room, create_item, create_user, delete_all_notifications, delete_category,
    delete_image, delete_item, delete_message, delete_notification, export_listings,
    force_end_auction, get_active_listings, get_all_purchases, get_all_users, get_auction_result,
    get_auction_results, get_auction_stats, get_bid, get_bid_history, get_bids, get_bids_for_item,
    get_cart, get_categories, get_category, get_connections, get_dashboard_stats, get_item,
    get_item_images, get_items, get_messages, get_notification_summary, get_notifications,
    get_pending_users, get_purchased_items, get_recent_activity, get_recommendations,
    get_sold_items, get_user_by_id, get_user_stats, get_user_websocket_stats, healthcheck,
    item_websocket_stats, login, mark_all_as_read, mark_as_read, purchase_item, remove_from_cart,
    retrain_model, send_message, serve_image, subscribe_to_user_events, suspend_user,
    track_category_view, update_category, update_item, upload_images, user_role, verify_user,
    websocket_handler, websocket_stats,
};
use crate::services::{
    CountdownService, ImageService, MessageService, RecommendationService, UserWebSocketService,
    WebSocketService,
};
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::fs::File;
use std::io::BufReader;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            configuration.application.base_url,
            configuration.application.ssl,
            configuration.uploads,
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect_lazy_with(configuration.with_db())
}

pub struct ApplicationBaseUrl(pub String);

pub fn load_ssl_config(
    ssl_settings: &SslSettings,
) -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
    let cert_file = &mut BufReader::new(File::open(&ssl_settings.certificate_path)?);
    let key_file = &mut BufReader::new(File::open(&ssl_settings.private_key_path)?);

    let cert_chain = certs(cert_file)?
        .into_iter()
        .map(rustls::Certificate)
        .collect();
    let mut keys: Vec<rustls::PrivateKey> = pkcs8_private_keys(key_file)?
        .into_iter()
        .map(rustls::PrivateKey)
        .collect();

    if keys.is_empty() {
        return Err("No private keys found".into());
    }

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0))?;

    Ok(config)
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    base_url: String,
    ssl_settings: Option<SslSettings>,
    upload_settings: UploadSettings,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let base_url = Data::new(ApplicationBaseUrl(base_url));
    let websocket_service = Data::new(WebSocketService::new());
    let user_websocket_service = Data::new(UserWebSocketService::new(websocket_service.clone()));
    let image_service = Data::new(ImageService::new(&upload_settings));
    let recommendation_service = Data::new(std::sync::Mutex::new(RecommendationService::new()));
    let message_service = Data::new(MessageService::new(
        db_pool.get_ref().clone(),
        websocket_service.get_ref().clone(),
    ));

    CountdownService::start_countdown_timer(db_pool.clone(), websocket_service.clone());
    tracing::info!("Started countdown timer service for real-time auction updates");

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/healthcheck", web::get().to(healthcheck))
            // User Routes
            .route("/users", web::post().to(create_user))
            .route("/users/connections", web::get().to(get_connections))
            .route("/users/{user_id}", web::get().to(get_user_by_id))
            .route("/users/{user_id}/stats", web::get().to(get_user_stats))
            .route(
                "/users/{user_id}/purchased-items",
                web::get().to(get_purchased_items),
            )
            .route("/users/{user_id}/sold-items", web::get().to(get_sold_items))
            .route(
                "/users/{user_id}/active-listings",
                web::get().to(get_active_listings),
            )
            .route(
                "/users/{user_id}/bid-history",
                web::get().to(get_bid_history),
            )
            .route("/users/{user_id}/password", web::put().to(change_password))
            .route(
                "/users/{user_id}/subscribe",
                web::post().to(subscribe_to_user_events),
            )
            .route("/login", web::post().to(login))
            .route("/user_role", web::get().to(user_role))
            // Notification routes
            .route("/notifications", web::get().to(get_notifications))
            .route(
                "/notifications/summary",
                web::get().to(get_notification_summary),
            )
            .route("/notifications/{id}/read", web::put().to(mark_as_read))
            .route("/notifications/read-all", web::put().to(mark_all_as_read))
            .route("/notifications/{id}", web::delete().to(delete_notification))
            .route("/notifications", web::delete().to(delete_all_notifications))
            // Admin User Management
            .route("/admin/users", web::get().to(get_all_users))
            .route("/admin/users/pending", web::get().to(get_pending_users))
            .route("/admin/users/{user_id}/verify", web::put().to(verify_user))
            .route(
                "/admin/users/{user_id}/suspend",
                web::put().to(suspend_user),
            )
            .route(
                "/admin/users/{user_id}/activate",
                web::put().to(activate_user),
            )
            // Admin Dashboard Routes
            .route("/admin/dashboard/stats", web::get().to(get_dashboard_stats))
            .route(
                "/admin/dashboard/activity",
                web::get().to(get_recent_activity),
            )
            .route("/admin/purchases", web::get().to(get_all_purchases))
            .route("/admin/export", web::get().to(export_listings))
            // Category routes
            .route("/categories", web::post().to(create_category))
            .route("/categories", web::get().to(get_categories))
            .route("/categories/{id}", web::get().to(get_category))
            .route("/categories/{id}", web::put().to(update_category))
            .route("/categories/{id}", web::delete().to(delete_category))
            // Item routes
            .route("/items", web::post().to(create_item))
            .route("/items", web::get().to(get_items))
            .route("/items/{item_id}", web::get().to(get_item))
            .route("/items/{item_id}", web::put().to(update_item))
            .route("/items/{item_id}", web::delete().to(delete_item))
            .route("/items/{item_id}/purchase", web::post().to(purchase_item))
            // Bid routes
            .route("/items/{item_id}/bids", web::post().to(create_bid))
            .route("/items/{item_id}/bids", web::get().to(get_bids_for_item))
            .route("/bids", web::get().to(get_bids))
            .route("/bids/{bid_id}", web::get().to(get_bid))
            // Cart routes
            .route("/cart", web::get().to(get_cart))
            .route("/cart", web::delete().to(clear_cart))
            .route("/cart/items/{item_id}", web::post().to(add_to_cart))
            .route("/cart/items/{item_id}", web::delete().to(remove_from_cart))
            // Auction routes
            .route("/auctions/stats", web::get().to(get_auction_stats))
            .route("/auctions/results", web::get().to(get_auction_results))
            .route(
                "/auctions/results/{item_id}",
                web::get().to(get_auction_result),
            )
            .route(
                "/admin/auctions/{item_id}/end",
                web::post().to(force_end_auction),
            )
            // Image routes
            .route("/items/{item_id}/images", web::post().to(upload_images))
            .route("/items/{item_id}/images", web::get().to(get_item_images))
            .route(
                "/items/{item_id}/images/{image_id}",
                web::delete().to(delete_image),
            )
            .route(
                "/uploads/items/{item_id}/{filename}",
                web::get().to(serve_image),
            )
            // Recommendation routes
            .route(
                "/recommendations/{user_id}",
                web::get().to(get_recommendations),
            )
            .route("/track-view", web::post().to(track_category_view))
            .route("/admin/retrain-model", web::post().to(retrain_model))
            // WebSocket routes
            .route("/ws", web::get().to(websocket_handler))
            .route("/admin/websockets/stats", web::get().to(websocket_stats))
            .route(
                "/items/{item_id}/websockets/stats",
                web::get().to(item_websocket_stats),
            )
            .route(
                "/admin/users/{user_id}/websocket-stats",
                web::get().to(get_user_websocket_stats),
            )
            // Message routes
            .route("/chat/rooms", web::post().to(create_chat_room))
            .route(
                "/chat/rooms/{room_id}/messages",
                web::get().to(get_messages),
            )
            .route(
                "/chat/rooms/{room_id}/messages",
                web::post().to(send_message),
            )
            .route(
                "/chat/rooms/{room_id}/messages/{message_id}",
                web::delete().to(delete_message),
            )
            .app_data(db_pool.clone())
            .app_data(base_url.clone())
            .app_data(websocket_service.clone())
            .app_data(user_websocket_service.clone())
            .app_data(image_service.clone())
            .app_data(recommendation_service.clone())
            .app_data(message_service.clone())
    });

    server = if let Some(ssl_config) = ssl_settings {
        if ssl_config.enabled {
            tracing::info!("Starting HTTPS server with SSL");
            let ssl_config = load_ssl_config(&ssl_config).map_err(std::io::Error::other)?;
            server.listen_rustls(listener, ssl_config)?
        } else {
            tracing::info!("Starting HTTP server (SSL disabled)");
            server.listen(listener)?
        }
    } else {
        tracing::info!("Starting HTTP server (no SSL config)");
        server.listen(listener)?
    };

    Ok(server.run())
}
