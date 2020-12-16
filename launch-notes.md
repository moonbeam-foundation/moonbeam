# Build

## Compile Polkadot with the real overseer feature
```bash
git checkout d7257026
cargo build --release --features=real-overseer
./target/release/polkadot --version
```

## Compile Moonbeam
```bash
git checkout a4d257b0
cargo build --release
./target/release/moonbase-alphanet --version
```

# Launch Relay

## Validator Keys

Determining what session keys are needed requires looking at the code in a few places.
* Rococo Session Key Definition - https://github.com/paritytech/polkadot/blob/master/runtime/rococo/src/lib.rs#L148-L152
* Substrate Session key codes - https://github.com/paritytech/substrate/blob/master/primitives/core/src/crypto.rs#L1085
* Polkadot Session key codes - https://github.com/paritytech/polkadot/blob/master/primitives/src/v0.rs#L71
* Upcoming (but not currently used) Polkadot session keys - https://github.com/paritytech/polkadot/blob/master/primitives/src/v1.rs#L62-L83
* Observation: We don't need session keys for "acco" and "stak", but they were in the old validator script.

```bash
# Generate a key and note the mnemonic
./polkadot-d7257026-real-overseer key generate
```

For this write up I'll be using these example keys.

### Alfie
```bash
# SR25519 - most keys
$ ./polkadot-d7257026-real-overseer key inspect-key --scheme sr25519 "rail order express dynamic sketch tip mask double cave medal guitar between"
Secret phrase `rail order express dynamic sketch tip mask double cave medal guitar between` is account:
  Secret seed:      0xb7c888bf9f01da9a2cb5805d6d6ada744ed22d8a24aa41c509a26268ec701461
  Public key (hex): 0xc0671bd602df3430ea648f52baef5068f1082ce03e6563677255d163220ddd42
  Account ID:       0xc0671bd602df3430ea648f52baef5068f1082ce03e6563677255d163220ddd42
  SS58 Address:     5GQygSBSa7BjEeYiE41Q7uFNMUVfMEuU3bjtPmjZQBUEvgjn

# ED25519 - grandpa
$ ./polkadot-d7257026-real-overseer key inspect-key --scheme ed25519 "rail order express dynamic sketch tip mask double cave medal guitar between"
Secret phrase `rail order express dynamic sketch tip mask double cave medal guitar between` is account:
  Secret seed:      0xb7c888bf9f01da9a2cb5805d6d6ada744ed22d8a24aa41c509a26268ec701461
  Public key (hex): 0x9a35999189aeac73680dbccc89b18335a545f62045a54c9225105a428976cc16
  Account ID:       0x9a35999189aeac73680dbccc89b18335a545f62045a54c9225105a428976cc16
  SS58 Address:     5FYu9sxGFZ15SapDCUJiJy8JXFCDBLv56iLrWrEDeT6BfybQ

```

### Bet
```bash
# SR25519 - most keys
$ ./polkadot-d7257026-real-overseer key inspect-key --scheme sr25519 "planet ill puzzle mirror fog system admit genre subject dance aim limit"
Secret phrase `planet ill puzzle mirror fog system admit genre subject dance aim limit` is account:
  Secret seed:      0xa7854968eaa257ed5e87b3dc765feca8e7dadf5322b2d04e6ba60e080164da15
  Public key (hex): 0x80d42ee5b73818f4cf65d9a4cbe87c990c83af82c9baceb565d85d0f2a3e4807
  Account ID:       0x80d42ee5b73818f4cf65d9a4cbe87c990c83af82c9baceb565d85d0f2a3e4807
  SS58 Address:     5Eyd1zj8BSFaAt66y152neBFhZTaQ1TJNoxZYdYkXqqXRf1X

# ED25519 - grandpa
$ ./polkadot-d7257026-real-overseer key inspect-key --scheme ed25519 "planet ill puzzle mirror fog system admit genre subject dance aim limit"
Secret phrase `planet ill puzzle mirror fog system admit genre subject dance aim limit` is account:
  Secret seed:      0xa7854968eaa257ed5e87b3dc765feca8e7dadf5322b2d04e6ba60e080164da15
  Public key (hex): 0xbd8d705b2742bb237d662b0a60414c9d7cef8f9407f816dac0912cac6b933f13
  Account ID:       0xbd8d705b2742bb237d662b0a60414c9d7cef8f9407f816dac0912cac6b933f13
  SS58 Address:     5GMEvWtt8CyxRwZeRAuxDy2rmuzsZAuwCochbHCkQVv1Mfev
```

