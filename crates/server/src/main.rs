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
    response::{Html, IntoResponse},
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
        .route("/rooms/:room_id/next", post(next_stage))
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
async fn health_check() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Big Picture - Game Lobby</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background-color: #1a1a2e;
            color: #e94560;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            text-align: center;
        }
        .container {
            background-color: #16213e;
            padding: 2.5rem;
            border-radius: 15px;
            box-shadow: 0 10px 30px rgba(0,0,0,0.5);
            max-width: 500px;
            width: 90%;
        }
        h1 {
            font-size: 3rem;
            margin-bottom: 0.5rem;
            text-transform: uppercase;
            letter-spacing: 3px;
            color: #e94560;
        }
        p {
            color: #4ecca3;
            font-size: 1.1rem;
            margin-bottom: 2rem;
        }
        .form-group {
            margin-bottom: 1.5rem;
            text-align: left;
        }
        label {
            display: block;
            margin-bottom: 0.5rem;
            color: #fff;
            font-size: 0.9rem;
        }
        input {
            width: 100%;
            padding: 0.8rem;
            border-radius: 8px;
            border: 2px solid #0f3460;
            background-color: #1a1a2e;
            color: #fff;
            font-size: 1rem;
            box-sizing: border-box;
            margin-bottom: 1rem;
        }
        input:focus {
            outline: none;
            border-color: #e94560;
        }
        .actions {
            display: flex;
            flex-direction: column;
            gap: 1rem;
            margin-top: 1rem;
        }
        button {
            padding: 1rem;
            border: none;
            border-radius: 8px;
            font-weight: bold;
            cursor: pointer;
            transition: transform 0.1s, background-color 0.2s;
            text-transform: uppercase;
        }
        button:active {
            transform: scale(0.98);
        }
        .btn-primary {
            background-color: #e94560;
            color: #fff;
        }
        .btn-primary:hover {
            background-color: #ff4d6d;
        }
        .btn-secondary {
            background-color: #4ecca3;
            color: #1a1a2e;
        }
        .btn-secondary:hover {
            background-color: #45b393;
        }
        .btn-outline {
            background-color: transparent;
            border: 2px solid #0f3460;
            color: #fff;
        }
        .btn-outline:hover {
            border-color: #e94560;
        }
        .btn-quit {
            background-color: #533483;
            color: #fff;
            margin-top: 2rem;
        }
        #player-list {
            list-style: none;
            padding: 0;
            margin: 2rem 0;
            text-align: left;
        }
        #player-list li {
            background-color: #1a1a2e;
            padding: 0.8rem 1.2rem;
            margin-bottom: 0.5rem;
            border-radius: 8px;
            display: flex;
            justify-content: space-between;
            align-items: center;
            border-left: 4px solid #4ecca3;
        }
        .room-code-display {
            font-size: 2.5rem;
            font-weight: bold;
            color: #4ecca3;
            letter-spacing: 5px;
            margin: 1rem 0;
            background: #1a1a2e;
            padding: 1rem;
            border-radius: 8px;
        }
        .goal-display {
            background: #1a1a2e;
            padding: 1.5rem;
            border-radius: 12px;
            border: 2px solid #4ecca3;
            font-size: 1.2rem;
            margin: 1rem 0;
            line-height: 1.6;
        }
        .starting-object-box {
            background: #16213e;
            padding: 1rem;
            border-radius: 8px;
            margin: 1rem 0;
            border-left: 4px solid #4ecca3;
        }
        #result {
            margin-top: 1rem;
            padding: 0.8rem;
            border-radius: 8px;
            display: none;
            font-size: 0.9rem;
        }
        .success {
            background-color: rgba(78, 204, 163, 0.2);
            color: #4ecca3;
            border: 1px solid #4ecca3;
        }
        .error {
            background-color: rgba(233, 69, 96, 0.2);
            color: #e94560;
            border: 1px solid #e94560;
        }
        .version {
            font-size: 0.8rem;
            color: #533483;
            margin-top: 2rem;
        }
        .hidden { display: none !important; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Big Picture</h1>
        
        <!-- Selection View -->
        <div id="view-selection">
            <p>Ready to play?</p>
            <div class="actions">
                <button class="btn-primary" onclick="showView('create')">New Room</button>
                <button class="btn-secondary" onclick="showView('join')">Join Room</button>
            </div>
        </div>

        <!-- Create View -->
        <div id="view-create" class="hidden">
            <p>Create a new room</p>
            <div class="form-group">
                <label for="create-nickname">Your Nickname</label>
                <input type="text" id="create-nickname" placeholder="Enter nickname..." maxlength="20">
            </div>
            <div class="actions">
                <button class="btn-primary" onclick="createAndJoin()">Create & Join</button>
                <button class="btn-outline" onclick="showView('selection')">Back</button>
            </div>
        </div>

        <!-- Join View -->
        <div id="view-join" class="hidden">
            <p>Join an existing room</p>
            <div class="form-group">
                <label for="join-code">Room Code</label>
                <input type="text" id="join-code" placeholder="e.g. ABC123" maxlength="6" style="text-transform: uppercase;">
                <label for="join-nickname">Your Nickname</label>
                <input type="text" id="join-nickname" placeholder="Enter nickname..." maxlength="20">
            </div>
            <div class="actions">
                <button class="btn-secondary" onclick="joinExisting()">Join Game</button>
                <button class="btn-outline" onclick="showView('selection')">Back</button>
            </div>
        </div>

        <!-- Lobby View -->
        <div id="view-lobby" class="hidden">
            <p>Waiting for players...</p>
            <div class="room-code-display" id="display-code">------</div>
            <ul id="player-list"></ul>
            <div class="actions">
                <button id="btn-start-game" class="btn-primary hidden" onclick="startGame()">Start Game</button>
                <button class="btn-quit" onclick="quitRoom()">Quit Room</button>
            </div>
        </div>

        <!-- Game View -->
        <div id="view-game" class="hidden">
            <div id="game-reveal" class="hidden">
                <h2>Communal Goal</h2>
                <div class="goal-display" id="display-goal">...</div>
                <div class="starting-object-box">
                    <h3>Your Starting Object:</h3>
                    <div id="display-starting-object" style="font-size: 1.5rem; color: #4ecca3; margin: 1rem 0;">...</div>
                </div>
                <div class="actions">
                    <button id="btn-next-stage" class="btn-primary hidden" onclick="nextStage()">Continue to Turns</button>
                </div>
            </div>
            <div id="game-turn" class="hidden">
                <p>Game in progress...</p>
            </div>
            <div class="actions">
                <button class="btn-quit" onclick="quitRoom()">Quit Game</button>
            </div>
        </div>

        <div id="result"></div>
        <div class="version">Server v0.1.0</div>
    </div>

    <script>
        let currentRoom = null; // { room_id, room_code, player_id, nickname }
        let pollInterval = null;

        function showView(viewName) {
            document.getElementById('view-selection').classList.add('hidden');
            document.getElementById('view-create').classList.add('hidden');
            document.getElementById('view-join').classList.add('hidden');
            document.getElementById('view-lobby').classList.add('hidden');
            document.getElementById('view-game').classList.add('hidden');
            document.getElementById('result').style.display = 'none';

            document.getElementById('view-' + viewName).classList.remove('hidden');
        }

        function showResult(message, isError = false) {
            const resultDiv = document.getElementById('result');
            resultDiv.style.display = 'block';
            resultDiv.textContent = message;
            resultDiv.className = isError ? 'error' : 'success';
        }

        async function createAndJoin() {
            const nickname = document.getElementById('create-nickname').value.trim();
            if (!nickname) {
                showResult('Please enter a nickname', true);
                return;
            }

            try {
                // 1. Create Room
                const createRes = await fetch('/rooms', { method: 'POST' });
                const createData = await createRes.json();
                if (!createRes.ok) throw new Error(createData.message || 'Failed to create room');

                // 2. Join Room
                const joinRes = await fetch(`/rooms/${createData.room_code}/join`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ nickname, avatar_id: 0 })
                });
                const joinData = await joinRes.json();
                if (!joinRes.ok) throw new Error(joinData.message || 'Failed to join room');

                currentRoom = {
                    room_id: createData.room_id,
                    room_code: createData.room_code,
                    player_id: joinData.player_id,
                    nickname: nickname
                };

                enterLobby();
            } catch (err) {
                showResult(err.message, true);
            }
        }

        async function joinExisting() {
            const code = document.getElementById('join-code').value.trim().toUpperCase();
            const nickname = document.getElementById('join-nickname').value.trim();

            if (!code || !nickname) {
                showResult('Please enter both code and nickname', true);
                return;
            }

            try {
                const response = await fetch(`/rooms/${code}/join`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ nickname, avatar_id: 0 })
                });
                const data = await response.json();
                
                if (response.ok) {
                    currentRoom = {
                        room_id: data.room_id,
                        room_code: code,
                        player_id: data.player_id,
                        nickname: nickname
                    };
                    enterLobby();
                } else {
                    showResult(data.message || 'Room not found or full', true);
                }
            } catch (err) {
                showResult('Network error', true);
            }
        }

        function enterLobby() {
            document.getElementById('display-code').textContent = currentRoom.room_code;
            showView('lobby');
            updatePlayerList();
            pollInterval = setInterval(updatePlayerList, 2000);
        }

        async function updatePlayerList() {
            if (!currentRoom) return;
            try {
                const res = await fetch(`/rooms/${currentRoom.room_id}`);
                if (!res.ok) {
                    if (res.status === 404) {
                        showResult('Room was closed by host', true);
                        setTimeout(quitRoom, 2000);
                    }
                    return;
                }
                const data = await res.json();
                
                // Check if we are the host (first player in list)
                const isHost = data.players.length > 0 && data.players[0].id === currentRoom.player_id;
                const startBtn = document.getElementById('btn-start-game');
                
                if (isHost) {
                    startBtn.classList.remove('hidden');
                    startBtn.disabled = data.players.length < 2;
                    startBtn.title = data.players.length < 2 ? 'Need at least 2 players' : '';
                } else {
                    startBtn.classList.add('hidden');
                }

                const list = document.getElementById('player-list');
                list.innerHTML = data.players.map((p, index) => `
                    <li>
                        <span>
                            ${index === 0 ? '👑 ' : ''}${p.nickname} 
                            ${p.id === currentRoom.player_id ? '(You)' : ''}
                        </span>
                        <span style="color: ${p.connected ? '#4ecca3' : '#e94560'}">
                            ${p.connected ? '● Online' : '○ Offline'}
                        </span>
                    </li>
                `).join('');

                if (data.state === 'InGame' && data.game) {
                    showView('game');
                    const isHost = data.players.length > 0 && data.players[0].id === currentRoom.player_id;
                    
                    if (data.game.stage === 'RevealGoal') {
                        document.getElementById('game-reveal').classList.remove('hidden');
                        document.getElementById('game-turn').classList.add('hidden');
                        document.getElementById('display-goal').textContent = data.game.communal_goal;
                        
                        const me = data.players.find(p => p.id === currentRoom.player_id);
                        const myObj = me ? me.starting_object : null;
                        document.getElementById('display-starting-object').textContent = myObj || 'Waiting...';
                        
                        const nextBtn = document.getElementById('btn-next-stage');
                        if (isHost) {
                            nextBtn.classList.remove('hidden');
                        } else {
                            nextBtn.classList.add('hidden');
                        }
                    } else if (data.game.stage === 'PlayerTurn') {
                        document.getElementById('game-reveal').classList.add('hidden');
                        document.getElementById('game-turn').classList.remove('hidden');
                        // Future: show turn UI
                    }
                }
            } catch (err) {
                console.error('Polling error', err);
            }
        }

        async function startGame() {
            if (!currentRoom) return;
            try {
                const res = await fetch(`/rooms/${currentRoom.room_id}/start`, { method: 'POST' });
                if (res.ok) {
                    showResult('Game started!');
                } else {
                    const text = await res.text();
                    showResult(text || 'Failed to start game', true);
                }
            } catch (err) {
                showResult('Network error', true);
            }
        }

        async function nextStage() {
            if (!currentRoom) return;
            try {
                const res = await fetch(`/rooms/${currentRoom.room_id}/next`, { method: 'POST' });
                if (!res.ok) {
                    const text = await res.text();
                    showResult(text || 'Failed to advance stage', true);
                }
            } catch (err) {
                showResult('Network error', true);
            }
        }

        async function quitRoom() {
            if (currentRoom) {
                try {
                    await fetch(`/rooms/${currentRoom.room_id}/leave`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ player_id: currentRoom.player_id })
                    });
                } catch (err) {
                    console.error('Error leaving room', err);
                }
            }
            
            clearInterval(pollInterval);
            currentRoom = null;
            showView('selection');
        }
    </script>
