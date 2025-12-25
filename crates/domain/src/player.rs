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

    #[test]
    fn test_player_id_uniqueness() {
        let p1 = Player::new("Alice".to_string(), AvatarId::default());
        let p2 = Player::new("Alice".to_string(), AvatarId::default());
        
        // Same nickname but different IDs
        assert_ne!(p1.id, p2.id);
    }

    #[test]
    fn test_player_avatar_ids() {
        let p0 = Player::new("Player0".to_string(), AvatarId::new(0));
        let p7 = Player::new("Player7".to_string(), AvatarId::new(7));
        
        assert_eq!(p0.avatar_id.as_u8(), 0);
        assert_eq!(p7.avatar_id.as_u8(), 7);
    }

    #[test]
    fn test_player_connection_state() {
        let mut player = Player::new("Test".to_string(), AvatarId::default());
        
        assert!(player.connected, "New player should be connected");
        
        player.disconnect();
        assert!(!player.connected);
        
        player.disconnect(); // Double disconnect should be safe
        assert!(!player.connected);
        
        player.reconnect();
        assert!(player.connected);
        
        player.reconnect(); // Double reconnect should be safe
        assert!(player.connected);
    }

    #[test]
    fn test_player_serialization() {
        let player = Player::new("SerTest".to_string(), AvatarId::new(5));
        
        let json = serde_json::to_string(&player).expect("Should serialize");
        let deserialized: Player = serde_json::from_str(&json).expect("Should deserialize");
        
        assert_eq!(deserialized.nickname, player.nickname);
        assert_eq!(deserialized.avatar_id, player.avatar_id);
        assert_eq!(deserialized.connected, player.connected);
    }

    #[test]
    fn test_player_empty_nickname() {
        let player = Player::new("".to_string(), AvatarId::default());
        assert_eq!(player.nickname, "");
    }

    #[test]
    fn test_player_long_nickname() {
        let long_name = "ThisIsAVeryLongNicknameThatShouldStillWork";
        let player = Player::new(long_name.to_string(), AvatarId::default());
        assert_eq!(player.nickname, long_name);
    }

    #[test]
    fn test_player_special_characters_nickname() {
        let special = "Alice_123!@#";
        let player = Player::new(special.to_string(), AvatarId::default());
        assert_eq!(player.nickname, special);
    }
}
