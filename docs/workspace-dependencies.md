## Workspace dependencies

## How dependencies features resolution works?

Features listed in the root-level `Cargo.toml` and in individual crates `Cargo.toml` are additive
(union of all sets of features). Also note that features may be enabled by other crates built to the
same target (wasm vs client have distinct feature sets however). Features listed in the root-level
`Cargo.toml` is enabled for all targets (hence the need for `default-features = false` for crates
used in a runtime crate.

Prefer adding features in individual crates, unless it is a security flag feature like
`forbid-evm-reentrancy` in which case it should be added to the root-level `Cargo.toml` to ensure it
is never disabled by mistake.

Note that `default-features = false` only has an effect inside the root-level Cargo.toml, and
should be added to any dependency that defaults to std if it is used in at least one runtime/wasm
crate.

## How to add a dependency?
1. Add `my-dependency = { workspace = true }` in your crate.
2. Look at the root-level `Cargo.toml` to see if the dependency is listed in it :
  - If it is not, add it in the proper section (Substrate/Frontier/etc) and subsection
    (wasm/client). Don't forget to add `default-features = false` if in wasm if necessary.
  - If it is, make sure it respects the std rule. If your crate is a runtime crate and the
    dependency was previously only used outside of the runtime, move the dependency in the "wasm"
    section and add `default-features = false`. It may require adding `features = ["std"]` in
    non-runtime crates `Cargo.toml`, however it is not necessary most of the time.
