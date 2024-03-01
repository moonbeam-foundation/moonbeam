# Functional testing for Moonbeam

> [!NOTE]\
> This folder contains a set of functional tests designed for Moonbeam network.
It is written in typescript, using the [Moonwall](https://moonsong-labs.github.io/moonwall/) framework.

## Test Categories

- `smoke`: Read-only tests that execute against live networks, primarily to verify state consistency and invariant conditions.
- `dev`: Tests that execute a single local dev node, using PolkadotJs / Ethers.js / Web3.js, to check the runtime and client in a relatively end-to-end manner.
- `chopsticks`: Tests that use the [Chopsticks](https://github.com/AcalaNetwork/chopsticks) framework to execute simulations of the state transition function against live state values - but in a sandboxed local environment.
- `para`: Tests that use the [ZombieNet](https://github.com/paritytech/zombienet) framework to verify Moonbeam in the context of a parachain connected to a relay chain, and other topologies.

## Installation

> [!NOTE]\
> PNPM is the package manager of choice for this repo, due to its superior handling of heavily nested dependencies.
There are [various](https://pnpm.io/installation) ways to install it, but perhaps the easiest is `sudo npm -g i pnpm`

Before running tests always install and update the package dependencies:

```bash
cd test
pnpm i 
```

## Usage Examples

Launch the CLI:

```bash
pnpm moonwall
```

Execute all dev tests:

```bash
pnpm moonwall test dev_moonbase
```

Execute a single test:

```bash
pnpm moonwall test dev_moonbase <test_case_id>
```

Execute a single test and keep node running:

```bash
pnpm moonwall run dev_moonbase <test_case_id>
```

Downloading the latest polkadot binary:

```bash
pnpm moonwall download polkadot latest
```

Running a chopsticks forked Moonbeam network:

```bash
pnpm moonwall run upgrade_moonbeam
```

Running a particular smoke test:

```bash
pnpm moonwall test smoke_moonbeam S100
```

Rename all prefixes for a suite (to keep them consistent)

```bash
pnpm moonwall derive <suite_root_dir> 
```

> [!NOTE]\
> For a full list of test environments and suites available, inspect the `moonwall.config.json` file.
Alternatively, use the CLI to browse networks and tests available.
