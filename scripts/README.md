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
./target/release/moonbeam --dev
```

It will creatte a new block each time a new transaction is received.
You can change this behavior by providing `--sealing 12000`
(to produce a block every 12s)

## Running complete local network

Moonbeam rely on `polkadot-launch` to provide a simple command to create a local network including
the relay and the parachain nodes.

The script [tools/launch.ts] contains a list of presets to execute the different possible networks.
Ex:

```
yarn launch --parachain moonbase-0.25.0
```

(More details in [tools/README.md])

## Running a parachain test

You can directly launch a parachain test with this script.
It takes care of getting the binary relay node and spawns 2 validators and 2 collators.

```bash
scripts/run-para-test-single.sh moonriver/test-balance-genesis.ts
```

# Development

The scripts are based on env variables.  
Most of them are initiated within [\_init_var.sh](_init_var.sh) script.
