//! Game state and turn progression logic.

use crate::types::{ImageId, OptionId, PlayerId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// The stage of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameStage {
    /// Revealing the communal goal and starting objects.
    RevealGoal,
    /// Active player turns.
    PlayerTurn,
    /// Voting on results.
    Voting,
    /// Final results and podium.
    Results,
}

/// The state of an active game.
///
/// Tracks the goal image, current image, turn order, and action history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// The target image players are trying to reach.
    pub goal_image: ImageId,
    
    /// The description of the communal goal.
    pub communal_goal: String,
    
    /// The initial image at the start of the game.
    pub starting_image: ImageId,
    
    /// The current image (updated after each turn).
    pub current_image: ImageId,
    
    /// The starting object assigned to each player.
    pub player_starting_objects: HashMap<PlayerId, String>,

    /// The current object description for each player (evolves during the game).
    pub player_current_objects: HashMap<PlayerId, String>,
    
    /// Current stage of the game.
    pub stage: GameStage,
    
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

    /// The 4 options available to the current player.
    pub current_options: Vec<String>,

    /// Timestamp when the current turn started (Unix seconds).
    pub turn_start_time: Option<u64>,

    /// Votes received: Voter -> Target -> Stars (0-5).
    pub votes: HashMap<PlayerId, HashMap<PlayerId, u8>>,

    /// Players who have submitted their votes.
    pub players_who_voted: HashSet<PlayerId>,

    /// Timestamp when the current stage started (Unix seconds).
    pub stage_start_time: u64,
}

