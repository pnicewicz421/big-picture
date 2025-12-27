FROM rustlang/rust:nightly as builder
WORKDIR /app

# Copy workspace Cargo.toml without the client member
COPY Cargo.toml.docker ./Cargo.toml

# Copy only server and domain crates (exclude client which has Godot dependencies)
COPY crates/domain ./crates/domain
COPY crates/server ./crates/server

# Build only the server (release mode for optimization)
RUN cargo build --release -p big-picture-server

# Use a minimal runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary
COPY --from=builder /app/target/release/big-picture-server /usr/local/bin/

# Expose port 3000
EXPOSE 3000

# Run the server
CMD ["big-picture-server"]
