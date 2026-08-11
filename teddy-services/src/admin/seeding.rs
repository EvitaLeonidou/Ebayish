// Seeding service functionality - placeholder
// Will contain seeding_service.rs functionality when moved from backend

use sqlx::PgPool;

/// Placeholder SeedingService struct for teddy-services crate
/// Full implementation exists in backend/src/services/seeding_service.rs
pub struct SeedingService;

#[derive(thiserror::Error, Debug)]
pub enum SeedingError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Seeding already completed")]
    AlreadyCompleted,
    #[error("Seeding functionality not implemented in teddy-services")]
    NotImplemented,
}

impl SeedingService {
    /// Placeholder method - full implementation in backend crate
    pub fn new() -> Self {
        Self
    }

    /// Placeholder seed_database method for compatibility with backend
    /// Full implementation exists in backend/src/services/seeding_service.rs
    pub async fn seed_database<T>(
        _pool: &PgPool,
        _settings: &T,
    ) -> Result<(), SeedingError> {
        // This is a placeholder - the actual implementation should be moved from backend
        tracing::warn!("SeedingService::seed_database called on placeholder implementation");
        Err(SeedingError::NotImplemented)
    }
}