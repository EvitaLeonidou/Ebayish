//! src/routes/mod.rs

mod admin;
mod auctions;
mod bids;
mod cart;
mod categories;
mod healthcheck;
mod images;
mod items;
pub mod login;
mod messages;
mod notifications;
mod recommendations;
mod user_profile;
mod user_role;
mod user_websockets;
mod users;
mod websockets;

pub use admin::*;
pub use auctions::*;
pub use bids::*;
pub use cart::*;
pub use categories::*;
pub use healthcheck::*;
pub use images::*;
pub use items::*;
pub use login::*;
pub use messages::*;
pub use notifications::*;
pub use recommendations::*;
pub use user_profile::*;
pub use user_role::*;
pub use user_websockets::*;
pub use users::*;
pub use websockets::*;