impl GameState {
    /// Create a new game state.
    pub fn new(
        goal_image: ImageId,
        communal_goal: String,
        starting_image: ImageId,
        player_starting_objects: HashMap<PlayerId, String>,
        players: Vec<PlayerId>,
        max_rounds: u32,
    ) -> Self {
        let current_image = starting_image.clone();
        let player_current_objects = player_starting_objects.clone();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            goal_image,
            communal_goal,
            starting_image,
            current_image,
            player_starting_objects,
            player_current_objects,
            stage: GameStage::RevealGoal,
            players_in_order: players,
            current_turn_index: 0,
            max_rounds,
            current_round: 0,
            actions: Vec::new(),
            current_options: Vec::new(),
            turn_start_time: None,
            votes: HashMap::new(),
            players_who_voted: HashSet::new(),
            stage_start_time: now,
        }
    }

    /// Transition to the next stage.
    pub fn next_stage(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.stage_start_time = now;

        match self.stage {
            GameStage::RevealGoal => {
                self.stage = GameStage::PlayerTurn;
                self.start_turn();
            },
            GameStage::PlayerTurn => self.stage = GameStage::Voting,
            GameStage::Voting => self.stage = GameStage::Results,
            GameStage::Results => {}
        }
    }

    /// Get the ID of the current player.
    pub fn current_player(&self) -> Option<PlayerId> {
        self.players_in_order.get(self.current_turn_index).copied()
    }

    /// Start the turn for the current player.
    pub fn start_turn(&mut self) {
        if let Some(_) = self.current_player() {
            self.current_options = crate::assets::generate_modification_options();
            self.turn_start_time = Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs());
        }
    }

    /// Submit an action for the current player.
    pub fn submit_action(&mut self, player_id: PlayerId, option_index: Option<usize>) -> Result<(), String> {
        if self.stage != GameStage::PlayerTurn {
            return Err("Not in turn stage".to_string());
        }
        if Some(player_id) != self.current_player() {
            return Err("Not your turn".to_string());
        }
        
        // Apply modification if option chosen
        if let Some(idx) = option_index {
            if idx >= self.current_options.len() {
                return Err("Invalid option".to_string());
            }
            let modifier = &self.current_options[idx];
            
            if let Some(obj) = self.player_current_objects.get_mut(&player_id) {
                *obj = crate::assets::apply_modification(obj, modifier);
                
                self.actions.push(PlayerAction {
                    player_id,
                    round: self.current_round,
                    option_chosen: Some(idx),
                    modification: modifier.clone(),
                    resulting_object: obj.clone(),
                });
            }
        } else {
             // Timeout or skip
             self.actions.push(PlayerAction {
                player_id,
                round: self.current_round,
                option_chosen: None,
                modification: "No action".to_string(),
                resulting_object: self.player_current_objects.get(&player_id).cloned().unwrap_or_default(),
            });
        }

        self.advance_turn();
        Ok(())
    }

    /// Advance to the next player's turn.
    fn advance_turn(&mut self) {
        self.current_turn_index += 1;
        
        // If we've gone through all players, start a new round
        if self.current_turn_index >= self.players_in_order.len() {
            self.current_turn_index = 0;
            self.current_round += 1;
        }

        if self.current_round >= self.max_rounds {
            self.stage = GameStage::Voting;
            self.stage_start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        } else {
            self.start_turn();
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

    /// Submit votes from one player for multiple targets.
    pub fn submit_votes(&mut self, voter_id: PlayerId, votes: HashMap<PlayerId, u8>) -> Result<(), String> {
        if self.stage != GameStage::Voting {
            return Err("Not in voting stage".to_string());
        }
        
        // Validate votes
        for (target_id, stars) in &votes {
            if *target_id == voter_id {
                return Err("Cannot vote for yourself".to_string());
            }
            if *stars > 5 {
                return Err("Stars must be between 0 and 5".to_string());
            }
        }

        // Store votes
        self.votes.insert(voter_id, votes);
        self.players_who_voted.insert(voter_id);

        // Check if all players have voted
        // Note: We only expect votes from connected players, but for simplicity we check against all players in order
        // In a real scenario, we might want to handle disconnected players better.
        if self.players_who_voted.len() >= self.players_in_order.len() {
            self.stage = GameStage::Results;
            self.stage_start_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
            
        Ok(())
    }

    /// Calculate scores for all players.
    pub fn calculate_scores(&self) -> HashMap<PlayerId, f32> {
        let mut scores = HashMap::new();
        
        for player_id in &self.players_in_order {
            let mut total_stars = 0;
            let mut vote_count = 0;
            
            for voter_votes in self.votes.values() {
                if let Some(&stars) = voter_votes.get(player_id) {
                    total_stars += stars as u32;
                    vote_count += 1;
                }
            }
            
            if vote_count > 0 {
                scores.insert(*player_id, total_stars as f32 / vote_count as f32);
            } else {
                scores.insert(*player_id, 0.0);
            }
        }
        
        scores
    }
}

/// A single player action during the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    /// The player who took this action.
    pub player_id: PlayerId,
    
    /// The round number when this action was taken (0-based).
    pub round: u32,
    
    /// The option chosen by the player (index).
    pub option_chosen: Option<usize>,
    
    /// Text description of the modification.
    pub modification: String,
    
    /// The resulting object description.
    pub resulting_object: String,
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
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
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
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
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
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
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
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
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

    #[test]
    fn test_game_with_single_player() {
        let players = vec![PlayerId::new()];
        let mut game = GameState::new(
            ImageId::new("goal"),
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
            players.clone(),
            3,
        );
        
        assert_eq!(game.player_count(), 1);
        assert_eq!(game.current_player(), Some(players[0]));
        
        // Record action - should advance to next round since only 1 player
        let action = PlayerAction::new(
            players[0],
            0,
            OptionId::new(1),
            "Solo action".to_string(),
            ImageId::new("img1"),
        );
        game.record_action(action);
        
        assert_eq!(game.current_round, 1);
        assert_eq!(game.current_player(), Some(players[0]));
    }

    #[test]
    fn test_game_with_max_players() {
        let players: Vec<PlayerId> = (0..8).map(|_| PlayerId::new()).collect();
        let game = GameState::new(
            ImageId::new("goal"),
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
            players.clone(),
            2,
        );
        
        assert_eq!(game.player_count(), 8);
    }

    #[test]
    fn test_game_turn_wrapping() {
        let players = vec![PlayerId::new(), PlayerId::new(), PlayerId::new()];
        let mut game = GameState::new(
            ImageId::new("goal"),
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
            players.clone(),
            1,
        );
        
        // Play all 3 players in round 0
        for (idx, &player) in players.iter().enumerate() {
            assert_eq!(game.current_player(), Some(player));
            let action = PlayerAction::new(
                player,
                0,
                OptionId::new(idx as u8),
                format!("Action {}", idx),
                ImageId::new(format!("img{}", idx)),
            );
            game.record_action(action);
        }
        
        // Should wrap back to round 1, player 0
        assert_eq!(game.current_round, 1);
        assert_eq!(game.current_turn_index, 0);
    }

    #[test]
    fn test_player_action_fields() {
        let player_id = PlayerId::new();
        let action = PlayerAction::new(
            player_id,
            5,
            OptionId::new(3),
            "Test description".to_string(),
            ImageId::new("result_image"),
        );
        
        assert_eq!(action.player_id, player_id);
        assert_eq!(action.round, 5);
        assert_eq!(action.option_chosen.as_u8(), 3);
        assert_eq!(action.description, "Test description");
        assert_eq!(action.resulting_image.as_str(), "result_image");
    }

    #[test]
    fn test_game_state_serialization() {
        let players = vec![PlayerId::new(), PlayerId::new()];
        let mut game = GameState::new(
            ImageId::new("goal_img"),
            "A test goal".to_string(),
            ImageId::new("start_img"),
            std::collections::HashMap::new(),
            players.clone(),
            3,
        );
        
        let action = PlayerAction::new(
            players[0],
            0,
            OptionId::new(1),
            "First move".to_string(),
            ImageId::new("after_move"),
        );
        game.record_action(action);
        
        let json = serde_json::to_string(&game).expect("Should serialize");
        let deserialized: GameState = serde_json::from_str(&json).expect("Should deserialize");
        
        assert_eq!(deserialized.goal_image, game.goal_image);
        assert_eq!(deserialized.current_image, game.current_image);
        assert_eq!(deserialized.players_in_order.len(), game.players_in_order.len());
        assert_eq!(deserialized.actions.len(), game.actions.len());
        assert_eq!(deserialized.current_round, game.current_round);
    }

    #[test]
    fn test_game_total_turns() {
        let players = vec![PlayerId::new(), PlayerId::new()];
        let game = GameState::new(
            ImageId::new("goal"),
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
            players,
            5,
        );
        
        // Total expected turns = 2 players * 5 rounds = 10
        // But total_turns() returns actions.len() which is 0 initially
        assert_eq!(game.total_turns(), 0);
        
        // Max possible turns calculation
        assert_eq!(game.player_count() * game.max_rounds as usize, 10);
    }

    #[test]
    fn test_game_current_player_none_when_finished() {
        let players = vec![PlayerId::new()];
        let mut game = GameState::new(
            ImageId::new("goal"),
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
            players.clone(),
            1,
        );
        
        let action = PlayerAction::new(
            players[0],
            0,
            OptionId::new(0),
            "Final action".to_string(),
            ImageId::new("final"),
        );
        game.record_action(action);
        
        assert!(game.is_finished());
        // After finishing, current_round >= max_rounds, so current_player should still work
        // but the game is finished
    }

    #[test]
    fn test_multiple_rounds_progression() {
        let players = vec![PlayerId::new(), PlayerId::new()];
        let mut game = GameState::new(
            ImageId::new("goal"),
            "A test goal".to_string(),
            ImageId::new("start"),
            std::collections::HashMap::new(),
            players.clone(),
            3,
        );
        
        for round in 0..3 {
            for (idx, &player) in players.iter().enumerate() {
                assert_eq!(game.current_round, round);
                assert_eq!(game.current_player(), Some(player));
                
                let action = PlayerAction::new(
                    player,
                    round,
                    OptionId::new(idx as u8),
                    format!("Round {} Player {}", round, idx),
                    ImageId::new(format!("r{}_p{}", round, idx)),
                );
                game.record_action(action);
            }
        }
        
        assert!(game.is_finished());
        assert_eq!(game.actions.len(), 6); // 3 rounds * 2 players
    }
}
