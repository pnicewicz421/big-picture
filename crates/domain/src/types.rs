//! Core type aliases and identifiers for the Big Picture domain.
//!
//! These types provide strong typing for various IDs and ensure type safety
//! throughout the domain model.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a game room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(Uuid);

impl RoomId {
    /// Create a new random RoomId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Create a RoomId from a string (for deserializing from URLs/JSON).
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for RoomId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(Uuid);

impl PlayerId {
    /// Create a new random PlayerId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Create a PlayerId from a string (for deserializing from URLs/JSON).
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for PlayerId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Avatar identifier (0-9 for initial set of 10 avatars).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AvatarId(u8);

impl AvatarId {
    /// Create a new AvatarId from a u8 value (0-9).
    ///
    /// # Panics
    /// Panics if the value is > 9 in debug mode.
    pub fn new(id: u8) -> Self {
        debug_assert!(id < 10, "AvatarId must be 0-9");
        Self(id)
    }

    /// Get the inner u8 value.
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    /// Default avatar (avatar 0).
    pub fn default() -> Self {
        Self(0)
    }
}

impl fmt::Display for AvatarId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Avatar{}", self.0)
    }
}

/// Unique identifier for an AI-generated image.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ImageId(String);

impl ImageId {
    /// Create a new ImageId from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string reference.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ImageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ImageId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ImageId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Identifier for a modification option presented to the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OptionId(u8);

impl OptionId {
    /// Create a new OptionId (0-3 for four options).
    pub fn new(id: u8) -> Self {
        debug_assert!(id < 4, "OptionId must be 0-3 (four options)");
        Self(id)
    }

    /// Get the inner u8 value.
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl fmt::Display for OptionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Option{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_id_creation() {
        let id1 = RoomId::new();
        let id2 = RoomId::new();
        assert_ne!(id1, id2, "Room IDs should be unique");
    }

    #[test]
    fn test_player_id_creation() {
        let id1 = PlayerId::new();
        let id2 = PlayerId::new();
        assert_ne!(id1, id2, "Player IDs should be unique");
    }

    #[test]
    fn test_avatar_id() {
        let avatar = AvatarId::new(5);
        assert_eq!(avatar.as_u8(), 5);
    }

    #[test]
    fn test_image_id() {
        let img = ImageId::new("image_12345");
        assert_eq!(img.as_str(), "image_12345");
    }

    #[test]
    fn test_option_id() {
        let opt = OptionId::new(2);
        assert_eq!(opt.as_u8(), 2);
    }

    #[test]
    fn test_room_id_serialization() {
        let id = RoomId::new();
        let json = serde_json::to_string(&id).expect("Should serialize");
        let deserialized: RoomId = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized, id);
    }

    #[test]
    fn test_player_id_serialization() {
        let id = PlayerId::new();
        let json = serde_json::to_string(&id).expect("Should serialize");
        let deserialized: PlayerId = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized, id);
    }

    #[test]
    fn test_avatar_id_bounds() {
        let a0 = AvatarId::new(0);
        let a7 = AvatarId::new(7);
        let a9 = AvatarId::new(9);
        
        assert_eq!(a0.as_u8(), 0);
        assert_eq!(a7.as_u8(), 7);
        assert_eq!(a9.as_u8(), 9);
    }

    #[test]
    fn test_avatar_id_default() {
        let avatar = AvatarId::default();
        assert_eq!(avatar.as_u8(), 0);
    }

    #[test]
    fn test_image_id_empty_string() {
        let img = ImageId::new("");
        assert_eq!(img.as_str(), "");
    }

    #[test]
    fn test_image_id_long_string() {
        let long_id = "very_long_image_identifier_with_many_characters_12345";
        let img = ImageId::new(long_id);
        assert_eq!(img.as_str(), long_id);
    }

    #[test]
    fn test_option_id_serialization() {
        let opt = OptionId::new(3);
        let json = serde_json::to_string(&opt).expect("Should serialize");
        let deserialized: OptionId = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.as_u8(), opt.as_u8());
    }

    #[test]
    fn test_room_state_variants() {
        use crate::room::RoomState::*;
        
        let lobby = Lobby;
        let in_game = InGame;
        let finished = Finished;
        
        assert!(matches!(lobby, Lobby));
        assert!(matches!(in_game, InGame));
        assert!(matches!(finished, Finished));
    }

    #[test]
    fn test_room_state_serialization() {
        use crate::room::RoomState::{self, *};
        
        let states = vec![Lobby, InGame, Finished];
        
        for state in states {
            let json = serde_json::to_string(&state).expect("Should serialize");
            let deserialized: RoomState = serde_json::from_str(&json).expect("Should deserialize");
            assert_eq!(deserialized, state);
        }
    }

    #[test]
    fn test_id_types_are_copy() {
        let id1 = RoomId::new();
        let id2 = id1; // Copy
        assert_eq!(id1, id2);
        
        let pid1 = PlayerId::new();
        let pid2 = pid1; // Copy
        assert_eq!(pid1, pid2);
    }

    #[test]
    fn test_avatar_id_equality() {
        let a1 = AvatarId::new(5);
        let a2 = AvatarId::new(5);
        let a3 = AvatarId::new(3);
        
        assert_eq!(a1, a2);
        assert_ne!(a1, a3);
    }
}
