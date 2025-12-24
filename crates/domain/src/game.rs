//! Game state and turn progression logic.

use crate::types::{ImageId, OptionId, PlayerId};
use serde::{Deserialize, Serialize};

/// The state of an active game.
///
/// Tracks the goal image, current image, turn order, and action history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// The target image players are trying to reach.
    pub goal_image: ImageId,
    
    /// The initial image at the start of the game.
    pub starting_image: ImageId,
    
    /// The current image (updated after each turn).
    pub current_image: ImageId,
    
    /// Players in turn order.
    pub players_in_order: Vec<PlayerId>,
    
    /// Index of the current player (0-based).
    pub current_turn_index: usize,
    
    /// Maximum number of rounds (each player acts once per round).
    pub max_rounds: u32,
    
    /// Current round number (0-based).
    pub current_round: u32,
    
    /// History of all actions taken.
    pub actions: Vec<PlayerAction>,
}

impl GameState {
    /// Create a new game state.
    pub fn new(
        goal_image: ImageId,
        starting_image: ImageId,
        players: Vec<PlayerId>,
        max_rounds: u32,
    ) -> Self {
        let current_image = starting_image.clone();
        
        Self {
            goal_image,
            starting_image,
            current_image,
            players_in_order: players,
            current_turn_index: 0,
            max_rounds,
            current_round: 0,
            actions: Vec::new(),
        }
    }

    /// Get the ID of the current player.
    pub fn current_player(&self) -> Option<PlayerId> {
        self.players_in_order.get(self.current_turn_index).copied()
    }

    /// Record a player action and advance to the next turn.
    pub fn record_action(&mut self, action: PlayerAction) {
        self.current_image = action.resulting_image.clone();
        self.actions.push(action);
        self.advance_turn();
    }

    /// Advance to the next player's turn.
    fn advance_turn(&mut self) {
        self.current_turn_index += 1;
        
        // If we've gone through all players, start a new round
        if self.current_turn_index >= self.players_in_order.len() {
            self.current_turn_index = 0;
            self.current_round += 1;
        }
    }

    /// Check if the game has reached its maximum rounds.
    pub fn is_finished(&self) -> bool {
        self.current_round >= self.max_rounds
    }

    /// Get the total number of turns taken.
    pub fn total_turns(&self) -> usize {
        self.actions.len()
    }

    /// Get the number of players in the game.
    pub fn player_count(&self) -> usize {
        self.players_in_order.len()
    }
}

/// A single player action during the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    /// The player who took this action.
    pub player_id: PlayerId,
    
    /// The round number when this action was taken (0-based).
    pub round: u32,
    
    /// The option chosen by the player (0-3).
    pub option_chosen: OptionId,
    
    /// Text description of the modification.
    pub description: String,
    
    /// The resulting image after this action.
    pub resulting_image: ImageId,
}

impl PlayerAction {
    /// Create a new player action.
    pub fn new(
        player_id: PlayerId,
        round: u32,
        option_chosen: OptionId,
        description: String,
        resulting_image: ImageId,
    ) -> Self {
        Self {
            player_id,
            round,
            option_chosen,
            description,
            resulting_image,
        }
    }
}

/// The outcome of a game evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameOutcome {
    /// Players successfully reached the goal.
    Success,
    
    /// Players got close to the goal.
    Close,
    
    /// Players did not reach the goal.
    Fail,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_creation() {
        let players = vec![PlayerId::new(), PlayerId::new(), PlayerId::new()];
        let game = GameState::new(
            ImageId::new("goal"),
            ImageId::new("start"),
            players.clone(),
            3,
        );
        
        assert_eq!(game.player_count(), 3);
        assert_eq!(game.current_round, 0);
        assert_eq!(game.current_turn_index, 0);
        assert_eq!(game.current_player(), Some(players[0]));
        assert!(!game.is_finished());
    }

    #[test]
    fn test_turn_progression() {
        let players = vec![PlayerId::new(), PlayerId::new()];
        let mut game = GameState::new(
            ImageId::new("goal"),
            ImageId::new("start"),
            players.clone(),
            2,
        );
        
        // First action
        let action1 = PlayerAction::new(
            players[0],
            0,
            OptionId::new(0),
            "Add clouds".to_string(),
            ImageId::new("img1"),
        );
        game.record_action(action1);
        
        assert_eq!(game.current_turn_index, 1);
        assert_eq!(game.current_round, 0);
        assert_eq!(game.current_player(), Some(players[1]));
        
        // Second action (completes round 0)
        let action2 = PlayerAction::new(
            players[1],
            0,
            OptionId::new(1),
            "Add trees".to_string(),
            ImageId::new("img2"),
        );
        game.record_action(action2);
        
        assert_eq!(game.current_turn_index, 0);
        assert_eq!(game.current_round, 1);
        assert_eq!(game.current_player(), Some(players[0]));
    }

    #[test]
    fn test_game_finish_condition() {
        let players = vec![PlayerId::new(), PlayerId::new()];
        let mut game = GameState::new(
            ImageId::new("goal"),
            ImageId::new("start"),
            players.clone(),
            2, // 2 rounds max
        );
        
        assert!(!game.is_finished());
        
        // Play through 2 rounds (4 turns total)
        for round in 0..2 {
            for (idx, &player) in players.iter().enumerate() {
                let action = PlayerAction::new(
                    player,
                    round,
                    OptionId::new(idx as u8),
                    format!("Action {} in round {}", idx, round),
                    ImageId::new(format!("img_r{}_p{}", round, idx)),
                );
                game.record_action(action);
            }
        }
        
        assert!(game.is_finished());
        assert_eq!(game.total_turns(), 4);
    }

    #[test]
    fn test_action_history() {
        let players = vec![PlayerId::new()];
        let mut game = GameState::new(
            ImageId::new("goal"),
            ImageId::new("start"),
            players.clone(),
            1,
        );
        
        let action = PlayerAction::new(
            players[0],
            0,
            OptionId::new(2),
            "Change color".to_string(),
            ImageId::new("new_img"),
        );
        
        game.record_action(action.clone());
        
        assert_eq!(game.actions.len(), 1);
        assert_eq!(game.actions[0].description, "Change color");
        assert_eq!(game.current_image.as_str(), "new_img");
    }
}
