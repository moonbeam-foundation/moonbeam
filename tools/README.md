# Tools

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

### Launch script

And then run `yarn run moonbeam-launch`

### Change Config

Change the path in the config_moonbeam.json file to use polkadot in a different location.
