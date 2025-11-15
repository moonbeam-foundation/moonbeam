#!/bin/bash
# Stop all Chopsticks processes

echo "Stopping Chopsticks networks..."

if [ -f test/tmp/moonbeam.pid ]; then
    MOONBEAM_PID=$(cat test/tmp/moonbeam.pid)
    kill $MOONBEAM_PID 2>/dev/null && echo "Stopped Moonbeam (PID $MOONBEAM_PID)"
    rm test/tmp/moonbeam.pid
fi

if [ -f test/tmp/assethub.pid ]; then
    ASSETHUB_PID=$(cat test/tmp/assethub.pid)
    kill $ASSETHUB_PID 2>/dev/null && echo "Stopped AssetHub (PID $ASSETHUB_PID)"
    rm test/tmp/assethub.pid
fi

if [ -f test/tmp/xcm.pid ]; then
    XCM_PID=$(cat test/tmp/xcm.pid)
    kill $XCM_PID 2>/dev/null && echo "Stopped XCM Bridge (PID $XCM_PID)"
    rm test/tmp/xcm.pid
fi

# Also kill any remaining chopsticks processes
pkill -f chopsticks 2>/dev/null && echo "Killed remaining chopsticks processes"

echo "Done!"
