#!/bin/bash

# import common functions
source "$FRAMEWORK_PATH/utils/bridges.sh"

LANE_ID="0000000000000000000000000000000000000000000000000000000000000000"

function init_polkadot_to_moonriver() {
    local relayer_path=$(ensure_relayer)

    RUST_LOG=info,runtime=trace,rpc=trace,bridge=trace \
        $relayer_path init-bridge polkadot-to-moonriver \
        --source-uri ws://localhost:9900 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8801 \
        --target-version-mode Auto \
        --target-signer $ALITH_PRIVATE_KEY
}

function init_kusama_to_moonbeam() {
    local relayer_path=$(ensure_relayer)

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path init-bridge kusama-to-moonbeam \
        --source-uri ws://localhost:9901 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8800 \
        --target-version-mode Auto \
        --target-signer $ALITH_PRIVATE_KEY
}

function run_relay() {
    local relayer_path=$(ensure_relayer)

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-headers-and-messages moonbeam-moonriver \
        --polkadot-uri ws://localhost:9900 \
        --polkadot-version-mode Auto \
        --moonbeam-uri ws://localhost:8800 \
        --moonbeam-version-mode Auto \
        --moonbeam-signer $ALITH_PRIVATE_KEY \
        --moonbeam-transactions-mortality 4 \
        --kusama-uri ws://localhost:9901 \
        --kusama-version-mode Auto \
        --moonriver-uri ws://localhost:8801 \
        --moonriver-version-mode Auto \
        --moonriver-signer $ALITH_PRIVATE_KEY \
        --moonriver-transactions-mortality 4 \
        --lane "${LANE_ID}"
}

function run_finality_relay() {
    local relayer_path=$(ensure_relayer)

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-headers polkadot-to-moonriver \
        --only-free-headers \
        --source-uri ws://localhost:9900 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8801 \
        --target-version-mode Auto \
        --target-signer $BOB_PRIVATE_KEY \
        --target-transactions-mortality 4&

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-headers kusama-to-moonbeam \
        --only-free-headers \
        --source-uri ws://localhost:9901 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8800 \
        --target-version-mode Auto \
        --target-signer $BOB_PRIVATE_KEY \
        --target-transactions-mortality 4
}

function run_parachains_relay() {
    local relayer_path=$(ensure_relayer)

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-parachains moonbeam-to-moonriver \
        --only-free-headers \
        --source-uri ws://localhost:9900 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8801 \
        --target-version-mode Auto \
        --target-signer $BOB_PRIVATE_KEY \
        --target-transactions-mortality 4&

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-parachains moonriver-to-moonbeam \
        --only-free-headers \
        --source-uri ws://localhost:9901 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8800 \
        --target-version-mode Auto \
        --target-signer $BOB_PRIVATE_KEY \
        --target-transactions-mortality 4
}

function run_messages_relay() {
    local relayer_path=$(ensure_relayer)

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-messages moonbeam-to-moonriver \
        --source-uri ws://localhost:8800 \
        --source-version-mode Auto \
        --source-signer $BOB_PRIVATE_KEY \
        --source-transactions-mortality 4 \
        --target-uri ws://localhost:8801 \
        --target-version-mode Auto \
        --target-signer $BOB_PRIVATE_KEY \
        --target-transactions-mortality 4 \
        --lane $LANE_ID&

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-messages moonriver-to-moonbeam \
        --source-uri ws://localhost:8801 \
        --source-version-mode Auto \
        --source-signer $BOB_PRIVATE_KEY \
        --source-transactions-mortality 4 \
        --target-uri ws://localhost:8800 \
        --target-version-mode Auto \
        --target-signer $BOB_PRIVATE_KEY \
        --target-transactions-mortality 4 \
        --lane $LANE_ID
}

case "$1" in
  run-relay)
    init_kusama_to_moonbeam
    init_polkadot_to_moonriver
    run_relay
    ;;
  run-finality-relay)
    init_kusama_to_moonbeam
    init_polkadot_to_moonriver
    run_finality_relay
    ;;
  run-parachains-relay)
    run_parachains_relay
    ;;
  run-messages-relay)
    run_messages_relay
    ;;
  reserve-transfer-assets-from-moonbeam-local)
      amount=$2
      ensure_polkadot_js_api
      # send GLMR to Alice account on Moonriver
      limited_reserve_transfer_assets \
          "ws://127.0.0.1:8800" \
          $ALITH_PRIVATE_KEY \
          "$(jq --null-input '{ "V4": { "parents": 2, "interior": { "X2": [ { "GlobalConsensus": "Kusama" }, { "Parachain": 1000 } ] } } }')" \
          "$(jq --null-input '{ "V4": { "parents": 0, "interior": { "X1": [ { "AccountId32": { "id": [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125] } } ] } } }')" \
          "$(jq --null-input '{ "V4": [ { "id": { "parents": 1, "interior": "Here" }, "fun": { "Fungible": '$amount' } } ] }')" \
          0 \
          "Unlimited"
      ;;
  *)
    echo "A command is require. Supported commands for:
    Local (zombienet) run:
          - run-relay
          - run-finality-relay
          - run-parachains-relay
          - run-messages-relay
          - reserve-transfer-assets-from-moonbeam-local";
    exit 1
    ;;
esac