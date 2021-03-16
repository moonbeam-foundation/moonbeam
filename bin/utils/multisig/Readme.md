# MultiSig

Multisig is a tool that lets you generate a MultiSignature (Ecdsa, Sr25519, Ed25519) provided a mnemonic, data to sign and the algorithm you want to use.

## Build
From the Moonbeam root, run

`cargo build -p multisig --release`

## Example
From the Moonbeam root

The account field can come in hex or ss58 format

### Generate signature
`./target/release/multisig /target/release/multisig generate-signature --data "this is what I want to sigb" --private-key "bottom drive obey lake curtain smoke basket hold race lonely fit walk" --algorithm "ed25519"`

### Generate signer
`./target/release/multisig /target/release/multisig generate-signer --private-key "bottom drive obey lake curtain smoke basket hold race lonely fit walk" --algorithm "ed25519"`

### Encode
`./target/release/multisig encode-data --account 297a8594822ee492d01cb4eeb8dc21fe89f0af8a0a8bfb982f3d665bcb518703  --index 0 --old-balance 0 --value 20000000000000`

### Encode and sign
`./target/release/multisig encode-and-sign --account d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d  --index 3000 --old-balance 0 --value 20000000000000 --private-key "bottom drive obey lake curtain smoke basket hold race lonely fit walk" --algorithm "ed25519"`
## Interaction with the crowdloan module (verifier)S
First, generate a signer:

`./target/release/multisig /target/release/multisig generate-signer --private-key "bottom drive obey lake curtain smoke basket hold race lonely fit walk" --algorithm "ed25519"`

Use this signer when creating a fund in the crowdloan module, as 'verifier'

For contributions, we first need to encode aproppriately the data we need to sign. If account `account` wants to contribute to fund `index`, having contributed previously `last_balance` willing to contribute an amount `value`,

`../target/release/multisig encode-data --account 297a8594822ee492d01cb4eeb8dc21fe89f0af8a0a8bfb982f3d665bcb518703  --index 0 --old-balance 0 --value 20000000000000`

The output will be the data to be signed. We take the output and call

`./target/release/multisig /target/release/multisig generate-signature --data "output of encode-data as hex" --private-key "bottom drive obey lake curtain smoke basket hold race lonely fit walk" --algorithm "ed25519"`