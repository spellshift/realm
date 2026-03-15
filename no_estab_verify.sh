#!/bin/bash

# Configuration
export IMIX_LOG=debug
export ENABLE_DEBUG_LOGGING=true
export IMIX_CALLBACK_URI="http1://127.0.0.1:8080"
export IMIX_SERVER_PUBKEY="GTxio8fGpjhdOXKBiRtBa2GipkB2uDlbxSJPRfLAHxc="
REDIRECTOR_PORT=8080

# Create temporary log files
TEMP=$(mktemp -d)
TAVERN_LOG="$TEMP/tavern.log"
REDIRECTOR_LOG="$TEMP/redirector.log"
AGENT_LOG="$TEMP/agent.log"

echo "Logs will be collected in:"
echo "  Tavern: $TAVERN_LOG"
echo "  Redirector: $REDIRECTOR_LOG"
echo "  Agent: $AGENT_LOG"

# Kill existing processes
pkill -f "go run ./tavern" || true
pkill -f "imix" || true

# Start Tavern server
echo "Starting Tavern server..."
go run ./tavern/ > "$TAVERN_LOG" 2>&1 &
TAVERN_PID=$!

# Wait for Tavern to start
echo "Waiting 10s for Tavern..."
sleep 10

# Start Redirector
echo "Starting HTTP1 Redirector..."
go run ./tavern/... -- redirector --transport http1 http://127.0.0.1:8000 > "$REDIRECTOR_LOG" 2>&1 &
REDIRECTOR_PID=$!

# Wait for Redirector to start
echo "Waiting 5s for Redirector..."
sleep 5

# Start Agent
echo "Starting Imix Agent..."
(cd implants/imix && cargo run) > "$AGENT_LOG" 2>&1 &
AGENT_PID=$!

# Prompts for Agent (AI)
echo "----------------------------------------------------------------"
echo "ACTION REQUIRED BY AGENT:"
echo "NOTE: The shell is very slow. When interacting send one character. Wait 1 second. Then Send another."
echo "1. Navigate to the Web UI (http://localhost:8000)"
echo "2. Create a Quest"
echo "3. Click the beacon"
echo "4. Click continue"
echo "5. Search for 'Reverse Shell'"
echo "6. Click continue"
echo "7. Click submit"
echo "8. Wait five seconds for the quest to complete"
echo "9. Click the shells tab"
echo "10. Click open shell"
echo "11. Wait for the terminal prompt to load"
echo "12. In the terminal, enter: echo \"hello world\""
echo "13. Press Enter and wait for output."
echo "14. Verify the output is "hello world""
echo "----------------------------------------------------------------"
read -p "Press Enter HERE once the command has been executed in the UI..."

# Monitor for ESTABLISHED connections
echo "Monitoring HTTP connections on port $REDIRECTOR_PORT for persistent states..."
  persistent_found=0
  last_ports=""
  exit_code=1 # Default to failure

  for i in {1..40}; do
    # Get all established connections to the redirector port
    # Format: local_port remote_address:port
    current_conns=$(ss -Hptuna | grep ":$REDIRECTOR_PORT" | grep "ESTAB" | awk '{print $5}' | sort | uniq)
    
    if [ -n "$current_conns" ]; then
      echo "[Check $i] Active ESTABLISHED ports:"
      echo "$current_conns"
      
      # Check if any port in current_conns was also in last_ports
      for port in $current_conns; do
        if echo "$last_ports" | grep -q "^$port$"; then
          echo "WARNING: Port $port has been ESTABLISHED for at least 2 consecutive checks!"
          persistent_found=$((persistent_found + 1))
        fi
      done
      last_ports="$current_conns"
    else
      echo "[Check $i] No ESTABLISHED connections."
      last_ports=""
    fi
    
    # If we found persistent connections for multiple rounds, it's a failure
    if [ $persistent_found -gt 5 ]; then
      echo "FAILED: Found persistent ESTABLISHED connections across multiple checks."
      exit_code=1
      break
    fi
    
    sleep 1
  done

  if [ $persistent_found -le 5 ]; then
    echo "SUCCESS: No persistent lingering connections detected. All connections were short-lived."
    exit_code=0
  fi

# Cleanup
echo "Cleaning up..."
kill $TAVERN_PID $REDIRECTOR_PID $AGENT_PID 2>/dev/null || true
pkill -f "tavern" || true
pkill -f "imix" || true

exit $exit_code