Insert Alfie's session keys
```bash
./polkadot-d7257026-real-overseer key insert --keystore-path ./alfie/chains/rococo_local_testnet/keystore --base-path alfie --suri "rail order express dynamic sketch tip mask double cave medal guitar between"  --key-type gran --scheme ed25519 && \
./polkadot-d7257026-real-overseer key insert --keystore-path ./alfie/chains/rococo_local_testnet/keystore --base-path alfie --suri "rail order express dynamic sketch tip mask double cave medal guitar between"  --key-type babe && \
./polkadot-d7257026-real-overseer key insert --keystore-path ./alfie/chains/rococo_local_testnet/keystore --base-path alfie --suri "rail order express dynamic sketch tip mask double cave medal guitar between"  --key-type imon && \
./polkadot-d7257026-real-overseer key insert --keystore-path ./alfie/chains/rococo_local_testnet/keystore --base-path alfie --suri "rail order express dynamic sketch tip mask double cave medal guitar between"  --key-type para && \
./polkadot-d7257026-real-overseer key insert --keystore-path ./alfie/chains/rococo_local_testnet/keystore --base-path alfie --suri "rail order express dynamic sketch tip mask double cave medal guitar between"  --key-type audi

#TODO Should we use different derivation paths for each keypair like Telmo recommended? Probably. But NOT for stash or controller. Session keys are HOT keys.
```

Repeat similarly for Bet
```bash
./polkadot-d7257026-real-overseer key insert --keystore-path ./bet/chains/rococo_local_testnet/keystore --base-path bet --suri "planet ill puzzle mirror fog system admit genre subject dance aim limit"  --key-type gran --scheme ed25519 && \
./polkadot-d7257026-real-overseer key insert --keystore-path ./bet/chains/rococo_local_testnet/keystore --base-path bet --suri "planet ill puzzle mirror fog system admit genre subject dance aim limit"  --key-type babe && \
./polkadot-d7257026-real-overseer key insert --keystore-path ./bet/chains/rococo_local_testnet/keystore --base-path bet --suri "planet ill puzzle mirror fog system admit genre subject dance aim limit"  --key-type imon && \
./polkadot-d7257026-real-overseer key insert --keystore-path ./bet/chains/rococo_local_testnet/keystore --base-path bet --suri "planet ill puzzle mirror fog system admit genre subject dance aim limit"  --key-type para && \
./polkadot-d7257026-real-overseer key insert --keystore-path ./bet/chains/rococo_local_testnet/keystore --base-path bet --suri "planet ill puzzle mirror fog system admit genre subject dance aim limit"  --key-type audi
```

`polkadot key insert` was unintuitive to me for several reasons. I've reported these in https://github.com/paritytech/polkadot/issues/2072
* Specifying `--chain` doesn't seem to have any effect.
* Specifying `--base-path` alone "works" but puts the keys in the wrong place.
* Sepcifying `--keystore-path` alone complains that `--base-path` is not specified.

## Chain Spec

Rococo local is known to throw the warning `💸 Chain does not have enough staking candidates to operate. Era Some(0)`. This is normal and harmless.

```bash
./polkadot-d7257026-real-overseer build-spec --chain rococo-local --disable-default-bootnode > rococo-local-d7257026-real-overseer.json
```

Insert the custom session keys like so in the chain spec. (TODO as I mentioned above, we should eventually use different offline keys for stash and controller.)

