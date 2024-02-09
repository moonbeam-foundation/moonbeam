#!/bin/bash

if [ $# -eq 0 ]; then
    echo "Error: No runtime specified."
    echo "Usage: $0 <runtime>"
    exit 1
fi

# Arguments
RUNTIME=$1

# Derived Values
SCRIPT_DIR=$(dirname "$0")

case $RUNTIME in
    moonbase)
        REMOTE_URI="wss://wss.api.moonbase.moonbeam.network"
    ;;
    moonbeam)
        REMOTE_URI="wss://wss.api.moonbeam.network"
    ;;
    moonriver)
        REMOTE_URI="wss://wss.api.moonriver.moonbeam.network"
    ;;
    *)
        echo "Error: Invalid runtime specified. Valid options are moonbase, moonbeam, moonriver."
        exit 2
    ;;
esac

# STATIC Values
LOCAL_URI="ws://127.0.0.1:9944"

cleanup() {
    kill $(cat /tmp/moonbeam_dev.pid)
}

trap cleanup EXIT INT TERM

nohup setsid $SCRIPT_DIR/../target/release/moonbeam --chain $RUNTIME-dev --tmp > /dev/null 2>&1 &
echo $! > /tmp/moonbeam_dev.pid
sleep 4

remote_output=$(subxt explore --url $REMOTE_URI pallet System constants Version)
filtered_remote=$(echo "$remote_output" | sed -n '/The value of the constant is:/,$p' | sed '1d')
remote_version=$(echo "$filtered_remote" | grep "spec_version" | awk -F': ' '{print $2}' | tr -d ',')

echo "The remote spec_version is: $remote_version"

local_output=$(subxt explore --url $LOCAL_URI pallet System constants Version)
filtered_local=$(echo "$local_output" | sed -n '/The value of the constant is:/,$p' | sed '1d')
local_version=$(echo "$filtered_local" | grep "spec_version" | awk -F': ' '{print $2}' | tr -d ',')

local_version=$(echo "$local_version" | sed 's/\x1b\[[0-9;]*m//g')

echo "The local spec_version is: $local_version"

mkdir -p "$SCRIPT_DIR/../runtime-diffs/$RUNTIME"

subxt diff -a $REMOTE_URI $LOCAL_URI
subxt diff -a $REMOTE_URI $LOCAL_URI | sed 's/\x1b\[[0-9;]*m//g' > "$SCRIPT_DIR/../runtime-diffs/$RUNTIME/$local_version.txt"
echo "saved to '$SCRIPT_DIR/../runtime-diffs/$RUNTIME/$local_version.txt'"
