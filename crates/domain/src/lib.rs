//! # Big Picture Domain
//!
//! Core domain types and business logic for the Big Picture multiplayer game.
//!
//! This crate is pure Rust with no external service dependencies, containing:
//! - Game entities: Room, Player, GameState
//! - Business rules: lobby capacity, turn order, game flow
//! - Domain errors and validation logic
//!
//! ## Complexity Tracking
//!
//! This crate uses PMAT-style complexity tracking. See `complexity.md` for metrics.

// Re-export main domain types
pub mod types;
pub mod room;
pub mod player;
pub mod game;
pub mod errors;
pub mod room_manager;
pub mod assets;

// Re-export commonly used types at crate root
pub use game::{GameOutcome, GameState, PlayerAction};
pub use player::Player;
pub use room::{Room, RoomState};
pub use types::{AvatarId, ImageId, OptionId, PlayerId, RoomId};
pub use errors::{RoomError, JoinError};
pub use room_manager::RoomManager;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_exports() {
        // Verify all main types are accessible
        let _room = Room::new("TEST".to_string());
        let _player = Player::new("Test".to_string(), AvatarId::default());
        let _room_id = RoomId::new();
        let _player_id = PlayerId::new();
    }
}
