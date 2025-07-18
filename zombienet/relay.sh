RELAYER_PATH="./zombienet/bin/substrate-relay"

LANE_ID="0000000000000000000000000000000000000000000000000000000000000000"

function init_bridge() {
    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $RELAYER_PATH init-bridge kusama-to-moonbeam \
        --source-uri ws://localhost:9901 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8800 \
        --target-version-mode Auto \
        --target-signer 0x4bbd6844d0f80895d861a5664809c1c0ed4fd94e43e25ba46649cba8c08fc5ad

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $RELAYER_PATH init-bridge polkadot-to-moonriver \
        --source-uri ws://localhost:9900 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8801 \
        --target-version-mode Auto \
        --target-signer 0x4bbd6844d0f80895d861a5664809c1c0ed4fd94e43e25ba46649cba8c08fc5ad
}

function init_bridge2() {
    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $RELAYER_PATH init-bridge betanet-to-stagenet \
        --source-uri wss://relay.api.moonbase.moonbeam.network \
        --source-version-mode Auto \
        --target-uri wss://wss.api.moondev.network \
        --target-version-mode Auto \
        --target-signer 0x4bbd6844d0f80895d861a5664809c1c0ed4fd94e43e25ba46649cba8c08fc5ad

    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $RELAYER_PATH init-bridge stagenet-to-betanet \
        --source-uri wss://stagenet-relay.api.moondev.network \
        --source-version-mode Auto \
        --target-uri wss://moonbase-beta.api.moonbase.moonbeam.network \
        --target-version-mode Auto \
        --target-signer 0x4bbd6844d0f80895d861a5664809c1c0ed4fd94e43e25ba46649cba8c08fc5ad
}

function run_finality_relay() {
    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $RELAYER_PATH relay-headers stagenet-to-betanet \
        --only-free-headers \
        --source-uri ws://localhost:9900 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8801 \
        --target-version-mode Auto \
	      --target-signer 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 \
        --target-transactions-mortality 4
}

function run_parachains_relay() {
    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $RELAYER_PATH relay-parachains polkadot-to-moonriver \
        --only-free-headers \
        --source-uri ws://localhost:9900 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8801 \
        --target-version-mode Auto \
	      --target-signer 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 \
        --target-transactions-mortality 4
}

function run_messages_relay() {
    RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $RELAYER_PATH relay-messages moonbeam-to-moonriver \
        --source-uri ws://localhost:8800 \
        --source-version-mode Auto \
        --target-uri ws://localhost:8801 \
        --target-version-mode Auto \
	      --source-signer 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 \
	      --target-signer 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 \
        --target-transactions-mortality 4 \
        --lane "$LANE_ID"
}

function run_relay_headers_and_messages() {
  #  RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $RELAYER_PATH relay-headers-and-messages moonbeam-moonriver \
        --kusama-uri ws://localhost:9901 \
        --kusama-version-mode Auto \
        --moonriver-uri ws://localhost:8801 \
        --moonriver-version-mode Auto \
        --moonriver-signer 0x4bbd6844d0f80895d861a5664809c1c0ed4fd94e43e25ba46649cba8c08fc5ad \
        --moonriver-transactions-mortality 16 \
        --polkadot-uri ws://localhost:9900 \
        --polkadot-version-mode Auto \
        --moonbeam-uri ws://localhost:8800 \
        --moonbeam-version-mode Auto \
        --moonbeam-signer 0x4bbd6844d0f80895d861a5664809c1c0ed4fd94e43e25ba46649cba8c08fc5ad \
        --moonbeam-transactions-mortality 16 \
        --no-prometheus \
        --only-free-headers \
        --lane "$LANE_ID"
}

function run_relay_headers_and_messages2() {
  #RUST_LOG=runtime=trace,rpc=trace,bridge=trace \
        $RELAYER_PATH relay-headers-and-messages betanet-stagenet \
        --betanet-relay-uri wss://relay.api.moonbase.moonbeam.network \
        --betanet-relay-version-mode Auto \
        --betanet-uri wss://moonbase-beta.api.moonbase.moonbeam.network \
        --betanet-version-mode Auto \
        --betanet-signer 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 \
        --betanet-transactions-mortality 16 \
        --stagenet-relay-uri wss://stagenet-relay.api.moondev.network \
        --stagenet-relay-version-mode Auto \
        --stagenet-uri wss://wss.api.moondev.network \
        --stagenet-version-mode Auto \
        --stagenet-signer 0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133 \
        --stagenet-transactions-mortality 16 \
        --no-prometheus \
        --only-free-headers \
        --lane "0000000000000000000000000000000000000000000000000000000000000000"
}

case "$1" in
  run-init-bridge)
    init_bridge
    ;;
  run-finality-relay)
    run_finality_relay
    ;;
  run-parachains-relay)
    run_parachains_relay
    ;;
  run-messages-relay)
    run_messages_relay
    ;;
  run-relay-headers-and-messages)
    run_relay_headers_and_messages
    ;;
  codegen)
    cd tools/runtime-codegen
    cargo run --bin runtime-codegen -- --from-node-url "ws://127.0.0.1:8800" > ../../relay-clients/client-moonbeam/src/codegen_runtime.rs
    cargo run --bin runtime-codegen -- --from-node-url "ws://127.0.0.1:8801" > ../../relay-clients/client-moonriver/src/codegen_runtime.rs
    ;;
  *)
    echo "A command is require. Supported commands for:
    Local (zombienet) run:
          - run-init-bridge
          - run-finality-relay
          - run-parachains-relay
          - run-messages-relay
          - run-relay-headers-and-messages";
    exit 1
    ;;
esac
