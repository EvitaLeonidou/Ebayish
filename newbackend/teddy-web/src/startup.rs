use teddy_infrastructure::config::{DatabaseSettings, Settings, SslSettings, UploadSettings};
use crate::routes;
use teddy_services::{
    auctions::countdown::CountdownService,
    media::storage::ImageService,
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
    let image_service = Data::new(ImageService::new(&upload_settings.upload_dir));

    // Start the countdown timer service
    CountdownService::start_countdown_timer(db_pool.get_ref().clone(), websocket_service.get_ref().clone());
    tracing::info!("Started countdown timer service for real-time auction updates");

    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(routes::configure_routes())
            .app_data(db_pool.clone())
            .app_data(base_url.clone())
            .app_data(websocket_service.clone())
            .app_data(image_service.clone())
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