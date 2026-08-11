// Middleware modules

pub mod jwt;
pub mod cors;
pub mod logging;

// Re-export commonly used middleware items
pub use jwt::{Claims, AuthenticatedUser};