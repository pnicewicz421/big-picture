//! Welcome screen controller for creating/joining rooms.

use godot::prelude::*;
use godot::classes::{Control, IControl, LineEdit, Label, Button, HttpRequest};
use godot::classes::http_client::Method;

#[derive(GodotClass)]
#[class(base=Control)]
pub struct WelcomeScreen {
    base: Base<Control>,
    
    #[var]
    server_url: GString,
    
    // Store room/player info for transitioning to lobby
    room_id: Option<String>,
    room_code: Option<String>,
    player_id: Option<String>,
    player_nickname: Option<String>,
}

#[godot_api]
impl IControl for WelcomeScreen {
    fn init(base: Base<Control>) -> Self {
        godot_print!("WelcomeScreen initialized");
        Self {
            base,
            server_url: "http://localhost:3000".into(),
            room_id: None,
            room_code: None,
            player_id: None,
            player_nickname: None,
        }
    }
    
    fn ready(&mut self) {
        godot_print!("WelcomeScreen ready");
        self.set_status("Ready to play! Make sure server is running on localhost:3000", false);
        
        // Create HTTPRequest nodes for network calls
        let mut create_request = HttpRequest::new_alloc();
        create_request.set_name("CreateRoomRequest");
        self.base_mut().add_child(&create_request);
        
        let mut join_request = HttpRequest::new_alloc();
        join_request.set_name("JoinRoomRequest");
        self.base_mut().add_child(&join_request);
    }
}

#[godot_api]
impl WelcomeScreen {
    #[func]
    fn on_create_room_pressed(&mut self) {
        godot_print!("Create Room button pressed");
        self.set_status("Creating room...", false);
        self.set_button_enabled("CreateRoomButton", false);
        
        // Make HTTP POST request to create room
        let url = format!("{}/rooms", self.server_url);
        godot_print!("Requesting: POST {}", url);
        
        // Get the request node and make the HTTP call
        let result = {
            let mut base = self.base_mut();
            if let Some(mut request) = base.try_get_node_as::<HttpRequest>("CreateRoomRequest") {
                let headers = PackedStringArray::new();
                
                // POST = method 3 in Godot's HTTPRequest
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
                godot_error!("Failed to start HTTP request: {:?}", result);
                self.set_status("Failed to start request", true);
                self.set_button_enabled("CreateRoomButton", true);
                return;
            }
            
            // Connect the request_completed signal
            let callable = self.base().callable("on_create_room_completed");
            let mut base = self.base_mut();
            if let Some(mut request) = base.try_get_node_as::<HttpRequest>("CreateRoomRequest") {
                if !request.is_connected("request_completed", &callable) {
                    request.connect("request_completed", &callable);
                }
            }
        }
    }
    
    #[func]
    fn on_join_room_pressed(&mut self) {
        godot_print!("Join Room button pressed");
        
        // Get room code from input
        let code = self.get_room_code_input();
        if code.is_empty() {
            self.set_status("Please enter a room code", true);
            return;
        }
        
        // For now, use a random nickname
        let nickname = format!("Player{}", (godot::classes::Time::singleton().get_ticks_msec() % 9999));
        let avatar_id = 0;
        
        self.set_status(&format!("Joining room {}...", code), false);
        self.set_button_enabled("JoinButton", false);
        
        // Make HTTP POST request to join room
        let url = format!("{}/rooms/{}/join", self.server_url, code);
        let body = format!(r#"{{"nickname":"{}","avatar_id":{}}}"#, nickname, avatar_id);
        
        godot_print!("Requesting: POST {} with body: {}", url, body);
        
        // Get the request node and make the HTTP call
        let result = {
            let mut base = self.base_mut();
            if let Some(mut request) = base.try_get_node_as::<HttpRequest>("JoinRoomRequest") {
                let mut headers = PackedStringArray::new();
                headers.push("Content-Type: application/json");
                
                // POST = method 3
                Some(request.request_ex(&url)
                    .custom_headers(&headers)
                    .method(Method::POST)
                    .request_data(&body)
                    .done())
            } else {
                None
            }
        };
        
        if let Some(result) = result {
            if result != godot::global::Error::OK {
                godot_error!("Failed to start HTTP request: {:?}", result);
                self.set_status("Failed to start request", true);
                self.set_button_enabled("JoinButton", true);
                return;
            }
            
            // Connect the request_completed signal
            let callable = self.base().callable("on_join_room_completed");
            let mut base = self.base_mut();
            if let Some(mut request) = base.try_get_node_as::<HttpRequest>("JoinRoomRequest") {
                if !request.is_connected("request_completed", &callable) {
                    request.connect("request_completed", &callable);
                }
            }
        }
    }
    
    #[func]
    fn on_create_room_completed(&mut self, _result: Variant, response_code: Variant, _headers: Variant, body: Variant) {
        let response_code = response_code.try_to::<i64>().unwrap_or(0) as i32;
        godot_print!("Create room response: code={}", response_code);
        
        if response_code == 200 {
            let body_bytes = body.try_to::<PackedByteArray>().unwrap_or_default();
            let body_vec = body_bytes.to_vec();
            let body_str = String::from_utf8_lossy(&body_vec);
            godot_print!("Response body: {}", body_str);
            
            // Parse JSON response
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
                if let (Some(room_code), Some(room_id)) = (json["room_code"].as_str(), json["room_id"].as_str()) {
                    self.room_code = Some(room_code.to_string());
                    self.room_id = Some(room_id.to_string());
                    
                    self.set_status(&format!("Room created! Code: {}", room_code), false);
                    godot_print!("Room code: {}, Room ID: {}", room_code, room_id);
                    
                    // Auto-join as host
                    self.auto_join_as_host(room_code);
                    return;
                }
            }
        }
        
        self.set_status(&format!("Failed to create room (code: {})", response_code), true);
        self.set_button_enabled("CreateRoomButton", true);
    }
    
