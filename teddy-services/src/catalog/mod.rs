// Catalog services module

pub mod items;
pub mod categories;
pub mod search;

// Re-export CategoryService and related types for easy access
pub use categories::{Category, CategoryService, CategoryServiceError};

// Re-export ItemService and related types for easy access
pub use items::{Item, ItemService, ItemServiceError};