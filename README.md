
# Moonbeam

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh
```

Build Wasm and native code:

```bash
cd node
cargo build --release
```
## Dev dependencies

Install subkey:
```bash
cargo install --force subkey --git https://github.com/paritytech/substrate
```


## Run

### Single node dev

TODO

### Multi-node local testnet

This will create 3 validator accounts - `//Armstrong`, `//Aldrin` and `//Collins` - on genesis. To be able to finalize blocks, Grandpa requires more than 2/3 of the validators to cast finality votes over a produced block, so we need to run all 3 nodes.

```bash
cd scripts/staging
./build-spec.sh
```
Two files - `spec.json` and `rawspec.json` - are created.

Next, execute `./run-node-armstrong.sh`, `./run-node-aldrin.sh` and `./run-node-collins.sh` in separate terminals.

> At this moment the chain *should* work as expected. However you will notice that blocks are not being produced. 
> 
> We are injecting `SessionKeys` on the `pallet_session` GenesisConfig. Theory says that should set the session keys on boot and start validating right away. Further investigation needs to be done to determine why it does not work and how to fix it. Proceed with the next steps to inject the keys manually.

Next, with all nodes running, manually inject the Public keys to the keystore using the `./set-keys.sh` helper script. Be aware that this assumes you are on a Unix machine. `subkey`, `curl`, `grep` and `cut` commands must be available in your environment to execute it.

> Confirmed by Joshy, **is not possible** to make effective the underlying `author_insertKey` rpc we just made without restarting the nodes. So we need to:

Restart all nodes.

They are now producing blocks and finalizing them.

Telemetry is enabled and can be seen here:
[https://telemetry.polkadot.io/#/Development](https://telemetry.polkadot.io/#/Development)


