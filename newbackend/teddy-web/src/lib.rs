// teddy-web: HTTP endpoints and request/response handling

pub mod dto;
pub mod handlers;
pub mod middleware;
pub mod errors;
pub mod routes;
pub mod startup;

// Re-export the error macro for use in handlers
pub use errors::common::error_chain_fmt;

// Re-export middleware items for use in handlers
pub use middleware::{Claims, AuthenticatedUser};