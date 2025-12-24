# Godot Project Setup

## Prerequisites

- Godot 4.3+ (download from [godotengine.org](https://godotengine.org/download))
- Rust toolchain (already installed)

## Building the Extension

```bash
# Build the GDExtension library
cargo build -p big-picture-client

# For release builds:
cargo build -p big-picture-client --release
```

This creates:
- Linux: `target/debug/libbig_picture_client.so`
- Windows: `target/debug/big_picture_client.dll`
- macOS: `target/debug/libbig_picture_client.dylib`

## Opening in Godot

1. Open Godot 4.3+
2. Click "Import"
3. Navigate to `/home/pnentertainment/Documents/drawme/godot`
4. Select `project.godot`
5. Click "Import & Edit"

## Verifying the Extension

When Godot opens the project:

1. Check the **Output** panel for:
   ```
   Big Picture extension initialized!
   ```

2. In the **FileSystem** panel, you should see:
   - `res://big_picture.gdextension` (green checkmark if loaded)
   - `res://scenes/welcome_screen.tscn`

3. Double-click `welcome_screen.tscn` to open the scene

4. Click the "Play" button (F5) to run the game

5. Check the console for:
   ```
   WelcomeScreen initialized
   WelcomeScreen ready
   ```

## Testing the Extension

- Click "Start New Game" button → should see "Create Room button pressed" in console
- Click "Join Game" button → should see "Join Room button pressed" in console

## Troubleshooting

### Extension not loading

1. Make sure the library was built:
   ```bash
   ls -la target/debug/libbig_picture_client.*
   ```

2. Check Godot's output for errors

3. Verify the path in `godot/big_picture.gdextension` matches your build output

### Scenes not appearing

Make sure the server is running:
```bash
cargo run -p big-picture-server
```

## Project Structure

```
godot/
├── project.godot           # Godot project config
├── big_picture.gdextension # Extension configuration
└── scenes/
    └── welcome_screen.tscn # Welcome/lobby screen
```

## Next Steps

Once verified:
- Task 6: Implement welcome screen functionality (connect to server)
- Task 7: Implement lobby screen (player list, "All is in!" button)
- Task 8: Add comprehensive tests
