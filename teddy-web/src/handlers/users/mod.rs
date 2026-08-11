// User handlers

pub mod registration;
pub mod management;
pub mod profile;

// Re-export functions for easy access
pub use registration::create_user;
pub use management::{get_all_users, suspend_user, activate_user};
pub use profile::get_user_by_id;