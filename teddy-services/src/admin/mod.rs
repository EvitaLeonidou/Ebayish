// Admin services module

pub mod seeding;

// Re-export admin services for convenient access
pub use seeding::{SeedingService, SeedingError};