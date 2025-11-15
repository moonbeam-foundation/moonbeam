#!/bin/bash
# Start Chopsticks fork networks with XCM connection
# Usage: ./scripts/start-chopsticks-xcm.sh [testnet|mainnet]

set -e

NETWORK=${1:-testnet}

echo "Starting Chopsticks XCM testing environment..."
echo "Network: $NETWORK"
echo ""

# Check if chopsticks is installed
if ! command -v chopsticks &> /dev/null; then
    echo "Error: chopsticks not found. Install with:"
    echo "  npm install -g @acala-network/chopsticks"
    exit 1
fi

# Create tmp directory for databases
mkdir -p test/tmp

# Determine which configs to use
if [ "$NETWORK" = "mainnet" ]; then
    MOONBEAM_CONFIG="test/configs/chopsticks/moonbeam.yml"
    ASSETHUB_CONFIG="test/configs/chopsticks/assethub-polkadot.yml"
    echo "Using Moonbeam + AssetHub Polkadot"
else
    MOONBEAM_CONFIG="test/configs/chopsticks/moonbase.yml"
    ASSETHUB_CONFIG="test/configs/chopsticks/westmint.yml"
    echo "Using Moonbase + Westmint (AssetHub Westend)"
fi

echo ""
echo "Starting networks in background..."
echo "Logs will be in test/tmp/"

# Start AssetHub fork
echo "1. Starting AssetHub fork on port 8001..."
chopsticks \
  --config="$ASSETHUB_CONFIG" \
  > test/tmp/chopsticks-assethub.log 2>&1 &
ASSETHUB_PID=$!
echo "   AssetHub PID: $ASSETHUB_PID"

# Wait a moment for AssetHub to start
sleep 5

# Start Moonbeam fork
echo "2. Starting Moonbeam fork on port 8000..."
chopsticks \
  --config="$MOONBEAM_CONFIG" \
  > test/tmp/chopsticks-moonbeam.log 2>&1 &
MOONBEAM_PID=$!
echo "   Moonbeam PID: $MOONBEAM_PID"

# Wait for Moonbeam to start
sleep 5

echo ""
echo "✅ Chopsticks networks started!"
echo ""
echo "Endpoints:"
echo "  Moonbeam:  ws://localhost:8000"
echo "  AssetHub:  ws://localhost:8001"
echo ""
echo "Logs:"
echo "  tail -f test/tmp/chopsticks-moonbeam.log"
echo "  tail -f test/tmp/chopsticks-assethub.log"
echo ""
echo "NOTE: XCM bridging between parachains requires a relaychain."
echo "For now, both chains are running independently."
echo "You can test XCM by using the native XCM pallets on each chain."
echo ""
echo "To stop all processes:"
echo "  kill $MOONBEAM_PID $ASSETHUB_PID"
echo ""
echo "Or use: ./scripts/stop-chopsticks.sh"
echo ""

# Save PIDs for cleanup script
echo "$MOONBEAM_PID" > test/tmp/moonbeam.pid
echo "$ASSETHUB_PID" > test/tmp/assethub.pid

# Wait for Ctrl+C
echo "Press Ctrl+C to stop all processes..."
trap "kill $MOONBEAM_PID $ASSETHUB_PID 2>/dev/null; exit" INT TERM

wait
