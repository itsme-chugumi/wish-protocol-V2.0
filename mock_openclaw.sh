#!/bin/sh
# Read input (which we ignore in this mock)
cat > /dev/null

# Respond with success
echo '{"action": "grant", "response": {"status": "ok", "message": "Request granted"}}'
