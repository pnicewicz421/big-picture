//! # Big Picture Game Server
//!
//! REST API server for managing game rooms, player sessions, and turn coordination.
//!
//! ## Architecture
//!
//! - Axum web framework with tokio async runtime
//! - In-memory state management (RoomManager)
//! - REST endpoints for lobby and game operations
//!
//! ## Endpoints
//!
//! - `GET /` - Health check
//! - `POST /rooms` - Create new room
//! - `POST /rooms/:code/join` - Join room
//! - `POST /rooms/:room_id/leave` - Leave room
//! - `POST /rooms/:code/rejoin` - Rejoin room
//! - `POST /rooms/:room_id/start` - Start game ("All is in!")
//! - `GET /rooms/:room_id` - Get room state

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use big_picture_domain::{
    AvatarId, JoinError, RoomError, RoomManager, RoomId, PlayerId,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Shared application state.
#[derive(Clone)]
struct AppState {
    room_manager: Arc<RwLock<RoomManager>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "big_picture_server=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Big Picture Server starting...");

    // Initialize shared state
    let state = AppState {
        room_manager: Arc::new(RwLock::new(RoomManager::new())),
    };

    // Configure CORS for cross-origin requests from Godot client
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/", get(health_check))
        .route("/rooms", post(create_room))
        .route("/rooms/:code/join", post(join_room))
        .route("/rooms/:room_id/leave", post(leave_room))
        .route("/rooms/:code/rejoin", post(rejoin_room))
        .route("/rooms/:room_id/start", post(start_game))
        .route("/rooms/:room_id", get(get_room_state))
        .layer(cors)
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    tracing::info!("Server ready at http://localhost:3000");
    
    axum::serve(listener, app).await.unwrap();
}

/// Health check endpoint.
async fn health_check() -> &'static str {
    "Big Picture Server v0.1.0"
}

// --- Request/Response DTOs ---

