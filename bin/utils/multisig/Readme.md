# MultiSig

Multisig is a tool that lets you generate a MultiSignature (Ecdsa, Sr25519, Ed25519) provided a mnemonic, data to sign and the algorithm you want to use.

## Build
From the Moonbeam root, run

`cargo build -p multisig --release`

## Example
From the Moonbeam root

`./target/release/multisig /target/release/multisig generate-signature --data "this is what I want to sigb" --private-key "bottom drive obey lake curtain smoke basket hold race lonely fit walk" --algorithm "ed25519"`
