#!/bin/bash
# Quick manual test guide for Godot integration

echo "════════════════════════════════════════════════════════════"
echo "   Big Picture - Godot Integration Manual Test Guide"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "📋 Prerequisites Check:"
echo ""

# Check server
if curl -s http://localhost:3000 > /dev/null 2>&1; then
    echo "✅ Server is running on localhost:3000"
else
    echo "❌ Server is NOT running!"
    echo ""
    echo "Start it with:"
    echo "  cd /home/pnentertainment/Documents/drawme"
    echo "  cargo run -p big-picture-server"
    echo ""
    exit 1
fi

# Check library
if [ -f "/home/pnentertainment/Documents/drawme/target/release/libbig_picture_client.so" ]; then
    echo "✅ Godot extension library is built"
else
    echo "❌ Extension library NOT found!"
    echo ""
    echo "Build it with:"
    echo "  cargo build -p big-picture-client --release"
    echo ""
    exit 1
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo "   Ready to Test! Follow These Steps:"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "1️⃣  Launch Godot:"
echo "   cd godot && godot"
echo ""
echo "2️⃣  Open the scene:"
echo "   - In the FileSystem panel, navigate to scenes/"
echo "   - Double-click 'welcome_screen.tscn'"
echo ""
echo "3️⃣  Run the scene:"
echo "   - Press F6 (or click 'Run Current Scene' button)"
echo "   - Or use the play button with scene icon"
echo ""
echo "4️⃣  Test 'Create Room':"
echo "   - Click the 'Create Room' button"
echo "   - Watch the status label update"
echo "   - Note the room code that appears"
echo "   - Check console for: 'Room created! Code: XXXXXX'"
echo ""
echo "5️⃣  Test 'Join Room':"
echo "   Option A - Using another Godot instance:"
echo "     - Open another Godot window"
echo "     - Run the same scene"
echo "     - Type the room code"
echo "     - Click 'Join Room'"
echo ""
echo "   Option B - Using curl (in another terminal):"
ROOM_CODE="EXAMPLE"
echo "     curl -X POST 'http://localhost:3000/rooms/$ROOM_CODE/join' \\"
echo "       -H 'Content-Type: application/json' \\"
echo "       -d '{\"nickname\":\"TestPlayer\",\"avatar_id\":1}'"
echo ""
echo "════════════════════════════════════════════════════════════"
echo "   What to Look For:"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "✓ Extension loads without errors"
echo "✓ 'Create Room' button becomes active after clicking"
echo "✓ Status shows: 'Room created! Code: XXXXXX'"
echo "✓ Console shows: 'Room code: XXXXXX, Room ID: ...'"
echo "✓ Auto-join happens: 'Joined room XXXXXX as HostXXXX'"
echo "✓ 'Join Room' accepts room code input"
echo "✓ Status shows: 'Joined room XXXXXX as PlayerXXXX'"
echo "✓ No errors in Output panel"
echo ""
echo "════════════════════════════════════════════════════════════"
echo "   Expected Console Output Sample:"
echo "════════════════════════════════════════════════════════════"
echo ""
cat << 'EOF'
WelcomeScreen initialized
WelcomeScreen ready
Status: Ready to play! Make sure server is running...

[After clicking Create Room]
Create Room button pressed
Requesting: POST http://localhost:3000/rooms
Create room response: code=200
Response body: {"room_code":"VAZPX3","room_id":"025d8..."}
Room code: VAZPX3, Room ID: 025d8d59-956a-463e-b866-58b429ab1471
Auto-joining as host: POST http://localhost:3000/rooms/VAZPX3/join
Join room response: code=200
Response body: {"player_id":"1aa084...","room_id":"025d8..."}
Joined room VAZPX3 as Host1234
Player ID: 1aa084ae-..., Room ID: 025d8...
Ready to transition to lobby!
EOF

echo ""
echo "════════════════════════════════════════════════════════════"
echo "All systems ready! Press ENTER to launch Godot..."
read

cd /home/pnentertainment/Documents/drawme/godot && godot
