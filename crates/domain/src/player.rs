//! Player entity and related types.

use crate::types::{AvatarId, PlayerId};
use serde::{Deserialize, Serialize};

/// A player in the game.
///
/// Players can be connected or disconnected. Disconnected players may rejoin
/// using the same nickname before the game finishes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player {
    /// Unique identifier for this player.
    pub id: PlayerId,
    
    /// Player's chosen nickname (used for display and rejoin matching).
    pub nickname: String,
    
    /// Player's chosen avatar.
    pub avatar_id: AvatarId,
    
    /// Whether the player is currently connected.
    pub connected: bool,
}

impl Player {
    /// Create a new player with the given nickname and avatar.
    pub fn new(nickname: String, avatar_id: AvatarId) -> Self {
        Self {
            id: PlayerId::new(),
            nickname,
            avatar_id,
            connected: true,
        }
    }

    /// Mark the player as disconnected.
    pub fn disconnect(&mut self) {
        self.connected = false;
    }

    /// Mark the player as reconnected.
    pub fn reconnect(&mut self) {
        self.connected = true;
    }

    /// Check if this player matches the given nickname (for rejoin).
    pub fn matches_nickname(&self, nickname: &str) -> bool {
        self.nickname == nickname
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new("Alice".to_string(), AvatarId::new(3));
        assert_eq!(player.nickname, "Alice");
        assert_eq!(player.avatar_id.as_u8(), 3);
        assert!(player.connected);
    }

    #[test]
    fn test_player_disconnect_reconnect() {
        let mut player = Player::new("Bob".to_string(), AvatarId::default());
        
        player.disconnect();
        assert!(!player.connected);
        
        player.reconnect();
        assert!(player.connected);
    }

    #[test]
    fn test_player_nickname_matching() {
        let player = Player::new("Charlie".to_string(), AvatarId::default());
        assert!(player.matches_nickname("Charlie"));
        assert!(!player.matches_nickname("charlie"));
        assert!(!player.matches_nickname("Bob"));
    }
}
