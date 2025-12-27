//! Lobby screen controller for displaying players and starting the game.

use godot::prelude::*;
use godot::classes::{Control, IControl, Label, Button, VBoxContainer, Timer, ITimer, HttpRequest};
use godot::classes::http_client::Method;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct LobbyScreen {
    base: Base<Control>,
    
    #[var]
    server_url: GString,
    
    #[var]
    room_id: GString,
    
    #[var]
    room_code: GString,
    
    #[var]
    player_id: GString,
    
    #[var]
    is_host: bool,
    q
    poll_timer: Option<Gd<Timer>>,
}

#[godot_api]
impl IControl for LobbyScreen {
    fn init(base: Base<Control>) -> Self {
        godot_print!("LobbyScreen initialized");
        Self {
            base,
            server_url: "http://192.168.1.19:3000".into(),
            room_id: "".into(),
            room_code: "".into(),
            player_id: "".into(),
            is_host: false,
            poll_timer: None,
        }
    }
    
    fn ready(&mut self) {
        godot_print!("LobbyScreen ready");
        godot_print!("Room: {}, Player: {}, Host: {}", self.room_code, self.player_id, self.is_host);
        
        // Create HTTPRequest node for polling
        let mut poll_request = HttpRequest::new_alloc();
        poll_request.set_name("PollRequest");
        self.base_mut().add_child(&poll_request);
        
        // Create timer for polling room state every 2 seconds
        let mut timer = Timer::new_alloc();
        timer.set_name("PollTimer");
        timer.set_wait_time(2.0);
        timer.set_autostart(true);
        timer.connect("timeout", &self.base().callable("on_poll_timer_timeout"));
        self.base_mut().add_child(&timer);
        self.poll_timer = Some(timer);
        
        // Connect start button
        {
            let callable = self.base().callable("on_start_game_pressed");
            if let Some(mut button) = self.base_mut().try_get_node_as::<Button>("VBoxContainer/StartGameButton") {
                button.connect("pressed", &callable);
            }
        }
        
        // Update UI with initial info
        self.update_room_info();
        
        // Do initial poll
        self.poll_room_state();
    }
}

#[godot_api]
impl LobbyScreen {
    #[func]
    fn set_room_info(&mut self, room_id: GString, room_code: GString, player_id: GString, is_host: bool) {
        godot_print!("Setting room info: room={}, code={}, player={}, host={}", room_id, room_code, player_id, is_host);
        self.room_id = room_id;
        self.room_code = room_code;
        self.player_id = player_id;
        self.is_host = is_host;
        
        self.update_room_info();
    }
    
    #[func]
    fn on_poll_timer_timeout(&mut self) {
        self.poll_room_state();
    }
    
    #[func]
    fn on_start_game_pressed(&mut self) {
        godot_print!("Start Game button pressed");
        
        if !self.is_host {
            godot_warn!("Only host can start the game");
            return;
        }
        
        let url = format!("{}/rooms/{}/start", self.server_url, self.room_id);
        godot_print!("Requesting: POST {}", url);
        
        let result = {
            let mut base = self.base_mut();
            if let Some(mut request) = base.try_get_node_as::<HttpRequest>("PollRequest") {
                let headers = PackedStringArray::new();
                Some(request.request_ex(&url)
                    .custom_headers(&headers)
                    .method(Method::POST)
                    .request_data("")
                    .done())
            } else {
                None
            }
        };
        
        if let Some(result) = result {
            if result != godot::global::Error::OK {
                godot_error!("Failed to start game: {:?}", result);
            }
        }
    }
    
    fn poll_room_state(&mut self) {
        if self.room_id.is_empty() {
            return;
        }
        
        let url = format!("{}/rooms/{}", self.server_url, self.room_id);
        
        // Connect signal first (separate scope)
        {
            let callable = self.base().callable("on_room_state_received");
            let mut base = self.base_mut();
            if let Some(mut request) = base.try_get_node_as::<HttpRequest>("PollRequest") {
                if !request.is_connected("request_completed", &callable) {
                    request.connect("request_completed", &callable);
                }
            }
        }
        
        // Make request (separate scope)
        let result = {
            let mut base = self.base_mut();
            if let Some(mut request) = base.try_get_node_as::<HttpRequest>("PollRequest") {
                let headers = PackedStringArray::new();
                Some(request.request_ex(&url)
                    .custom_headers(&headers)
                    .method(Method::GET)
                    .request_data("")
                    .done())
            } else {
                None
            }
        };
        
        if let Some(result) = result {
            if result != godot::global::Error::OK {
                godot_error!("Failed to poll room state: {:?}", result);
            }
        }
    }
    
    #[func]
    fn on_room_state_received(&mut self, _result: Variant, response_code: Variant, _headers: Variant, body: Variant) {
        let response_code = response_code.try_to::<i64>().unwrap_or(0) as i32;
        
        if response_code != 200 {
            godot_warn!("Failed to get room state: code={}", response_code);
            return;
        }
        
        let body_bytes = body.try_to::<PackedByteArray>().unwrap_or_default();
        let body_vec = body_bytes.to_vec();
        let body_str = String::from_utf8_lossy(&body_vec);
        
        // Parse JSON response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
            let player_count = json["player_count"].as_i64().unwrap_or(0);
            let state = json["state"].as_str().unwrap_or("Unknown");
            
            godot_print!("Room state: {} players, state: {}", player_count, state);
            
            // Update player list
            if let Some(players) = json["players"].as_array() {
                self.update_player_list(players);
            }
            
            // Enable/disable start button based on player count
            if self.is_host {
                self.set_start_button_enabled(player_count >= 2);
            }
            
            // Check if game has started
            if state == "InGame" {
                godot_print!("Game has started! Transitioning to game screen...");
                // TODO: Transition to game screen
            }
        }
    }
    
    fn update_room_info(&mut self) {
        let room_code = self.room_code.clone();
        let is_host = self.is_host;
        
        // Update room code label
        if let Some(mut label) = self.base_mut().try_get_node_as::<Label>("VBoxContainer/RoomCodeLabel") {
            label.set_text(&format!("Room Code: {}", room_code));
        }
        
        // Show/hide start button based on host status
        if let Some(mut button) = self.base_mut().try_get_node_as::<Button>("VBoxContainer/StartGameButton") {
            button.set_visible(is_host);
            button.set_disabled(true); // Will be enabled when 2+ players
        }
    }
    
    fn update_player_list(&mut self, players: &[serde_json::Value]) {
        if let Some(mut container) = self.base_mut().try_get_node_as::<VBoxContainer>("VBoxContainer/PlayersContainer") {
            // Clear existing player labels (but keep the first child which is the header)
            let child_count = container.get_child_count();
            for i in (1..child_count).rev() {
                if let Some(mut child) = container.get_child(i) {
                    container.remove_child(&child);
                    child.queue_free();
                }
            }
            
            // Add new player labels
            for player in players {
                let nickname = player["nickname"].as_str().unwrap_or("Unknown");
                let connected = player["connected"].as_bool().unwrap_or(false);
                let status = if connected { "✓" } else { "✗" };
                
                let mut label = Label::new_alloc();
                label.set_text(&format!("{} {}", status, nickname));
                container.add_child(&label);
            }
        }
    }
    
    fn set_start_button_enabled(&mut self, enabled: bool) {
        if let Some(mut button) = self.base_mut().try_get_node_as::<Button>("VBoxContainer/StartGameButton") {
            button.set_disabled(!enabled);
        }
    }
}
