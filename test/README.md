# Functional testing for Moonbeam

## :construction: This folder is currently under W.I.P :construction:

> :information_source: This folder is meant to eventually replace `/tests/` when it is feature complete.

This folder contains a set of functional tests designed for Moonbeam network.

It is written in typescript, using the [Moonwall](https://github.com/Moonsong-Labs/moonwall) framework.

## Test Categories

- `smoke`: Read-only tests that execute against live networks, primarily to verify live state consistency.
- `dev`: Tests that execute a single local dev node, using PolkadotJs / Ethers.js / Web3.js, to check the runtime and client in a relatively end-to-end manner.
- `chopsticks`: Tests that use the [Chopsticks](https://github.com/AcalaNetwork/chopsticks) framework to execute simulations of the state transition function against live state values - but in a sandboxed local environment
- `zombienet`: Tests that use the [ZombieNet](https://github.com/paritytech/zombienet) framework to verify Moonbeam in the context of a parachain connected to a relay chain, and other topologies.

## Installation

PNPM is the package manager of choice for this repo, due to its superior handling of heavily nested dependencies.
There are [many](https://pnpm.io/installation) ways to install it, but perhaps the easiest is `sudo npm -g i pnpm`

Once

## Usage



