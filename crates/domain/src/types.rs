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
}