#[derive(Debug, Serialize, Deserialize)]
struct CreateRoomResponse {
    room_code: String,
    room_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JoinRoomRequest {
    nickname: String,
    avatar_id: u8,
}

#[derive(Debug, Serialize, Deserialize)]
struct JoinRoomResponse {
    player_id: String,
    room_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LeaveRoomRequest {
    player_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RejoinRoomRequest {
    nickname: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RejoinRoomResponse {
    player_id: String,
    room_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RoomStateResponse {
    room_id: String,
    room_code: String,
    state: String,
    player_count: usize,
    players: Vec<PlayerInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerInfo {
    id: String,
    nickname: String,
    avatar_id: u8,
    connected: bool,
}

// --- Handlers ---

/// POST /rooms - Create a new room.
async fn create_room(
    State(state): State<AppState>,
) -> Result<Json<CreateRoomResponse>, AppError> {
    let mut manager = state.room_manager.write().await;
    let (room_id, code) = manager.create_room();
    
    tracing::info!("Created room {} with code {}", room_id, code);
    
    Ok(Json(CreateRoomResponse {
        room_code: code,
        room_id: room_id.to_string(),
    }))
}

/// POST /rooms/:code/join - Join a room by code.
async fn join_room(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(req): Json<JoinRoomRequest>,
) -> Result<Json<JoinRoomResponse>, AppError> {
    let mut manager = state.room_manager.write().await;
    
    let avatar = AvatarId::new(req.avatar_id);
    let (room_id, player_id) = manager
        .join_room(&code, req.nickname.clone(), avatar)
        .map_err(AppError::from)?;
    
    tracing::info!(
        "Player {} ({}) joined room {} (code: {})",
        req.nickname,
        player_id,
        room_id,
        code
    );
    
    Ok(Json(JoinRoomResponse {
        player_id: player_id.to_string(),
        room_id: room_id.to_string(),
    }))
}

/// POST /rooms/:room_id/leave - Leave a room.
async fn leave_room(
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
    Json(req): Json<LeaveRoomRequest>,
) -> Result<StatusCode, AppError> {
    let mut manager = state.room_manager.write().await;
    
    let room_id = RoomId::from_string(&room_id_str)
        .map_err(|_| AppError::InvalidRoomId)?;
    let player_id = PlayerId::from_string(&req.player_id)
        .map_err(|_| AppError::InvalidPlayerId)?;
    
    manager
        .leave_room(room_id, player_id)
        .map_err(AppError::from)?;
    
    tracing::info!("Player {} left room {}", req.player_id, room_id);
    
    Ok(StatusCode::OK)
}

/// POST /rooms/:code/rejoin - Rejoin a room by nickname.
async fn rejoin_room(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(req): Json<RejoinRoomRequest>,
) -> Result<Json<RejoinRoomResponse>, AppError> {
    let mut manager = state.room_manager.write().await;
    
    let (room_id, player_id) = manager
        .rejoin_room(&code, &req.nickname)
        .map_err(AppError::from)?;
    
    tracing::info!(
        "Player {} rejoined room {} (code: {})",
        req.nickname,
        room_id,
        code
    );
    
    Ok(Json(RejoinRoomResponse {
        player_id: player_id.to_string(),
        room_id: room_id.to_string(),
    }))
}

/// POST /rooms/:room_id/start - Start the game (placeholder).
async fn start_game(
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
) -> Result<StatusCode, AppError> {
    let manager = state.room_manager.read().await;
    let room_id = RoomId::from_string(&room_id_str)
        .map_err(|_| AppError::InvalidRoomId)?;
    
    let room = manager
        .get_room(&room_id)
        .ok_or(RoomError::RoomNotFound)?;
    
    let player_count = room.player_count();
    
    if !(2..=8).contains(&player_count) {
        return Err(AppError::InvalidPlayerCount(player_count));
    }
    
    tracing::info!("Starting game in room {} with {} players", room_id, player_count);
    
    // TODO: Actually start the game (future task)
    Ok(StatusCode::OK)
}

/// GET /rooms/:room_id - Get current room state.
async fn get_room_state(
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
) -> Result<Json<RoomStateResponse>, AppError> {
    let manager = state.room_manager.read().await;
    let room_id = RoomId::from_string(&room_id_str)
        .map_err(|_| AppError::InvalidRoomId)?;
    
    let room = manager
        .get_room(&room_id)
        .ok_or(RoomError::RoomNotFound)?;
    
    let players: Vec<PlayerInfo> = room
        .players
        .iter()
        .map(|p| PlayerInfo {
            id: p.id.to_string(),
            nickname: p.nickname.clone(),
            avatar_id: p.avatar_id.as_u8(),
            connected: p.connected,
        })
        .collect();
    
    Ok(Json(RoomStateResponse {
        room_id: room_id.to_string(),
        room_code: room.code.clone(),
        state: format!("{:?}", room.state),
        player_count: room.player_count(),
        players,
    }))
}

// --- Error Handling ---

#[derive(Debug)]
enum AppError {
    Room(RoomError),
    Join(JoinError),
    InvalidPlayerCount(usize),
    InvalidRoomId,
    InvalidPlayerId,
}

impl From<RoomError> for AppError {
    fn from(err: RoomError) -> Self {
        AppError::Room(err)
    }
}

impl From<JoinError> for AppError {
    fn from(err: JoinError) -> Self {
        AppError::Join(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Room(RoomError::RoomNotFound) | AppError::Room(RoomError::NotFound(_)) => {
                (StatusCode::NOT_FOUND, "Room not found".to_string())
            }
            AppError::Room(RoomError::RoomFull) | AppError::Room(RoomError::Full(_)) => {
                (StatusCode::CONFLICT, "Room is full".to_string())
            }
            AppError::Room(RoomError::PlayerNotFoundSimple) | AppError::Room(RoomError::PlayerNotFound(_, _)) => {
                (StatusCode::NOT_FOUND, "Player not found".to_string())
            }
            AppError::Room(RoomError::GameAlreadyStarted) | AppError::Room(RoomError::AlreadyStarted(_)) => {
                (StatusCode::CONFLICT, "Game already started".to_string())
            }
            AppError::Room(RoomError::NicknameTaken(_, _)) => {
                (StatusCode::CONFLICT, "Nickname already taken".to_string())
            }
            AppError::Room(RoomError::InvalidCode(code)) => {
                (StatusCode::NOT_FOUND, format!("Invalid room code: {}", code))
            }
            AppError::Room(RoomError::Internal(msg)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            AppError::Join(JoinError::DuplicateNickname) => {
                (StatusCode::CONFLICT, "Nickname already taken".to_string())
            }
            AppError::Join(JoinError::RoomFull) => {
                (StatusCode::CONFLICT, "Room is full".to_string())
            }
            AppError::Join(JoinError::GameInProgress) => {
                (StatusCode::CONFLICT, "Game already in progress".to_string())
            }
            AppError::Join(JoinError::RoomNotFound) => {
                (StatusCode::NOT_FOUND, "Room not found".to_string())
            }
            AppError::Join(JoinError::InvalidNickname) => {
                (StatusCode::BAD_REQUEST, "Invalid nickname".to_string())
            }
            AppError::InvalidPlayerCount(count) => {
                (StatusCode::BAD_REQUEST, format!("Invalid player count: {} (need 2-8)", count))
            }
            AppError::InvalidRoomId => {
                (StatusCode::BAD_REQUEST, "Invalid room ID".to_string())
            }
            AppError::InvalidPlayerId => {
                (StatusCode::BAD_REQUEST, "Invalid player ID".to_string())
            }
        };
        
        (status, message).into_response()
    }
}
