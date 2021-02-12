# Scripts

This folder contains a list of script useful to develop/test the Moonbeam Node

## Requirements

For running nodes, you need to have **docker** running.
It is used to run the polkadot relay node (also needed to generate the relay specs for the parachain)

!!All the commands are to be executed from the repository root folder.!!

The node scripts are based on the USER_PORT env variable to set their
ports, following this strategy:  
(for USER_PORT=33000)

```
Standalone Nodes: 33[5-8]XX (supports 3 standalone nodes)
P2P: 34[5-8]42
RPC: 34[5-8]43
WS: 34[5-8]44
```

(so your first standalone node will have RPC at 33443)

```
Relay Nodes: 33[0-2]XX (supports 3 relay nodes)
P2P: 33[0-9]42
RPC: 33[0-9]43
WS: 33[0-9]44
```

(so your first relay node will have RPC at 33043)

```
Parachain Nodes: 34[0-9]XX (supports 9 parachain nodes)
P2P: 34[0-9]52
RPC: 34[0-9]53
WS: 34[0-9]54
```

(so your first parachain node will have RPC at 34053)

### Setting your USER_PORT

If you are running this on a shared remote computer, it is highly suggested to change the USER_PORT.

In your `~/.bashrc`, add at the end:

```
export USER_PORT=<XX000>
```

(replace XX by any digit that is not taken by someone else)

### Building the nodes

```bash
cargo build --release
```

# Standalone nodes

The standalone nodes are made to be executed without explicitly supplied specs.  
They also don't require any runtime wasm file or genesis state.

```bash
scripts/run-moonbase-dev.sh
```

# Alphanet local nodes

The alphanet nodes will run on a rococo-local relay, preventing them from connecting to the real alphanet.
It requires having relay nodes (at least 2) and parachain nodes (at least 1).
Those require sharing many files (specs, runtime wasm, genesis state).

The following steps will guide you through the generation of those files.

## Generating the relay specs

```bash
scripts/generate-relay-specs.sh
```

The script downloads `purestake/moonbase-relay-testnet:$POLKADOT_VERSION` docker image and execute the build-spec.
It also relies on the `rococo-local` for the specs.

## Generating the parachain specs

```bash
scripts/generate-parachain-specs.sh
```

The script executes (by default) `target/release/moonbeam` `build-spec`
It also relies on the [../specs/alphanet/parachain-specs-template.json] for the specs template.
The files generated are (by default) stored in `build/alphanet/parachain-specs-[plain,raw].json`

It also generates the `build/alphanet/runtime.wasm` and `build/alphanet/genesis.txt`

## Running Relay nodes

You can run up to 3 relay chain validators with this script. We use the `purestake/moonbase-relay-testnet` docker image for validators. Currently this image is manually published from commit (TODO), but this will change in the future.
Each node will get its key inserted prior to running the node.

```bash
scripts/run-alphanet-relay.sh
```

## Running Parachain nodes

You can run up to 3 relay nodes with this script

```bash
scripts/run-alphanet-parachain.sh
```

# Development

The scripts are based on env variables.  
Most of them are initiated within [\_init_var.sh](_init_var.sh) script.
