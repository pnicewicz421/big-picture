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
        .route("/rooms/:room_id/action", post(submit_action))
        .route("/rooms/:room_id/votes", post(submit_votes))
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
        .option-btn {
            display: block;
            width: 100%;
            padding: 1rem;
            margin: 0.5rem 0;
            background: #16213e;
            border: 2px solid #4ecca3;
            color: #fff;
            border-radius: 8px;
            cursor: pointer;
            font-size: 1rem;
            transition: all 0.2s;
        }
        .option-btn:hover {
            background: #4ecca3;
            color: #1a1a2e;
        }
        .voting-controls button {
            background: none;
            border: 1px solid #4ecca3;
            color: #4ecca3;
            cursor: pointer;
            padding: 0.2rem 0.5rem;
            margin: 0 0.1rem;
            border-radius: 4px;
        }
        .voting-controls button:hover {
            background: #4ecca3;
            color: #1a1a2e;
        }
        .voting-controls button.selected {
            background: #4ecca3;
            color: #1a1a2e;
        }
        .podium-place {
            font-size: 1.5rem;
            font-weight: bold;
            margin: 1rem 0;
            padding: 1rem;
            border-radius: 10px;
            opacity: 0;
            transform: translateY(20px);
            transition: all 0.5s ease-out;
        }
        .podium-1 { background: gold; color: #000; border: 4px solid #fff; font-size: 2rem; }
        .podium-2 { background: silver; color: #000; }
        .podium-3 { background: #cd7f32; color: #000; }
        .visible { opacity: 1; transform: translateY(0); }
    </style>
</head>
<body>
    <div class="container">
        <h1>Big Picture</h1>
        
        <!-- Selection View -->
        <div id="view-selection">
            <p>Ready to play?</p>
            <div class="actions">
                <button class="btn-primary" onclick="createTV()">Host Game (TV)</button>
                <button class="btn-secondary" onclick="showView('join')">Join Game (Player)</button>
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
                <button class="btn-secondary" onclick="joinPlayer()">Join Game</button>
                <button class="btn-outline" onclick="showView('selection')">Back</button>
            </div>
        </div>

        <!-- Lobby View -->
        <div id="view-lobby" class="hidden">
            <p id="lobby-status">Waiting for players...</p>
            <div class="room-code-display" id="display-code">------</div>
            <ul id="player-list"></ul>
            <div class="actions">
                <button id="btn-start-game" class="btn-primary hidden" onclick="startGame()">Start Game</button>
                <button class="btn-quit" onclick="quitRoom()">Quit Room</button>
            </div>
        </div>

        <!-- Game View -->
        <div id="view-game" class="hidden">
            <!-- Persistent Goal -->
            <div id="game-header" style="margin-bottom: 1rem; border-bottom: 1px solid #4ecca3; padding-bottom: 1rem;">
                <small style="color: #888; text-transform: uppercase; letter-spacing: 1px;">Communal Goal</small>
                <div class="goal-display" id="display-goal" style="margin: 0.5rem 0; padding: 1rem; font-size: 1.1rem;">...</div>
            </div>

            <!-- Reveal Stage -->
            <div id="stage-reveal" class="hidden">
                
                <!-- TV Only -->
                <div id="tv-reveal-info" class="hidden">
                    <p>Players are checking their starting objects...</p>
                    <p id="tv-reveal-timer" style="font-size: 1.5rem; color: #e94560; font-weight: bold; margin: 1rem 0;"></p>
                    <button class="btn-primary" onclick="nextStage()">Start Turns Now</button>
                </div>

                <!-- Player Only -->
                <div id="player-reveal-info" class="hidden">
                    <div class="starting-object-box">
                        <h3>Your Starting Object:</h3>
                        <div id="display-starting-object" style="font-size: 1.5rem; color: #4ecca3; margin: 1rem 0;">...</div>
                    </div>
                    <p>Look at the TV for the goal!</p>
                </div>
            </div>

            <!-- Turn Stage -->
            <div id="stage-turn" class="hidden">
                <h3 id="turn-status">Round 1</h3>
                <div id="turn-timer" style="font-size: 2rem; color: #e94560; font-weight: bold;">10</div>
                
                <!-- TV Only -->
                <div id="tv-turn-info" class="hidden">
                    <div class="starting-object-box">
                        <h3>Current Player: <span id="tv-current-player">...</span></h3>
                        <div id="tv-current-object" style="font-size: 1.5rem; color: #4ecca3;">...</div>
                    </div>
                </div>

                <!-- Player Only -->
                <div id="player-turn-info" class="hidden">
                    <div id="my-turn-ui" class="hidden">
                        <h3>It's YOUR Turn!</h3>
                        <p>Choose an option to modify your object:</p>
                        <div id="turn-options" class="actions" style="flex-direction: column; gap: 0.5rem;"></div>
                    </div>
                    <div id="others-turn-ui" class="hidden">
                        <p>Waiting for <span id="other-player-name">...</span> to move...</p>
                    </div>
                </div>
            </div>

            <!-- Voting Stage -->
            <div id="stage-voting" class="hidden">
                <h2>Vote for the Best!</h2>
                <p>Rate how well each player matched the goal.</p>
                <ul id="voting-list" style="list-style: none; padding: 0;"></ul>
                <div id="voting-status" class="hidden" style="margin-top: 1rem; color: #4ecca3;">Waiting for others...</div>
                <button id="btn-submit-votes" class="btn-primary" onclick="submitAllVotes()">Submit Votes</button>
            </div>

            <!-- Results Stage -->
            <div id="stage-results" class="hidden">
                <h2>Final Results</h2>
                <div id="podium-container"></div>
                <ul id="final-results-list" style="list-style: none; padding: 0; margin-top: 2rem;"></ul>
                <div class="actions">
                    <button class="btn-quit" onclick="quitRoom()">Quit Game</button>
                </div>
            </div>
        </div>

        <div id="result"></div>
        <div id="debug-info" style="font-size: 0.7rem; color: #666; margin-top: 1rem; display: none;"></div>
        <div class="version">Server v0.2.1</div>
    </div>

    <script>
        let currentRoom = null; // { room_id, room_code, player_id, nickname, isTV }
        let pollInterval = null;
        let timerInterval = null;

        function updateDebugInfo(data) {
            const debugEl = document.getElementById('debug-info');
            if (!currentRoom) return;
            
            debugEl.style.display = 'block';
            debugEl.innerHTML = `
                Room: ${currentRoom.room_code} | ID: ${currentRoom.player_id}<br>
                State: ${data.state} | Stage: ${data.game ? data.game.stage : 'N/A'}<br>
                Turn: ${data.game ? data.game.current_turn_player_id : 'N/A'}<br>
                My Turn: ${data.game && data.game.current_turn_player_id === currentRoom.player_id}
            `;
        }

        function showView(viewName) {
            ['selection', 'create', 'join', 'lobby', 'game'].forEach(v => {
                const el = document.getElementById('view-' + v);
                if (el) el.classList.add('hidden');
            });
            document.getElementById('result').style.display = 'none';
            
            const target = document.getElementById('view-' + viewName);
            if (target) target.classList.remove('hidden');
        }

        function showResult(message, isError = false) {
            const resultDiv = document.getElementById('result');
            resultDiv.style.display = 'block';
            resultDiv.textContent = message;
            resultDiv.className = isError ? 'error' : 'success';
        }

        async function createTV() {
            try {
                const createRes = await fetch('/rooms', { method: 'POST' });
                const createData = await createRes.json();
                if (!createRes.ok) throw new Error(createData.message || 'Failed to create room');

                currentRoom = {
                    room_id: createData.room_id,
                    room_code: createData.room_code,
                    isTV: true
                };

                enterLobby();
            } catch (err) {
                showResult(err.message, true);
            }
        }

        async function joinPlayer() {
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
                        nickname: nickname,
                        isTV: false
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
            
            if (currentRoom.isTV) {
                document.getElementById('btn-start-game').classList.remove('hidden');
                document.getElementById('lobby-status').textContent = "Waiting for players to join...";
            } else {
                document.getElementById('btn-start-game').classList.add('hidden');
                document.getElementById('lobby-status').textContent = "Waiting for TV to start game...";
            }

            updateGameState();
            pollInterval = setInterval(updateGameState, 1000);
        }

        async function updateGameState() {
            if (!currentRoom) return;
            try {
                const res = await fetch(`/rooms/${currentRoom.room_id}`);
                if (!res.ok) {
                    if (res.status === 404) {
                        showResult('Room closed', true);
                        setTimeout(quitRoom, 2000);
                    }
                    return;
                }
                const data = await res.json();
                
                // Update Player List
                const list = document.getElementById('player-list');
                list.innerHTML = data.players.map((p, index) => `
                    <li>
                        <span>${p.nickname}</span>
                        <span style="color: ${p.connected ? '#4ecca3' : '#e94560'}">
                            ${p.connected ? '●' : '○'}
                        </span>
                    </li>
                `).join('');

                // Enable Start Button for TV if enough players
                if (currentRoom.isTV) {
                    const startBtn = document.getElementById('btn-start-game');
                    startBtn.disabled = data.players.length < 2;
                }

                // Game State Handling
                if (data.state === 'InGame' && data.game) {
                    showView('game');
                    updateGameView(data);
                }
                
                updateDebugInfo(data);
            } catch (err) {
                console.error('Polling error', err);
            }
        }

        function updateGameView(data) {
            const game = data.game;
            const isTV = currentRoom.isTV;

            // Always update goal
            document.getElementById('display-goal').textContent = game.communal_goal;

            // Hide all stages first
            ['reveal', 'turn', 'voting', 'results'].forEach(s => {
                const el = document.getElementById('stage-' + s);
                if (el) el.classList.add('hidden');
            });

            if (game.stage === 'RevealGoal') {
                document.getElementById('stage-reveal').classList.remove('hidden');
                
                if (isTV) {
                    document.getElementById('tv-reveal-info').classList.remove('hidden');
                    document.getElementById('player-reveal-info').classList.add('hidden');

                    // Auto-advance logic
                    if (game.stage_start_time) {
                        const elapsed = Math.floor(Date.now() / 1000) - game.stage_start_time;
                        const remaining = Math.max(0, 10 - elapsed);
                        document.getElementById('tv-reveal-timer').textContent = `Starting in ${remaining}...`;
                        
                        if (remaining === 0) {
                            nextStage();
                        }
                    }
                } else {
                    document.getElementById('tv-reveal-info').classList.add('hidden');
                    document.getElementById('player-reveal-info').classList.remove('hidden');
                    
                    const me = data.players.find(p => p.id === currentRoom.player_id);
                    const myObj = me ? me.starting_object : null;
                    document.getElementById('display-starting-object').textContent = myObj || 'Loading...';
                }
            } else if (game.stage === 'PlayerTurn') {
                document.getElementById('stage-turn').classList.remove('hidden');
                document.getElementById('turn-status').textContent = `Round ${game.current_round + 1}`;
                
                // Timer logic
                if (game.turn_start_time) {
                    const elapsed = Math.floor(Date.now() / 1000) - game.turn_start_time;
                    const remaining = Math.max(0, 10 - elapsed);
                    document.getElementById('turn-timer').textContent = remaining;
                }

                const currentPlayerId = game.current_turn_player_id;
                const currentPlayer = data.players.find(p => p.id === currentPlayerId);
                const currentPlayerName = currentPlayer ? currentPlayer.nickname : 'Unknown';

                if (isTV) {
                    document.getElementById('tv-turn-info').classList.remove('hidden');
                    document.getElementById('player-turn-info').classList.add('hidden');
                    
                    document.getElementById('tv-current-player').textContent = currentPlayerName;
                    const currentObj = game.player_current_objects[currentPlayerId];
                    document.getElementById('tv-current-object').textContent = currentObj || '...';
                } else {
                    document.getElementById('tv-turn-info').classList.add('hidden');
                    document.getElementById('player-turn-info').classList.remove('hidden');
                    
                    const isMyTurn = currentPlayerId === currentRoom.player_id;
                    
                    if (isMyTurn) {
                        document.getElementById('my-turn-ui').classList.remove('hidden');
                        document.getElementById('others-turn-ui').classList.add('hidden');
                        
                        // Render options
                        const optionsDiv = document.getElementById('turn-options');
                        const turnKey = `${currentPlayerId}-${game.current_round}`;
                        if (optionsDiv.innerHTML === '' || optionsDiv.dataset.turn !== turnKey) {
                            optionsDiv.dataset.turn = turnKey;
                            optionsDiv.innerHTML = game.current_options.map((opt, idx) => `
                                <button class="option-btn" onclick="submitAction(${idx})">${opt}</button>
                            `).join('');
                        }
                    } else {
                        document.getElementById('my-turn-ui').classList.add('hidden');
                        document.getElementById('others-turn-ui').classList.remove('hidden');
                        document.getElementById('other-player-name').textContent = currentPlayerName;
                    }
                }
            } else if (game.stage === 'Voting') {
                document.getElementById('stage-voting').classList.remove('hidden');
                renderVotingList(data);
            } else if (game.stage === 'Results') {
                document.getElementById('stage-results').classList.remove('hidden');
                renderResults(data);
            }
        }

        let hasVoted = false;
        let resultsShown = false;

        function renderVotingList(data) {
            const game = data.game;
            const myId = currentRoom.player_id;
            const isTV = currentRoom.isTV;

            // TV just shows status
            if (isTV) {
                document.getElementById('voting-list').innerHTML = '<p>Players are voting...</p>';
                document.getElementById('btn-submit-votes').classList.add('hidden');
                
                // Show who has voted
                const votedCount = game.players_who_voted ? game.players_who_voted.length : 0;
                const totalPlayers = data.players.length;
                document.getElementById('voting-status').textContent = `${votedCount}/${totalPlayers} players have voted.`;
                document.getElementById('voting-status').classList.remove('hidden');
                return;
            }

            // Check if I have already voted
            if (game.players_who_voted && game.players_who_voted.includes(myId)) {
                document.getElementById('voting-list').innerHTML = '';
                document.getElementById('btn-submit-votes').classList.add('hidden');
                document.getElementById('voting-status').classList.remove('hidden');
                document.getElementById('voting-status').textContent = "Votes submitted! Waiting for others...";
                return;
            }

            // Only render the list once to avoid resetting selections
            const list = document.getElementById('voting-list');
            if (list.children.length > 0) return;

            document.getElementById('voting-status').classList.add('hidden');
            document.getElementById('btn-submit-votes').classList.remove('hidden');

            const otherPlayers = data.players.filter(p => p.id !== myId);
            
            list.innerHTML = otherPlayers.map(p => {
                const finalObj = game.player_current_objects[p.id] || 'Unknown Object';
                return `
                <li class="voting-item" data-player-id="${p.id}" style="background: #16213e; padding: 1rem; margin-bottom: 1rem; border-radius: 8px;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
                        <span style="font-weight: bold; color: #fff;">${p.nickname}</span>
                        <span style="color: #4ecca3;">${finalObj}</span>
                    </div>
                    <div class="star-rating" style="display: flex; justify-content: center; gap: 0.5rem;">
                        ${[1, 2, 3, 4, 5].map(i => `
                            <button class="star-btn" onclick="ratePlayer('${p.id}', ${i}, this)" style="background: none; border: 1px solid #4ecca3; color: #4ecca3; width: 30px; height: 30px; border-radius: 50%; cursor: pointer;">${i}</button>
                        `).join('')}
                    </div>
                </li>
            `}).join('');
        }

        function ratePlayer(targetId, rating, btn) {
            // Visual feedback
            const parent = btn.parentElement;
            Array.from(parent.children).forEach(c => {
                c.style.background = 'none';
                c.style.color = '#4ecca3';
            });
            btn.style.background = '#4ecca3';
            btn.style.color = '#16213e';
            
            // Store rating
            parent.dataset.rating = rating;
        }

        async function submitAllVotes() {
            const list = document.getElementById('voting-list');
            const items = list.querySelectorAll('.voting-item');
            const votes = {};
            let allRated = true;

            items.forEach(item => {
                const targetId = item.dataset.playerId;
                const ratingDiv = item.querySelector('.star-rating');
                const rating = ratingDiv.dataset.rating;
                
                if (!rating) {
                    allRated = false;
                } else {
                    votes[targetId] = parseInt(rating);
                }
            });

            if (!allRated) {
                showResult('Please rate all players!', true);
                setTimeout(() => document.getElementById('result').style.display = 'none', 2000);
                return;
            }

            try {
                const res = await fetch(`/rooms/${currentRoom.room_id}/votes`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        voter_id: currentRoom.player_id,
                        votes: votes
                    })
                });

                if (res.ok) {
                    hasVoted = true;
                    document.getElementById('voting-list').innerHTML = '';
                    document.getElementById('btn-submit-votes').classList.add('hidden');
                    document.getElementById('voting-status').classList.remove('hidden');
                    document.getElementById('voting-status').textContent = "Votes submitted! Waiting for others...";
                } else {
                    const err = await res.json();
                    showResult(err.message || 'Failed to submit votes', true);
                }
            } catch (e) {
                showResult('Network error', true);
            }
        }

        function renderResults(data) {
            if (resultsShown) return;
            resultsShown = true;

            const scores = data.game.scores || {};
            const players = data.players.map(p => ({
                ...p,
                score: scores[p.id] || 0,
                finalObj: data.game.player_current_objects[p.id]
            }));

            // Sort by score descending
            players.sort((a, b) => b.score - a.score);

            const container = document.getElementById('podium-container');
            container.innerHTML = ''; // Clear previous

            // We want to show 3rd, then 2nd, then 1st
            // If fewer than 3 players, adjust accordingly
            const podiumOrder = [];
            if (players.length >= 3) podiumOrder.push(2); // 3rd place (index 2)
            if (players.length >= 2) podiumOrder.push(1); // 2nd place (index 1)
            if (players.length >= 1) podiumOrder.push(0); // 1st place (index 0)

            let delay = 0;
            
            podiumOrder.forEach(idx => {
                const p = players[idx];
                const rank = idx + 1;
                
                setTimeout(() => {
                    const el = document.createElement('div');
                    el.className = `podium-place podium-${rank}`;
                    el.innerHTML = `
                        <div class="rank">#${rank}</div>
                        <div class="name">${p.nickname}</div>
                        <div class="score">${p.score.toFixed(1)} pts</div>
                        <div class="obj">${p.finalObj}</div>
                    `;
                    container.appendChild(el);
                    
                    // Trigger reflow for animation
                    void el.offsetWidth;
                    el.classList.add('visible');
                }, delay);
                
                delay += 2000; // 2 seconds per reveal
            });

            // Show full list after podium
            setTimeout(() => {
                const list = document.getElementById('final-results-list');
                list.innerHTML = players.map((p, i) => `
                    <li style="padding: 0.5rem; border-bottom: 1px solid #333; display: flex; justify-content: space-between;">
                        <span>${i+1}. ${p.nickname} (${p.finalObj})</span>
                        <span>${p.score.toFixed(1)}</span>
                    </li>
                `).join('');
            }, delay + 1000);
        }

        async function startGame() {
            if (!currentRoom) return;
            await fetch(`/rooms/${currentRoom.room_id}/start`, { method: 'POST' });
        }

        async function nextStage() {
            if (!currentRoom) return;
            await fetch(`/rooms/${currentRoom.room_id}/next`, { method: 'POST' });
        }

        async function submitAction(index) {
            if (!currentRoom) return;
            try {
                await fetch(`/rooms/${currentRoom.room_id}/action`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ 
                        player_id: currentRoom.player_id,
                        option_index: index 
                    })
                });
                // Clear options to prevent double click
                document.getElementById('turn-options').innerHTML = '<p>Submitted!</p>';
            } catch (err) {
                console.error(err);
            }
        }


        async function quitRoom() {
            if (currentRoom && !currentRoom.isTV) {
                try {
                    await fetch(`/rooms/${currentRoom.room_id}/leave`, {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ player_id: currentRoom.player_id })
                    });
                } catch (err) {}
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
    player_current_objects: std::collections::HashMap<String, String>,
    current_image_id: String,
    goal_image_id: String,
    current_turn_player_id: Option<String>,
    current_options: Vec<String>,
    turn_start_time: Option<u64>,
    stage_start_time: u64,
    current_round: u32,
    scores: std::collections::HashMap<String, f32>,
    players_who_voted: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubmitActionRequest {
    player_id: String,
    option_index: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubmitVotesRequest {
    voter_id: String,
    votes: std::collections::HashMap<String, u8>,
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

/// POST /rooms/:room_id/action - Submit a player action.
async fn submit_action(
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
    Json(req): Json<SubmitActionRequest>,
) -> Result<StatusCode, AppError> {
    let mut manager = state.room_manager.write().await;
    let room_id = RoomId::from_string(&room_id_str)
        .map_err(|_| AppError::InvalidRoomId)?;
    let player_id = PlayerId::from_string(&req.player_id)
        .map_err(|_| AppError::InvalidPlayerId)?;
    
    let room = manager
        .get_room_mut(&room_id)
        .ok_or(RoomError::RoomNotFound)?;
    
    if let Some(game) = &mut room.game {
        game.submit_action(player_id, req.option_index)
            .map_err(|e| RoomError::Internal(e))?;
        Ok(StatusCode::OK)
    } else {
        Err(RoomError::Internal("Game not started".to_string()).into())
    }
}

/// POST /rooms/:room_id/votes - Submit votes.
async fn submit_votes(
    State(state): State<AppState>,
    Path(room_id_str): Path<String>,
    Json(req): Json<SubmitVotesRequest>,
) -> Result<StatusCode, AppError> {
    let mut manager = state.room_manager.write().await;
    let room_id = RoomId::from_string(&room_id_str)
        .map_err(|_| AppError::InvalidRoomId)?;
    let voter_id = PlayerId::from_string(&req.voter_id)
        .map_err(|_| AppError::InvalidPlayerId)?;
    
    let room = manager
        .get_room_mut(&room_id)
        .ok_or(RoomError::RoomNotFound)?;
    
    if let Some(game) = &mut room.game {
        let mut votes = std::collections::HashMap::new();
        for (target_str, stars) in req.votes {
            let target_id = PlayerId::from_string(&target_str)
                .map_err(|_| AppError::InvalidPlayerId)?;
            votes.insert(target_id, stars);
        }

        game.submit_votes(voter_id, votes)
            .map_err(|e| RoomError::Internal(e))?;
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
        player_current_objects: g.player_current_objects.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
        current_image_id: g.current_image.as_str().to_string(),
        goal_image_id: g.goal_image.as_str().to_string(),
        current_turn_player_id: g.current_player().map(|id| id.to_string()),
        current_options: g.current_options.clone(),
        turn_start_time: g.turn_start_time,
        stage_start_time: g.stage_start_time,
        current_round: g.current_round,
        scores: g.calculate_scores().iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        players_who_voted: g.players_who_voted.iter().map(|id| id.to_string()).collect(),
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

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
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
        
        (status, Json(ErrorResponse { message })).into_response()
    }
}
