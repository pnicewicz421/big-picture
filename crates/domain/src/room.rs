//! Room entity and state management.

use crate::game::GameState;
use crate::player::Player;
use crate::types::{PlayerId, RoomId};
use serde::{Deserialize, Serialize};

/// The state of a game room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomState {
    /// Lobby phase: players can join and leave.
    Lobby,
    
    /// Game is in progress.
    InGame,
    
    /// Game has finished.
    Finished,
}

/// A game room that contains players and game state.
///
/// Rooms progress through states: Lobby → InGame → Finished.
/// Players can only join/leave during the Lobby state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    /// Unique identifier for this room.
    pub id: RoomId,
    
    /// Human-readable room code for joining (4-6 characters).
    pub code: String,
    
    /// Players in this room (max 8).
    pub players: Vec<Player>,
    
    /// Current state of the room.
    pub state: RoomState,
    
    /// Game state (only present when state is InGame or Finished).
    pub game: Option<GameState>,
}

impl Room {
    /// Create a new room with the given code.
    pub fn new(code: String) -> Self {
        Self {
            id: RoomId::new(),
            code,
            players: Vec::new(),
            state: RoomState::Lobby,
            game: None,
        }
    }

    /// Get the number of players currently in the room.
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Check if the room is full (8 players).
    pub fn is_full(&self) -> bool {
        self.players.len() >= 8
    }

    /// Check if the room has enough players to start (2-8).
    pub fn can_start(&self) -> bool {
        let count = self.players.len();
        count >= 2 && count <= 8
    }

    /// Check if a player with the given nickname exists in this room.
    pub fn has_player_with_nickname(&self, nickname: &str) -> bool {
        self.players.iter().any(|p| p.matches_nickname(nickname))
    }

    /// Find a player by ID.
    pub fn find_player(&self, player_id: PlayerId) -> Option<&Player> {
        self.players.iter().find(|p| p.id == player_id)
    }

    /// Find a player by ID (mutable).
    pub fn find_player_mut(&mut self, player_id: PlayerId) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.id == player_id)
    }

    /// Find a player by nickname.
    pub fn find_player_by_nickname(&self, nickname: &str) -> Option<&Player> {
        self.players.iter().find(|p| p.matches_nickname(nickname))
    }

    /// Add a player to the room.
    ///
    /// Returns the player's ID if successful.
    pub fn add_player(&mut self, player: Player) -> PlayerId {
        let id = player.id;
        self.players.push(player);
        id
    }

    /// Remove a player from the room by ID.
    ///
    /// Returns true if the player was found and removed.
    pub fn remove_player(&mut self, player_id: PlayerId) -> bool {
        if let Some(pos) = self.players.iter().position(|p| p.id == player_id) {
            self.players.remove(pos);
            true
        } else {
            false
        }
    }

    /// Transition the room to the InGame state.
    ///
    /// This should only be called when the room is in Lobby state
    /// and has 2-8 players.
    pub fn start_game(&mut self, game_state: GameState) {
        debug_assert!(self.state == RoomState::Lobby, "Can only start from Lobby");
        debug_assert!(self.can_start(), "Need 2-8 players to start");
        
        self.state = RoomState::InGame;
        self.game = Some(game_state);
    }

    /// Transition the room to the Finished state.
    pub fn finish_game(&mut self) {
        debug_assert!(self.state == RoomState::InGame, "Can only finish from InGame");
        self.state = RoomState::Finished;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AvatarId;

    fn create_test_player(nickname: &str) -> Player {
        Player::new(nickname.to_string(), AvatarId::default())
    }

    #[test]
    fn test_room_creation() {
        let room = Room::new("ABC123".to_string());
        assert_eq!(room.code, "ABC123");
        assert_eq!(room.state, RoomState::Lobby);
        assert_eq!(room.player_count(), 0);
        assert!(room.game.is_none());
    }

    #[test]
    fn test_room_add_remove_players() {
        let mut room = Room::new("TEST01".to_string());
        
        let p1 = create_test_player("Alice");
        let p1_id = room.add_player(p1);
        assert_eq!(room.player_count(), 1);
        
        let p2 = create_test_player("Bob");
        room.add_player(p2);
        assert_eq!(room.player_count(), 2);
        
        assert!(room.remove_player(p1_id));
        assert_eq!(room.player_count(), 1);
        
        assert!(!room.remove_player(p1_id), "Should not remove twice");
    }

    #[test]
    fn test_room_capacity() {
        let mut room = Room::new("FULL01".to_string());
        
        // Add 8 players
        for i in 0..8 {
            room.add_player(create_test_player(&format!("Player{}", i)));
        }
        
        assert!(room.is_full());
        assert_eq!(room.player_count(), 8);
    }

    #[test]
    fn test_room_can_start() {
        let mut room = Room::new("START1".to_string());
        
        assert!(!room.can_start(), "Cannot start with 0 players");
        
        room.add_player(create_test_player("Alice"));
        assert!(!room.can_start(), "Cannot start with 1 player");
        
        room.add_player(create_test_player("Bob"));
        assert!(room.can_start(), "Can start with 2 players");
        
        // Add more players (up to 8)
        for i in 3..=8 {
            room.add_player(create_test_player(&format!("Player{}", i)));
        }
        assert!(room.can_start(), "Can start with 8 players");
    }

    #[test]
    fn test_room_find_player() {
        let mut room = Room::new("FIND01".to_string());
        
        let player = create_test_player("Alice");
        let player_id = player.id;
        room.add_player(player);
        
        assert!(room.find_player(player_id).is_some());
        assert!(room.find_player_by_nickname("Alice").is_some());
        assert!(room.find_player_by_nickname("Bob").is_none());
    }

    #[test]
    fn test_room_has_player_with_nickname() {
        let mut room = Room::new("NICK01".to_string());
        
        room.add_player(create_test_player("Alice"));
        
        assert!(room.has_player_with_nickname("Alice"));
        assert!(!room.has_player_with_nickname("Bob"));
    }
}
