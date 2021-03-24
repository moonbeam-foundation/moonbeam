# Tools

_NB: this folder is yarn onlu_

## Moonbeam-Launch

Moonbeam-launch is a fork of https://github.com/paritytech/polkadot-launch that allows to run
a test script on multiple nodes.

To use it, follow instructions to build polkadot in the same repo : (from https://github.com/PureStake/polkadot-launch/tree/moonbeam-launch)

### Build Parachain

In the moonbeam repo, checkout to the desired commit of moonbeam and then run:

```
cargo build --release
./target/release/moonbase-alphanet --version
```

### Build Relaychain

First, in the moonbeam repo, look in the cargo.lock file to get the sha of the commit of the used polkadot version (ctrl+f `https://github.com/paritytech/polkadot`), or run any of the relay related scripts to see that sha logged.

Then, in the polkadot repo, cloned in the same repo as the moonbeam repo, run:

```
git checkout <commit sha>
cargo build --release --features=real-overseer
```

### Launch Script

And then run `yarn run moonbeam-launch` to start a network with `config_moonbeam.json`

If you want to run a simple test sending transactions to different addresses, run `yarn run moonbeam-test`

### Launch Custom Scripts

Before you run a custom script, run `yarn run build-moonbeam-launch` to install the correct dependency

If you want to run the staking test, run `ts-node test-staking.ts`

If you want to write your own custom test, use the start function from `polkadot-launch` :

`import { start } from "polkadot-launch";`

And then you can call it with the desired spec this way:

`await start("config_moonbeam_staking.json");`

### Change Config

Change the path in the config_moonbeam.json file to use polkadot in a different location.

### Generate Test Specs

To update the specs, run :
`./target/release/moonbeam build-spec --chain local >test.json`
