# Zombienet Chain Specs

## Current setup

Since polkadot-sdk stable2512, the `polkadot` binary no longer ships
`polkadot-local` or `kusama-local` as built-in chain specs. Those moved to the
[polkadot-fellows/runtimes](https://github.com/polkadot-fellows/runtimes)
`chain-spec-generator` binary.

The committed `polkadot-local.json` and `kusama-local.json` specs were generated
from `polkadot-fellows/runtimes` commit `422623dbf` ("Update crates to SDK
2512-2 via psvm") using the `chain-spec-generator` with the `fast-runtime`
feature.

## Regenerating specs

If the specs need updating (e.g., after a polkadot-sdk upgrade), regenerate them
from the matching polkadot-fellows/runtimes version:

1. Clone [polkadot-fellows/runtimes](https://github.com/polkadot-fellows/runtimes.git)
2. Checkout the commit/tag that bumps to the matching SDK version
   (e.g., `422623dbf` for stable2512)
3. Build (only the relay runtimes needed):
   ```sh
   cargo build --release -p chain-spec-generator \
     --features polkadot,kusama,fast-runtime --no-default-features
   ```
4. Generate:
   ```sh
   ./target/release/chain-spec-generator polkadot-local > polkadot-local.json
   ./target/release/chain-spec-generator kusama-local > kusama-local.json
   ```
5. Copy the generated files into this directory (`zombienet/specs/`)
