#!/bin/bash

# import common functions
source "$FRAMEWORK_PATH/utils/bridges.sh"

function transfer_assets() {
    local url=$1
    local seed=$2
    local destination=$3
    local beneficiary=$4
    local assets=$5
    local fee_asset_item=$6
    local weight_limit=$7
    echo "  calling transfer_assets:"
    echo "      url: ${url}"
    echo "      seed: ${seed}"
    echo "      destination: ${destination}"
    echo "      beneficiary: ${beneficiary}"
    echo "      assets: ${assets}"
    echo "      fee_asset_item: ${fee_asset_item}"
    echo "      weight_limit: ${weight_limit}"
    echo ""
    echo "--------------------------------------------------"

    call_polkadot_js_api \
        --ws "${url?}" \
        --seed "${seed?}" \
        --sign ethereum \
        tx.polkadotXcm.transferAssets \
            "${destination}" \
            "${beneficiary}" \
            "${assets}" \
            "${fee_asset_item}" \
            "${weight_limit}"
}

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
        --target-signer $BALTATHAR_PRIVATE_KEY \
        --target-transactions-mortality 4&

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-headers kusama-to-moonbeam \
        --only-free-headers \
        --source-uri ws://localhost:9901 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8800 \
        --target-version-mode Auto \
        --target-signer $BALTATHAR_PRIVATE_KEY \
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
        --target-signer $BALTATHAR_PRIVATE_KEY \
        --target-transactions-mortality 4&

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-parachains moonriver-to-moonbeam \
        --only-free-headers \
        --source-uri ws://localhost:9901 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8800 \
        --target-version-mode Auto \
        --target-signer $BALTATHAR_PRIVATE_KEY \
        --target-transactions-mortality 4
}

function run_messages_relay() {
    local relayer_path=$(ensure_relayer)

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-messages moonbeam-to-moonriver \
        --source-uri ws://localhost:8800 \
        --source-version-mode Auto \
        --source-signer $BALTATHAR_PRIVATE_KEY \
        --source-transactions-mortality 4 \
        --target-uri ws://localhost:8801 \
        --target-version-mode Auto \
        --target-signer $BALTATHAR_PRIVATE_KEY \
        --target-transactions-mortality 4 \
        --lane $LANE_ID&

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $relayer_path relay-messages moonriver-to-moonbeam \
        --source-uri ws://localhost:8801 \
        --source-version-mode Auto \
        --source-signer $BALTATHAR_PRIVATE_KEY \
        --source-transactions-mortality 4 \
        --target-uri ws://localhost:8800 \
        --target-version-mode Auto \
        --target-signer $BALTATHAR_PRIVATE_KEY \
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
      transfer_assets \
          "ws://127.0.0.1:8800" \
          $ALITH_PRIVATE_KEY \
          "$(jq --null-input '{ "V5": { "parents": 2, "interior": { "X2": [ { "GlobalConsensus": "Kusama" }, { "Parachain": 2023 } ] } } }')" \
          "$(jq --null-input '{ "V5": { "parents": 0, "interior": { "X1": [ { "AccountKey20": { "key": [242, 79, 243, 169, 207, 4, 199, 29, 188, 148, 208, 181, 102, 247, 162, 123, 148, 86, 108, 172] } } ] } } }')" \
          "$(jq --null-input '{ "V5": [ { "id": { "parents": 0, "interior": { "X1": [ { "PalletInstance": 10  } ] } }, "fun": { "Fungible": '$amount' } } ] }')" \
          0 \
          "Unlimited"
      ;;
  reserve-transfer-assets-from-moonriver-local)
      amount=$2
      ensure_polkadot_js_api
      # send MOVR to Alice account on Moonbeam
      transfer_assets \
          "ws://127.0.0.1:8801" \
          $ALITH_PRIVATE_KEY \
          "$(jq --null-input '{ "V5": { "parents": 2, "interior": { "X2": [ { "GlobalConsensus": "Polkadot" }, { "Parachain": 2004 } ] } } }')" \
          "$(jq --null-input '{ "V5": { "parents": 0, "interior": { "X1": [ { "AccountKey20": { "key": [242, 79, 243, 169, 207, 4, 199, 29, 188, 148, 208, 181, 102, 247, 162, 123, 148, 86, 108, 172] } } ] } } }')" \
          "$(jq --null-input '{ "V5": [ { "id": { "parents": 0, "interior": { "X1": [ { "PalletInstance": 10  } ] } }, "fun": { "Fungible": '$amount' } } ] }')" \
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