</body>
</html>
"#)
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
    game: Option<GameInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameInfo {
    stage: String,
    communal_goal: String,
    player_starting_objects: std::collections::HashMap<String, String>,
    current_image_id: String,
    goal_image_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlayerInfo {
    id: String,
    nickname: String,
    avatar_id: u8,
    connected: bool,
    starting_object: Option<String>,
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
    
    // Drop the read lock before getting a write lock
    drop(manager);
    
    let mut manager = state.room_manager.write().await;
    manager.start_game(&room_id)?;
    
    Ok(StatusCode::OK)
}

/// POST /rooms/:room_id/next - Transition to the next game stage.
async fn next_stage(
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut manager = state.room_manager.write().await;
    let room_id = RoomId::from_string(&room_id_str)
        .map_err(|_| AppError::InvalidRoomId)?;
    
    let room = manager
        .get_room_mut(&room_id)
        .ok_or(RoomError::RoomNotFound)?;
    
    if let Some(game) = &mut room.game {
        game.next_stage();
        Ok(StatusCode::OK)
    } else {
        Err(RoomError::Internal("Game not started".to_string()).into())
    }
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
        .map(|p| {
            let starting_object = room.game.as_ref().and_then(|g| g.player_starting_objects.get(&p.id).cloned());
            PlayerInfo {
                id: p.id.to_string(),
                nickname: p.nickname.clone(),
                avatar_id: p.avatar_id.as_u8(),
                connected: p.connected,
                starting_object,
            }
        })
        .collect();
    
    let game = room.game.as_ref().map(|g| GameInfo {
        stage: format!("{:?}", g.stage),
        communal_goal: g.communal_goal.clone(),
        player_starting_objects: g.player_starting_objects.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
        current_image_id: g.current_image.as_str().to_string(),
        goal_image_id: g.goal_image.as_str().to_string(),
    });

    Ok(Json(RoomStateResponse {
        room_id: room_id.to_string(),
        room_code: room.code.clone(),
        state: format!("{:?}", room.state),
        player_count: room.player_count(),
        players,
        game,
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
            AppError::Room(RoomError::NotEnoughPlayers(_)) => {
                (StatusCode::BAD_REQUEST, "Not enough players to start the game".to_string())
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