```json
"palletSession": {
  "keys": [
    [
      "5GQygSBSa7BjEeYiE41Q7uFNMUVfMEuU3bjtPmjZQBUEvgjn",
      "5GQygSBSa7BjEeYiE41Q7uFNMUVfMEuU3bjtPmjZQBUEvgjn",
      {
        "grandpa": "5FYu9sxGFZ15SapDCUJiJy8JXFCDBLv56iLrWrEDeT6BfybQ",
        "babe": "5GQygSBSa7BjEeYiE41Q7uFNMUVfMEuU3bjtPmjZQBUEvgjn",
        "im_online": "5GQygSBSa7BjEeYiE41Q7uFNMUVfMEuU3bjtPmjZQBUEvgjn",
        "parachain_validator": "5GQygSBSa7BjEeYiE41Q7uFNMUVfMEuU3bjtPmjZQBUEvgjn",
        "authority_discovery": "5GQygSBSa7BjEeYiE41Q7uFNMUVfMEuU3bjtPmjZQBUEvgjn"
      }
    ],
    [
      "5Eyd1zj8BSFaAt66y152neBFhZTaQ1TJNoxZYdYkXqqXRf1X",
      "5Eyd1zj8BSFaAt66y152neBFhZTaQ1TJNoxZYdYkXqqXRf1X",
      {
        "grandpa": "5GMEvWtt8CyxRwZeRAuxDy2rmuzsZAuwCochbHCkQVv1Mfev",
        "babe": "5Eyd1zj8BSFaAt66y152neBFhZTaQ1TJNoxZYdYkXqqXRf1X",
        "im_online": "5Eyd1zj8BSFaAt66y152neBFhZTaQ1TJNoxZYdYkXqqXRf1X",
        "parachain_validator": "5Eyd1zj8BSFaAt66y152neBFhZTaQ1TJNoxZYdYkXqqXRf1X",
        "authority_discovery": "5Eyd1zj8BSFaAt66y152neBFhZTaQ1TJNoxZYdYkXqqXRf1X"
      }
    ]
  ]
},
```

Finally, bake a raw spec

```bash
./polkadot-d7257026-real-overseer build-spec --chain rococo-local-d7257026-real-overseer.json --disable-default-bootnode --raw > rococo-local-d7257026-real-overseer-raw.json
```

## Validator Commands

This version of Polkadot is known to throw the warning `Ran out of free WASM instances`. This harmless, and is [issue #2070](https://github.com/paritytech/polkadot/issues/2070) and addressed by [PR #2069](https://github.com/paritytech/polkadot/pull/2069).

```bash
# Alfie
./polkadot-d7257026-real-overseer --chain rococo-local-d7257026-real-overseer-raw.json --validator --base-path ./alfie/

# Bet
./polkadot-d7257026-real-overseer --chain rococo-local-d7257026-real-overseer-raw.json --validator  --base-path ./bet/ --port 30334
```

# Launch Parachain

## Export genesis state and wasm

```bash
./target/release/moonbase-alphanet export-genesis-state --parachain-id 200 > genesis-state
./target/release/moonbase-alphanet export-genesis-wasm > genesis-wasm
```

## Launch Collators

Collators don't need session keys yet (They will once we have aura on the parachain). They only differ from each other in port numbers.

```bash
./target/release/moonbase-alphanet --collator --tmp --parachain-id 200 --port 40335 --ws-port 9946 -- --execution wasm --chain ../polkadot/rococo-local-d7257026-real-overseer-raw.json --port 30335
./target/release/moonbase-alphanet --collator --tmp --parachain-id 200 --port 40336 --ws-port 9947 -- --execution wasm --chain ../polkadot/rococo-local-d7257026-real-overseer-raw.json --port 30336
```

## Launch Parachain Full Nodes

Same as the Collators but no `--collator` flag (and different ports)

```bash
./target/release/moonbase-alphanet --tmp --parachain-id 200 --port 40337 --ws-port 9948 -- --execution wasm --chain ../polkadot/rococo-local-d7257026-real-overseer-raw.json --port 30337
./target/release/moonbase-alphanet --tmp --parachain-id 200 --port 40338 --ws-port 9949 -- --execution wasm --chain ../polkadot/rococo-local-d7257026-real-overseer-raw.json --port 30338
```

# Registration Transaction

Here we use the polkadot js tools docker image. The only tag available is `latest` which includes what we need as of 4 Dec 2020. For reference I'm using this image:
```
REPOSITORY                                          TAG                 IMAGE ID            CREATED             SIZE
jacogr/polkadot-js-tools                            latest              755149046430        29 hours ago        664MB
```

The runtime wasm is too large (~1MB) to be passed directly on the CLI. Instead we use the `--parmas` flag introduced in https://github.com/polkadot-js/tools/pull/91. It requires all the params to be in a single space-separated file, so we make that first.

```bash
# Create the file that holds all the parameters
echo -n "200 {\"genesis_head\":\"$(cat genesis-state)\",\"validation_code\":\"" > parachain-config && \
cat genesis-wasm  >> parachain-config && \
echo -n "\",\"parachain\":true}" >> parachain-config

# Submit the transaction
docker run --rm --network=host \
  -v $(pwd)/parachain-config:/config \
  jacogr/polkadot-js-tools:latest api \
    --ws "ws://localhost:9944" \
    --sudo \
    --seed "//Alice" \
    --params /config \
    tx.parasSudoWrapper.sudoScheduleParaInitialize
```
