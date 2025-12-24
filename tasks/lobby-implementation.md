# Lobby Implementation Tasks

## Overview
Implement the welcome screen and lobby system for Big Picture, enabling room creation, player join/leave/rejoin, and game start with 2-8 players.

---

## Task 1: Set up Rust project structure (0.5 days)

**Goal**: Create a Cargo workspace with separate crates for domain logic, backend server, and Godot client bindings.

**Files to create**:
- `Cargo.toml` (workspace root)
- `crates/domain/Cargo.toml`
- `crates/domain/src/lib.rs`
- `crates/server/Cargo.toml`
- `crates/server/src/main.rs`
- `crates/client/Cargo.toml` (godot-rust bindings)
- `crates/client/src/lib.rs`

**Dependencies**:
- domain: serde, uuid, thiserror
- server: axum, tokio, tower-http, serde_json
- client: godot (gdext or gdnative), reqwest
- jugar
- pmat

**Acceptance**: `cargo build --all` succeeds.

---

## Task 2: Define core domain types (1 day)

**Goal**: Implement Room, Player, GameState, and related types matching the spec.

**Files to create/modify**:
- `crates/domain/src/types.rs` – RoomId, PlayerId, AvatarId, ImageId
- `crates/domain/src/room.rs` – Room struct and RoomState enum
- `crates/domain/src/player.rs` – Player struct
- `crates/domain/src/game.rs` – GameState, PlayerAction
- `crates/domain/src/lib.rs` – re-exports

**Key types** (from planning.md):
```rust
pub struct Room {
    pub id: RoomId,
    pub code: String,
    pub players: Vec<Player>,
    pub state: RoomState,
    pub game: Option<GameState>,
}

pub enum RoomState {
    Lobby,
    InGame,
    Finished,
}

pub struct Player {
    pub id: PlayerId,
    pub nickname: String,
    pub avatar_id: AvatarId,
    pub connected: bool,
}
```

**Acceptance**: All types compile with proper derives (Clone, Debug, Serialize, Deserialize).

---

## Task 3: Implement room management logic (2 days)

**Goal**: Create, join, leave, rejoin logic with capacity enforcement (2-8 players).

**Files to create/modify**:
- `crates/domain/src/room_manager.rs` – RoomManager struct
- `crates/domain/src/errors.rs` – RoomError, JoinError enums

**Key behaviors**:
- `create_room()` → generates 4-6 character room code
- `join_room(code, nickname, avatar)` → validates capacity, checks duplicates
- `leave_room(room_id, player_id)` → removes player
- `rejoin_room(code, nickname)` → allows same nickname to reconnect
- Enforce: 2-8 player limit, no joins after "All is in!"

**Edge cases**:
- Room full (8 players)
- Duplicate nickname handling
- Invalid room codes
- Joining after game started

**Acceptance**: Unit tests cover all acceptance criteria from planning.md sections: "Room creation and join", "Player limits", "Join/leave/rejoin".

---

## Task 4: Set up Axum backend server (1.5 days)

**Goal**: REST API for room operations with in-memory state.

**Files to create/modify**:
- `crates/server/src/main.rs` – server setup, routes
- `crates/server/src/routes/rooms.rs` – room endpoints
- `crates/server/src/state.rs` – AppState with RoomManager

**Endpoints**:
- `POST /rooms` → create room, return room_id and code
- `POST /rooms/:code/join` → join with nickname/avatar
- `DELETE /rooms/:room_id/players/:player_id` → leave
- `POST /rooms/:room_id/start` → "All is in!" transition
- `GET /rooms/:room_id` → get current lobby state

**Response format** (JSON):
```json
{
  "room_id": "uuid",
  "code": "ABC123",
  "players": [...],
  "state": "Lobby"
}
```

**Acceptance**: Server runs on localhost:3000, endpoints return correct JSON, handle errors with proper HTTP status codes.

---

## Task 5: Create Godot project with godot-rust (1 day)

