# Running and Deploying Big Picture

## Local Development

### Server
To run the server locally:
```bash
cd crates/server
cargo run
```
The server will be available at `http://localhost:3000`.

### Client (Godot)
1. Build the Rust library:
   ```bash
   cd crates/client
   cargo build
   ```
2. Open the project in Godot 4.x.
3. Run the project from the Godot editor.

## Deployment (Fly.io)

The server is deployed to Fly.io using Docker.

### Prerequisites
- [Fly.io CLI](https://fly.io/docs/hands-on/install-flyctl/)
- A Fly.io account with billing information configured.

### Deployment Steps
1. Ensure `Cargo.toml.docker` is up to date with the workspace members (excluding `crates/client`).
2. Run the deployment command:
   ```bash
   fly deploy
   ```
3. The server will be available at `https://big-picture-game.fly.dev`.

### Docker Configuration
The project uses a multi-stage Dockerfile:
- **Builder**: Uses `rustlang/rust:nightly` to support `edition2024` dependencies.
- **Runner**: Uses `debian:bookworm-slim` for a small footprint.
- **Workspace**: Uses `Cargo.toml.docker` to avoid building the Godot client in the cloud environment.

## Multi-Device Testing
1. Deploy the server to Fly.io.
2. Update the `server_url` in `crates/client/src/welcome_screen.rs` and `crates/client/src/lobby_screen.rs` to `https://big-picture-game.fly.dev`.
3. Export the Godot client for your target platforms (Android, iOS, Web, etc.).
4. Players can also join via the web interface at `https://big-picture-game.fly.dev` to create or join rooms.
