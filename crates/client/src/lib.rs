//! # Big Picture Godot Client
//!
//! Rust GDExtension for Godot 4.x, implementing UI logic and backend communication.
//!
//! ## Scenes
//!
//! - Welcome screen: Room creation and join UI
//! - Lobby screen: Player list and "All is in!" button
//! - Game screen: Image display, turn indicator, player options
//!
//! ## Architecture
//!
//! - Rust classes extend Godot nodes via `godot::prelude`
//! - HTTP communication with backend server using `reqwest`
//! - State synchronized via polling (future: WebSockets)

use godot::prelude::*;

struct BigPictureExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BigPictureExtension {}

// TODO: Implement GDExtension classes for:
// - WelcomeScreen
// - LobbyScreen
// - GameScreen
