//! src/configuration.rs
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub uploads: UploadSettings,
    pub seeding: SeedingSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
    pub ssl: Option<SslSettings>,
}

#[derive(serde::Deserialize, Clone)]
pub struct SslSettings {
    pub enabled: bool,
    pub certificate_path: String,
    pub private_key_path: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct UploadSettings {
    pub max_file_size: usize,
    pub max_files_per_item: usize,
    pub allowed_types: Vec<String>,
    pub upload_dir: String,
    pub temp_dir: String,
    pub enable_image_processing: Option<bool>,
    pub max_image_width: Option<u32>,
    pub max_image_height: Option<u32>,
}

impl Default for UploadSettings {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, //10mb
            max_files_per_item: 5,
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/webp".to_string(),
            ],
            upload_dir: "./uploads".to_string(),
            temp_dir: "./uploads/temp".to_string(),
            enable_image_processing: Some(false),
            max_image_width: Some(4096),
            max_image_height: Some(4096),
        }
    }
}

impl UploadSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_file_size == 0 {
            return Err("max_file_size must be greater than 0".to_string());
        }

        if self.max_file_size > 100 * 1024 * 1024 {
            // 100MB max
            return Err("max_file_size cannot exceed 100MB".to_string());
        }

        if self.max_files_per_item == 0 || self.max_files_per_item > 10 {
            return Err("max_files_per_item must be between 1 and 10".to_string());
        }

        if self.allowed_types.is_empty() {
            return Err("allowed_types cannot be empty".to_string());
        }

        for mime_type in &self.allowed_types {
            if !mime_type.starts_with("image/") {
                return Err(format!("Invalid MIME type: {}", mime_type));
            }
        }

        if self.upload_dir.trim().is_empty() {
            return Err("upload_dir cannot be empty".to_string());
        }

        if self.temp_dir.trim().is_empty() {
            return Err("temp_dir cannot be empty".to_string());
        }

        #[allow(clippy::collapsible_if)]
        if let Some(width) = self.max_image_width {
            if width == 0 || width > 10000 {
                return Err("max_image_width must be between 1 and 10000".to_string());
            }
        }

        #[allow(clippy::collapsible_if)]
        if let Some(height) = self.max_image_height {
            if height == 0 || height > 10000 {
                return Err("max_image_height must be between 1 and 10000".to_string());
            }
        }

        Ok(())
    }

    pub fn is_allowed_mime_type(&self, mime_type: &str) -> bool {
        self.allowed_types.contains(&mime_type.to_string())
    }

    pub fn get_upload_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.upload_dir)
    }

    pub fn get_temp_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.temp_dir)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct SeedingSettings {
    pub enabled: bool,
    pub environment_filter: Vec<String>,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u64,
    pub seller_username: String,
    pub seller_email: String,
    pub seller_first_name: String,
    pub seller_last_name: String,
    pub seller_phone: String,
    pub seller_location: String,
    pub seller_country: String,
}

impl Default for SeedingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            environment_filter: vec!["production".to_string(), "docker".to_string()],
            retry_attempts: 3,
            retry_delay_seconds: 2,
            seller_username: "o paliatzis".to_string(),
            seller_email: "paliatzis@example.com".to_string(),
            seller_first_name: "Παλιάτζης".to_string(),
            seller_last_name: "Ο".to_string(),
            seller_phone: "+30 2101234567".to_string(),
            seller_location: "Athens".to_string(),
            seller_country: "Greece".to_string(),
        }
    }
}

