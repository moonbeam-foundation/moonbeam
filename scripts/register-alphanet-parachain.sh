#!/bin/bash

source scripts/_init_var.sh

RELAY_PORT=$((USER_PORT + 42))
RELAY_INDEX=0
BOOTNODES_ARGS=""


if [ -z "$SUDO_SEED" ]; then
    echo "Missing \$SUDO_SEED"
    exit 1
fi

if [ ! -f "$PARACHAIN_WASM" ]; then
    echo "Missing $PARACHAIN_WASM. Please run scripts/generate-parachain-specs.sh"
    exit 1
fi

if [ ! -f "$PARACHAIN_GENESIS" ]; then
    echo "Missing $PARACHAIN_GENESIS. Please run scripts/generate-parachain-specs.sh"
    exit 1
fi


PARACHAIN_CONFIG="$PARACHAIN_BUILD_FOLDER/moonbase-alphanet-runtime.config.json";
RELAYCHAIN_TYPES="$PARACHAIN_BUILD_FOLDER/moonbase-alphanet-relay-types.json";
echo -n "1000 {\"genesisHead\":\"$(cat $PARACHAIN_GENESIS)\",\"validationCode\":\"" > $PARACHAIN_CONFIG;
cat $PARACHAIN_WASM  >> $PARACHAIN_CONFIG;
echo -n "\",\"parachain\":true}" >> $PARACHAIN_CONFIG;

echo '{
  "HrmpChannelId": {
    "sender": "u32",
    "receiver": "u32"
  },
  "SignedAvailabilityBitfield": {
    "payload": "BitVec",
    "validator_index": "u32",
    "signature": "Signature"
  },
  "SignedAvailabilityBitfields": "Vec<SignedAvailabilityBitfield>",
  "ValidatorSignature": "Signature",
  "HeadData": "Vec<u8>",
  "CandidateDescriptor": {
    "para_id": "u32",
    "relay_parent": "Hash",
    "collator_id": "Hash",
    "persisted_validation_data_hash": "Hash",
    "pov_hash": "Hash",
    "erasure_root": "Hash",
    "signature": "Signature"
  },
  "CandidateReceipt": {
    "descriptor": "CandidateDescriptor",
    "commitments_hash": "Hash"
  },
  "UpwardMessage": "Vec<u8>",
  "OutboundHrmpMessage": {
    "recipient": "u32",
    "data": "Vec<u8>"
  },
  "ValidationCode": "Vec<u8>",
  "CandidateCommitments": {
    "upward_messages": "Vec<UpwardMessage>",
    "horizontal_messages": "Vec<OutboundHrmpMessage>",
    "new_validation_code": "Option<ValidationCode>",
    "head_data": "HeadData",
    "processed_downward_messages": "u32",
    "hrmp_watermark": "BlockNumber"
  },
  "CommittedCandidateReceipt": {
    "descriptor": "CandidateDescriptor",
    "commitments": "CandidateCommitments"
  },
  "ValidityAttestation": {
    "_enum": {
      "DummyOffsetBy1": "Raw",
      "Implicit": "ValidatorSignature",
      "Explicit": "ValidatorSignature"
    }
  },
  "BackedCandidate": {
    "candidate": "CommittedCandidateReceipt",
    "validity_votes": "Vec<ValidityAttestation>",
    "validator_indices": "BitVec"
  },
  "CandidatePendingAvailablility": {
    "core": "u32",
    "descriptor": "CandidateDescriptor",
    "availability_votes": "BitVec",
    "relay_parent_number": "BlockNumber",
    "backed_in_number": "BlockNumber"
  }
}' > $RELAYCHAIN_TYPES;


docker run --rm --network=host \
  -v $(pwd)/$PARACHAIN_CONFIG:/config \
  -v $(pwd)/$RELAYCHAIN_TYPES:/types \
  jacogr/polkadot-js-tools:latest api \
    --ws "ws://localhost:$((RELAY_PORT + 2))" \
    --sudo \
    --types "/types" \
    --seed "$SUDO_SEED" \
    --params /config \
    tx.parasSudoWrapper.sudoScheduleParaInitialize
        