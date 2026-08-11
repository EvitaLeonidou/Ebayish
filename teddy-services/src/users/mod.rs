// User services module

pub mod management;
pub mod authentication;

// Re-export key types and services for convenient access
pub use management::{UserService, UserServiceError, PendingUser, UserCredentials};