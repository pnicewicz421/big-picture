#!/bin/bash

# Integration test script for Big Picture
cd "$(dirname "$0")"

echo "==================================="
echo "Big Picture Integration Test Suite"
echo "==================================="
echo ""

# Check server is running
echo "1. Testing backend server..."
if curl -s http://localhost:3000 > /dev/null; then
    echo "   ✓ Server is responding"
else
    echo "   ✗ Server is not running!"
    exit 1
fi

# Test room creation
echo ""
echo "2. Testing room creation endpoint..."
RESPONSE=$(curl -s -X POST http://localhost:3000/rooms -H "Content-Type: application/json")
ROOM_CODE=$(echo $RESPONSE | jq -r '.room_code')
ROOM_ID=$(echo $RESPONSE | jq -r '.room_id')

if [ ! -z "$ROOM_CODE" ] && [ "$ROOM_CODE" != "null" ]; then
    echo "   ✓ Created room with code: $ROOM_CODE"
else
    echo "   ✗ Failed to create room"
    exit 1
fi

# Test room joining
echo ""
echo "3. Testing room join endpoint..."
JOIN_RESPONSE=$(curl -s -X POST "http://localhost:3000/rooms/${ROOM_CODE}/join" \
    -H "Content-Type: application/json" \
    -d '{"nickname":"IntegrationTest","avatar_id":0}')
PLAYER_ID=$(echo $JOIN_RESPONSE | jq -r '.player_id')

if [ ! -z "$PLAYER_ID" ] && [ "$PLAYER_ID" != "null" ]; then
    echo "   ✓ Joined room as player: $PLAYER_ID"
else
    echo "   ✗ Failed to join room"
    exit 1
fi

# Test room state
echo ""
echo "4. Testing room state endpoint..."
STATE_RESPONSE=$(curl -s "http://localhost:3000/rooms/${ROOM_ID}")
PLAYER_COUNT=$(echo $STATE_RESPONSE | jq -r '.player_count')

if [ "$PLAYER_COUNT" == "1" ]; then
    echo "   ✓ Room state correct (1 player)"
else
    echo "   ✗ Unexpected player count: $PLAYER_COUNT"
    exit 1
fi

# Test Godot extension
echo ""
echo "5. Testing Godot extension..."
if [ -f "target/release/libbig_picture_client.so" ]; then
    echo "   ✓ Extension library exists"
else
    echo "   ✗ Extension library not found"
    exit 1
fi

echo ""
echo "==================================="
echo "All Integration Tests Passed! ✓"
echo "==================================="
echo ""
echo "Backend Server:"
echo "  - Health check: ✓"
echo "  - Room creation: ✓"
echo "  - Room joining: ✓"
echo "  - Room state: ✓"
echo ""
echo "Godot Client:"
echo "  - Extension library: ✓"
echo "  - Ready for manual testing in Godot editor"
echo ""
echo "Next steps:"
echo "  1. Run: cd godot && godot"
echo "  2. Open scenes/welcome_screen.tscn"
echo "  3. Press F6 to run the scene"
echo "  4. Test 'Create Room' and 'Join Room' buttons"
