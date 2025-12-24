//! Network communication module for backend API calls.

use serde::{Deserialize, Serialize};

/// Base URL for the backend server.
const SERVER_URL: &str = "http://localhost:3000";

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRoomResponse {
    pub room_code: String,
    pub room_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomRequest {
    pub nickname: String,
    pub avatar_id: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomResponse {
    pub player_id: String,
    pub room_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: String,
    pub nickname: String,
    pub avatar_id: u8,
    pub connected: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomStateResponse {
    pub room_id: String,
    pub room_code: String,
    pub state: String,
    pub player_count: usize,
    pub players: Vec<PlayerInfo>,
}

/// Create a new room on the server.
pub async fn create_room() -> Result<CreateRoomResponse, String> {
    let url = format!("{}/rooms", SERVER_URL);
    
    let response = reqwest::Client::new()
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Server error: {}", response.status()));
    }
    
    response
        .json::<CreateRoomResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Join a room by code.
pub async fn join_room(code: &str, nickname: String, avatar_id: u8) -> Result<JoinRoomResponse, String> {
    let url = format!("{}/rooms/{}/join", SERVER_URL, code);
    
    let request = JoinRoomRequest { nickname, avatar_id };
    
    let response = reqwest::Client::new()
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Server error {}: {}", status, body));
    }
    
    response
        .json::<JoinRoomResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Get the current state of a room.
pub async fn get_room_state(room_id: &str) -> Result<RoomStateResponse, String> {
    let url = format!("{}/rooms/{}", SERVER_URL, room_id);
    
    let response = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Server error: {}", response.status()));
    }
    
    response
        .json::<RoomStateResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// Start the game in a room.
pub async fn start_game(room_id: &str) -> Result<(), String> {
    let url = format!("{}/rooms/{}/start", SERVER_URL, room_id);
    
    let response = reqwest::Client::new()
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Server error: {}", body));
    }
    
    Ok(())
}
