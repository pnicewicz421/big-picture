//! Welcome screen controller for creating/joining rooms.

use godot::prelude::*;
use godot::classes::{Control, IControl};

#[derive(GodotClass)]
#[class(base=Control)]
pub struct WelcomeScreen {
    base: Base<Control>,
    
    #[var]
    server_url: GString,
}

#[godot_api]
impl IControl for WelcomeScreen {
    fn init(base: Base<Control>) -> Self {
        godot_print!("WelcomeScreen initialized");
        Self {
            base,
            server_url: "http://localhost:3000".into(),
        }
    }
    
    fn ready(&mut self) {
        godot_print!("WelcomeScreen ready");
    }
}

#[godot_api]
impl WelcomeScreen {
    #[func]
    fn on_create_room_pressed(&mut self) {
        godot_print!("Create Room button pressed");
        
        // Call deferred to not block the main thread
        self.base_mut().call_deferred("create_room_async", &[]);
    }
    
    #[func]
    fn on_join_room_pressed(&mut self) {
        godot_print!("Join Room button pressed");
        
        // Get room code from LineEdit node
        // We'll implement this when we create the actual scene
    }
    
    #[func]
    fn create_room_async(&mut self) {
        // This will be called deferred to not block the main thread
        // In a real implementation, we'd use Godot's thread pool or signals
        godot_print!("Creating room...");
    }
    
    #[func]
    fn set_status_message(&mut self, message: GString) {
        godot_print!("Status: {}", message);
        // Update a Label node with the status message
    }
}
