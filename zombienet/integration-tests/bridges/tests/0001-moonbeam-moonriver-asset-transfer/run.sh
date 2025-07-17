#!/bin/bash

# Test that checks if asset transfer works between Moonbeam<>Moonriver bridge.

set -e

source "$FRAMEWORK_PATH/utils/common.sh"
source "$FRAMEWORK_PATH/utils/zombienet.sh"

export ENV_PATH=`realpath ${BASH_SOURCE%/*}/../../environments/moonbeam-moonriver`

$ENV_PATH/spawn.sh --init --start-relayer &
env_pid=$!

ensure_process_file $env_pid $TEST_DIR/moonbeam.env 600
moonbeam_dir=`cat $TEST_DIR/moonbeam.env`
echo

ensure_process_file $env_pid $TEST_DIR/moonriver.env 300
moonriver_dir=`cat $TEST_DIR/moonriver.env`
echo

run_zndsl ${BASH_SOURCE%/*}/movr-reaches-moonbeam.zndsl $moonbeam_dir
run_zndsl ${BASH_SOURCE%/*}/glmr-reaches-moonriver.zndsl $moonriver_dir