impl SeedingSettings {
    pub fn should_run_seeding(&self) -> bool {
        if !self.enabled {
            return false;
        }

        let current_environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".to_string())
            .to_lowercase();

        self.environment_filter
            .iter()
            .any(|env| env.to_lowercase() == current_environment)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.seller_username.trim().is_empty() {
            return Err("seller_username cannot be empty".to_string());
        }

        if self.seller_email.trim().is_empty() {
            return Err("seller_email cannot be empty".to_string());
        }

        if self.seller_first_name.trim().is_empty() {
            return Err("seller_first_name cannot be empty".to_string());
        }

        if self.seller_last_name.trim().is_empty() {
            return Err("seller_last_name cannot be empty".to_string());
        }

        if self.retry_attempts == 0 || self.retry_attempts > 10 {
            return Err("retry_attempts must be between 1 and 10".to_string());
        }

        if self.retry_delay_seconds == 0 || self.retry_delay_seconds > 60 {
            return Err("retry_delay_seconds must be between 1 and 60".to_string());
        }

        Ok(())
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    //detect the running envirnment
    //default to local
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    let environment_filename = format!("{}.yml", environment.as_str());

    //allow for configuaration coming from environment variables
    //not just configuration files
    let settings = config::Config::builder()
        .add_source(config::File::from(configuration_directory.join("base.yml")))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    let mut configuration: Settings = settings.try_deserialize()?;

    // Apply defaults if not specified
    if configuration.uploads.allowed_types.is_empty() {
        configuration.uploads.allowed_types = UploadSettings::default().allowed_types;
    }

    // Apply seeding defaults if not specified
    if configuration.seeding.seller_username.is_empty() {
        configuration.seeding = SeedingSettings::default();
    }

    configuration
        .uploads
        .validate()
        .map_err(|e| config::ConfigError::Message(format!("Upload configuration error: {}", e)))?;

    configuration
        .seeding
        .validate()
        .map_err(|e| config::ConfigError::Message(format!("Seeding configuration error: {}", e)))?;

    Ok(configuration)
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        )
    }

    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            //try encrypted if it fails fallback to unencrypted
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported Environment. use `local` or `production`."
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upload_settings_validation() {
        let valid_settings = UploadSettings::default();
        assert!(valid_settings.validate().is_ok());

        let mut invalid_settings = UploadSettings::default();
        invalid_settings.max_file_size = 0;
        assert!(invalid_settings.validate().is_err());

        invalid_settings = UploadSettings::default();
        invalid_settings.max_files_per_item = 0;
        assert!(invalid_settings.validate().is_err());

        invalid_settings = UploadSettings::default();
        invalid_settings.allowed_types.clear();
        assert!(invalid_settings.validate().is_err());
    }

    #[test]
    fn test_mime_type_validation() {
        let settings = UploadSettings::default();
        assert!(settings.is_allowed_mime_type("image/jpeg"));
        assert!(settings.is_allowed_mime_type("image/png"));
        assert!(!settings.is_allowed_mime_type("text/plain"));
        assert!(!settings.is_allowed_mime_type("video/mp4"));
    }

    #[test]
    fn test_file_size_limits() {
        let mut settings = UploadSettings::default();

        //valid file sizes
        settings.max_file_size = 1024;
        assert!(settings.validate().is_ok());

        settings.max_file_size = 50 * 1024 * 1024; // 50mb
        assert!(settings.validate().is_ok());

        //invalid file sizes
        settings.max_file_size = 0;
        assert!(settings.validate().is_err());

        settings.max_file_size = 101 * 1024 * 1024; // 101mb
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_directory_validation() {
        let mut settings = UploadSettings::default();

        settings.upload_dir = "".to_string();
        assert!(settings.validate().is_err());

        settings.upload_dir = "   ".to_string();
        assert!(settings.validate().is_err());

        settings = UploadSettings::default();
        settings.temp_dir = "".to_string();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_image_dimensions() {
        let mut settings = UploadSettings::default();

        // test valid dimensions
        settings.max_image_width = Some(1920);
        settings.max_image_height = Some(1080);
        assert!(settings.validate().is_ok());

        // test invalid dimensions
        settings.max_image_width = Some(0);
        assert!(settings.validate().is_err());

        settings.max_image_width = Some(1920);
        settings.max_image_height = Some(10001);
        assert!(settings.validate().is_err());

        // test None values (should be valid)
        settings.max_image_width = None;
        settings.max_image_height = None;
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_path_methods() {
        let settings = UploadSettings::default();

        assert_eq!(settings.get_upload_path().to_string_lossy(), "./uploads");
        assert_eq!(settings.get_temp_path().to_string_lossy(), "./uploads/temp");

        //test custom paths
        let mut custom_settings = settings.clone();
        custom_settings.upload_dir = "/custom/upload".to_string();
        custom_settings.temp_dir = "/custom/temp".to_string();

        assert_eq!(
            custom_settings.get_upload_path().to_string_lossy(),
            "/custom/upload"
        );
        assert_eq!(
            custom_settings.get_temp_path().to_string_lossy(),
            "/custom/temp"
        );
    }
}
