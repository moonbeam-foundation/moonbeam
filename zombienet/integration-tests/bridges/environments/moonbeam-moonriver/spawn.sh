#!/bin/bash

set -e

trap "trap - SIGTERM && kill -9 -$$" SIGINT SIGTERM EXIT

source "$FRAMEWORK_PATH/utils/zombienet.sh"
source "${BASH_SOURCE%/*}/helper.sh"

# whether to init the chains (create reserve assets, fund accounts, etc...)
init=0
start_relayer=0
while [ $# -ne 0 ]
do
    arg="$1"
    case "$arg" in
        --init)
            init=1
            ;;
        --start-relayer)
            start_relayer=1
            ;;
    esac
    shift
done

logs_dir=$TEST_DIR/logs
bridge_script="${BASH_SOURCE%/*}/bridge.sh"

moonbeam_def=${ZOMBIENET_CONFIGS}/moonbeam-polkadot.toml
start_zombienet $TEST_DIR $moonbeam_def moonbeam_dir moonbeam_pid
echo

moonriver_def=${ZOMBIENET_CONFIGS}/moonriver-kusama.toml
start_zombienet $TEST_DIR $moonriver_def moonriver_dir moonriver_pid
echo

if [[ $init -eq 1 ]]; then
  run_zndsl ${BASH_SOURCE%/*}/moonbeam-init.zndsl $moonbeam_dir
  run_zndsl ${BASH_SOURCE%/*}/moonriver-init.zndsl $moonriver_dir
fi

if [[ $start_relayer -eq 1 ]]; then
  ${BASH_SOURCE%/*}/start_relayer.sh $moonbeam_dir $moonriver_dir finality_relayer_pid parachains_relayer_pid messages_relayer_pid
fi

echo $moonbeam_dir > $TEST_DIR/moonbeam.env
echo $moonriver_dir > $TEST_DIR/moonriver.env
echo

wait_for_process $moonbeam_pid $moonriver_pid $finality_relayer_pid $parachains_relayer_pid $messages_relayer_pid;

kill -9 -$$