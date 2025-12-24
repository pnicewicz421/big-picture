# Testing the Big Picture Server

## Running the Server

```bash
cargo run -p big-picture-server
```

Server runs on `http://localhost:3000`

## API Endpoints

### Health Check
```bash
curl http://localhost:3000
# Returns: Big Picture Server v0.1.0
```

### Create Room
```bash
curl -X POST http://localhost:3000/rooms
# Returns: {"room_code":"ABC123","room_id":"uuid-here"}
```

### Join Room
```bash
curl -X POST http://localhost:3000/rooms/ABC123/join \
  -H "Content-Type: application/json" \
  -d '{"nickname":"Alice","avatar_id":0}'
# Returns: {"player_id":"uuid","room_id":"uuid"}
```

### Get Room State
```bash
curl http://localhost:3000/rooms/{room_id}
# Returns room info with player list
```

### Leave Room
```bash
curl -X POST http://localhost:3000/rooms/{room_id}/leave \
  -H "Content-Type: application/json" \
  -d '{"player_id":"uuid"}'
```

### Rejoin Room
```bash
curl -X POST http://localhost:3000/rooms/ABC123/rejoin \
  -H "Content-Type: application/json" \
  -d '{"nickname":"Alice"}'
```

### Start Game
```bash
curl -X POST http://localhost:3000/rooms/{room_id}/start
# Requires 2-8 players in lobby
```

## Example Test Sequence

```bash
# 1. Create room
ROOM=$(curl -s -X POST http://localhost:3000/rooms)
CODE=$(echo $ROOM | grep -o '"room_code":"[^"]*"' | cut -d'"' -f4)
ROOM_ID=$(echo $ROOM | grep -o '"room_id":"[^"]*"' | cut -d'"' -f4)

echo "Room Code: $CODE"
echo "Room ID: $ROOM_ID"

# 2. Join as player 1
PLAYER1=$(curl -s -X POST http://localhost:3000/rooms/$CODE/join \
  -H "Content-Type: application/json" \
  -d '{"nickname":"Alice","avatar_id":0}')

# 3. Join as player 2
PLAYER2=$(curl -s -X POST http://localhost:3000/rooms/$CODE/join \
  -H "Content-Type: application/json" \
  -d '{"nickname":"Bob","avatar_id":1}')

# 4. Check room state
curl -s http://localhost:3000/rooms/$ROOM_ID | python3 -m json.tool

# 5. Start game
curl -s -X POST http://localhost:3000/rooms/$ROOM_ID/start
```
