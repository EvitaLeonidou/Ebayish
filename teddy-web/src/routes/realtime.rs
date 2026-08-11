use actix_web::{web, Scope};
use crate::handlers::realtime::{
    stats::{item_websocket_stats, websocket_stats},
    websockets::websocket_handler,
};

pub fn configure() -> Scope {
    web::scope("")
        // WebSocket routes
        .route("/ws", web::get().to(websocket_handler))
        .route("/items/{item_id}/websockets/stats", web::get().to(item_websocket_stats))
        .service(
            web::scope("/admin")
                .route("/websockets/stats", web::get().to(websocket_stats))
        )
}