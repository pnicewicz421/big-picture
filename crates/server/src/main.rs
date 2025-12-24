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
//! - `POST /rooms` - Create new room
//! - `POST /rooms/:code/join` - Join room
//! - `DELETE /rooms/:room_id/players/:player_id` - Leave room
//! - `POST /rooms/:room_id/start` - Start game ("All is in!")
//! - `GET /rooms/:room_id` - Get room state

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "big_picture_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Big Picture Server starting...");

    // TODO: Set up routes and state
    // TODO: Start server on localhost:3000

    tracing::info!("Server ready at http://localhost:3000");
}
