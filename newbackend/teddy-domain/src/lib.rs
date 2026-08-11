// teddy-domain: Core business entities and domain logic

pub mod entities;
pub mod errors;
pub mod utils;

// Re-export main domain types for convenience
pub use entities::{
    CategoryName, DisplayOrderManager, FileValidator, ItemImage, NewBid, NewCategory, NewItem,
    NewItemImage, NewUser, UserEmail, Username,
};
pub use utils::hash_password;