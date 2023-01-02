mod error;
mod traits;

pub mod generic;
pub mod steam_deck;

pub use error::StateError;
pub use traits::OnPoll;