**Goal**: Set up Godot 4.x project with Rust GDExtension bindings.

**Files to create**:
- `godot/project.godot` – Godot project config
- `godot/big_picture.gdextension` – extension config
- `crates/client/src/lib.rs` – register Rust classes with Godot

**Godot-Rust setup**:
- Use `gdext` crate for Godot 4.x
- Create basic GDExtension library
- Test that Godot can load Rust scripts

**Acceptance**: Godot editor loads the project, Rust extension is recognized.

---

## Task 6: Implement welcome screen scene (1.5 days)

**Goal**: Godot scene with "Start New Game" and "Join Game" UI.

**Files to create**:
- `godot/scenes/welcome_screen.tscn` – UI layout
- `crates/client/src/welcome_screen.rs` – Rust logic for the scene

**UI elements**:
- Title label: "Big Picture"
- Button: "Start New Game"
- Input field + button: "Join Game" (room code input)

**Rust behavior**:
- On "Start New Game": POST to `/rooms`, display room code
- On "Join Game": validate input, POST to `/rooms/:code/join`
- Transition to lobby screen on success

**Acceptance**: Manual test shows welcome screen, buttons trigger network calls, transitions work.

---

## Task 7: Implement lobby screen scene (2 days)

**Goal**: Display room code, player list, "All is in!" button.

**Files to create**:
- `godot/scenes/lobby_screen.tscn` – UI layout
- `crates/client/src/lobby_screen.rs` – Rust logic

**UI elements**:
- Room code display (large, prominent)
- Player list (nickname + avatar icons)
- "All is in!" button (enabled when 2-8 players)
- "Leave" button

**Rust behavior**:
- Poll `/rooms/:room_id` every 1-2 seconds for updates
- Update player list UI dynamically
- On "All is in!": POST to `/rooms/:room_id/start`
- On "Leave": DELETE to `/rooms/:room_id/players/:player_id`

**Acceptance**: 
- Multiple clients can join same room
- Player list updates in real-time
- "All is in!" only enabled with 2-8 players
- Transitions to game screen on start

---

## Task 8: Write unit tests (1 day)

**Goal**: Achieve >80% coverage for domain and room management logic.

**Files to create**:
- `crates/domain/tests/room_tests.rs`
- `crates/domain/tests/player_tests.rs`

**Test scenarios** (from planning.md acceptance criteria):
- Room creation generates valid code
- Join succeeds with <8 players
- Join fails with 8 players (room full)
- Leave removes player
- Rejoin with same nickname succeeds
- "All is in!" with 2-8 players transitions to InGame
- Join after start fails

**Acceptance**: `cargo test` passes all tests, covers edge cases.

---

## Open Questions & Risks

1. **Godot version**: Use Godot 4.x with `gdext` or Godot 3.x with `gdnative`?
   - Decision: Godot 4.x + `gdext` for future-proofing
   
2. **Room code generation**: How to ensure uniqueness?
   - Decision: Use random alphanumeric, check for collisions in RoomManager
   
3. **Player device clients**: Separate Godot client or web-based?
   - Decision: Start with Godot desktop client for all devices, optimize later
   
4. **State sync**: Polling vs WebSockets?
   - Decision: Start with polling (simpler), add WebSockets in iteration 2
   
5. **Avatar selection**: Predefined set or user upload?
   - Decision: Predefined set of 8-10 simple avatars (emoji or icons)
   
6. **Network error handling**: Retry logic, timeout values?
   - Decision: 5-second timeout, 3 retries with exponential backoff
   
7. **Complexity tracking**: When to measure?
   - Decision: After each major task, document in `complexity.md`

---

## Implementation Order

1. ✅ Project structure
2. ✅ Domain types
3. ✅ Room management logic + tests
4. ✅ Backend server
5. ✅ Godot project setup
6. ✅ Welcome screen
7. ✅ Lobby screen
8. ✅ Integration testing

Estimated total: 10-12 developer-days
