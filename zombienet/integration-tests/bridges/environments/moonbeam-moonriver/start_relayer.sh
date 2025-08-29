#!/bin/bash

set -e

source "$FRAMEWORK_PATH/utils/common.sh"
source "$FRAMEWORK_PATH/utils/zombienet.sh"

moonbeam_dir=$1
moonriver_dir=$2
__finality_relayer_pid=$3
__relay_headers_and_messages_pid=$4

logs_dir=$TEST_DIR/logs
bridge_script="${BASH_SOURCE%/*}/bridge.sh"

# start finality relayer
finality_relayer_log=$logs_dir/relayer_finality.log
echo -e "Starting moonbeam-moonriver finality relayer. Logs available at: $finality_relayer_log\n"
start_background_process "$bridge_script run-finality-relay" $finality_relayer_log finality_relayer_pid

# start parachains relayer
relay_headers_and_messages_log=$logs_dir/relayer_parachains.log
echo -e "Starting relay-headers-and-messages. Logs available at: $parachains_relayer_log\n"
start_background_process "$bridge_script relay-headers-and-messages" $relay_headers_and_messages_log relay_headers_and_messages_pid

run_zndsl ${BASH_SOURCE%/*}/moonbeam-bridge.zndsl $moonbeam_dir
run_zndsl ${BASH_SOURCE%/*}/moonriver-bridge.zndsl $moonriver_dir

eval $__finality_relayer_pid="'$finality_relayer_pid'"
eval $__relay_headers_and_messages_pid="'$relay_headers_and_messages_pid'"