// Business entities module

pub mod bid;
pub mod category;
pub mod item;
pub mod user;
pub mod image;

// Re-export main domain types
pub use bid::NewBid;
pub use category::{CategoryName, NewCategory};
pub use image::{DisplayOrderManager, FileValidator, ItemImage, NewItemImage};
pub use item::NewItem;
pub use user::{NewUser, UserEmail, Username};