    #[func]
    fn on_join_room_completed(&mut self, _result: Variant, response_code: Variant, _headers: Variant, body: Variant) {
        let response_code = response_code.try_to::<i64>().unwrap_or(0) as i32;
        godot_print!("Join room response: code={}", response_code);
        
        if response_code == 200 {
            let body_bytes = body.try_to::<PackedByteArray>().unwrap_or_default();
            let body_vec = body_bytes.to_vec();
            let body_str = String::from_utf8_lossy(&body_vec);
            godot_print!("Response body: {}", body_str);
            
            // Parse JSON response
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
                if let (Some(room_id), Some(player_id)) = (json["room_id"].as_str(), json["player_id"].as_str()) {
                    self.player_id = Some(player_id.to_string());
                    if self.room_id.is_none() {
                        self.room_id = Some(room_id.to_string());
                    }
                    
                    let nickname = self.player_nickname.as_deref().unwrap_or("Player");
                    let code = self.room_code.as_deref().unwrap_or("???");
                    
                    self.set_status(&format!("Joined room {} as {}", code, nickname), false);
                    godot_print!("Player ID: {}, Room ID: {}", player_id, room_id);
                    godot_print!("Ready to transition to lobby!");
                    
                    self.set_button_enabled("JoinButton", true);
                    return;
                }
            }
        }
        
        let body_bytes = body.try_to::<PackedByteArray>().unwrap_or_default();
        let body_vec = body_bytes.to_vec();
        let body_str = String::from_utf8_lossy(&body_vec);
        self.set_status(&format!("Failed to join: {}", body_str), true);
        self.set_button_enabled("JoinButton", true);
    }
    
    fn auto_join_as_host(&mut self, room_code: &str) {
        let nickname = format!("Host{}", (godot::classes::Time::singleton().get_ticks_msec() % 9999));
        self.player_nickname = Some(nickname.clone());
        
        let url = format!("{}/rooms/{}/join", self.server_url, room_code);
        let body = format!(r#"{{"nickname":"{}","avatar_id":0}}"#, nickname);
        
        godot_print!("Auto-joining as host: POST {} with body: {}", url, body);
        
        if let Some(mut request) = self.base_mut().try_get_node_as::<HttpRequest>("JoinRoomRequest") {
            let mut headers = PackedStringArray::new();
            headers.push("Content-Type: application/json");
            
            request.request_ex(&url)
                .custom_headers(&headers)
                .method(Method::POST)
                .request_data(&body)
                .done();
        }
    }
    
    // Helper methods
    fn set_status(&mut self, message: &str, is_error: bool) {
        if let Some(mut label) = self.base_mut().try_get_node_as::<Label>("CenterContainer/VBoxContainer/StatusLabel") {
            label.set_text(message);
            if is_error {
                godot_warn!("Status (error): {}", message);
            } else {
                godot_print!("Status: {}", message);
            }
        }
    }
    
    fn get_room_code_input(&self) -> String {
        if let Some(input) = self.base().try_get_node_as::<LineEdit>("CenterContainer/VBoxContainer/JoinRoomContainer/RoomCodeInput") {
            input.get_text().to_string().to_uppercase()
        } else {
            String::new()
        }
    }
    
    fn set_button_enabled(&mut self, button_path: &str, enabled: bool) {
        let full_path = format!("CenterContainer/VBoxContainer/{}", button_path);
        if let Some(mut button) = self.base_mut().try_get_node_as::<Button>(&full_path) {
            button.set_disabled(!enabled);
        }
    }
}
