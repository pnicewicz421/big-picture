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
