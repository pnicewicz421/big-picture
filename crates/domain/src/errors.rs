//! Domain-specific errors for room and player management.

use thiserror::Error;
use crate::types::{PlayerId, RoomId};

/// Errors that can occur during room operations.
#[derive(Debug, Error, serde::Serialize, serde::Deserialize)]
pub enum RoomError {
    #[error("Room not found: {0}")]
    NotFound(String),
    
    #[error("Room not found")]
    RoomNotFound,
    
    #[error("Room {0} is full (max 8 players)")]
    Full(RoomId),
    
    #[error("Room is full")]
    RoomFull,
    
    #[error("Game already started in room {0}")]
    AlreadyStarted(RoomId),
    
    #[error("Game already started")]
    GameAlreadyStarted,
    
    #[error("Player {0} not found in room {1}")]
    PlayerNotFound(PlayerId, RoomId),
    
    #[error("Player not found")]
    PlayerNotFoundSimple,
    
    #[error("Nickname '{0}' is already taken in room {1}")]
    NicknameTaken(String, RoomId),
    
    #[error("Not enough players to start (need at least 2)")]
    NotEnoughPlayers(RoomId),
    
    #[error("Invalid room code: {0}")]
    InvalidCode(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Errors that can occur when a player tries to join a room.
#[derive(Debug, Error, serde::Serialize, serde::Deserialize)]
pub enum JoinError {
    #[error("Room not found")]
    RoomNotFound,
    
    #[error("Room is full")]
    RoomFull,
    
    #[error("Game already in progress")]
    GameInProgress,
    
    #[error("Nickname already taken")]
    DuplicateNickname,
    
    #[error("Invalid nickname")]
    InvalidNickname,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_error_display() {
        let room_id = RoomId::new();
        let player_id = PlayerId::new();
        
        let err = RoomError::NotFound("ABC123".to_string());
        assert!(err.to_string().contains("ABC123"));
        
        let err = RoomError::Full(room_id);
        assert!(err.to_string().contains("full"));
        
        let err = RoomError::PlayerNotFound(player_id, room_id);
        assert!(err.to_string().contains("not found"));
        
        let err = RoomError::NicknameTaken("Alice".to_string(), room_id);
        assert!(err.to_string().contains("Alice"));
        assert!(err.to_string().contains("taken"));
    }

    #[test]
    fn test_room_error_serialization() {
        let errors = vec![
            RoomError::NotFound("TEST01".to_string()),
            RoomError::RoomNotFound,
            RoomError::RoomFull,
            RoomError::GameAlreadyStarted,
            RoomError::PlayerNotFoundSimple,
            RoomError::InvalidCode("INVALID".to_string()),
            RoomError::Internal("Something went wrong".to_string()),
        ];
        
        for error in errors {
            let json = serde_json::to_string(&error).expect("Should serialize");
            let _deserialized: RoomError = serde_json::from_str(&json).expect("Should deserialize");
        }
    }

    #[test]
    fn test_join_error_display() {
        let err = JoinError::RoomNotFound;
        assert!(err.to_string().contains("not found"));
        
        let err = JoinError::RoomFull;
        assert!(err.to_string().contains("full"));
        
        let err = JoinError::GameInProgress;
        assert!(err.to_string().contains("progress"));
        
        let err = JoinError::DuplicateNickname;
        assert!(err.to_string().contains("taken"));
    }

    #[test]
    fn test_join_error_serialization() {
        let errors = vec![
            JoinError::RoomNotFound,
            JoinError::RoomFull,
            JoinError::GameInProgress,
            JoinError::DuplicateNickname,
            JoinError::InvalidNickname,
        ];
        
        for error in errors {
            let json = serde_json::to_string(&error).expect("Should serialize");
            let _deserialized: JoinError = serde_json::from_str(&json).expect("Should deserialize");
        }
    }

    #[test]
    fn test_error_variants_are_error_trait() {
        use std::error::Error;
        
        let room_err: Box<dyn Error> = Box::new(RoomError::RoomNotFound);
        assert!(room_err.to_string().len() > 0);
        
        let join_err: Box<dyn Error> = Box::new(JoinError::RoomFull);
        assert!(join_err.to_string().len() > 0);
    }
}
