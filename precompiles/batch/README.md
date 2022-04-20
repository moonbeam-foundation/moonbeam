# Batch precompile

To perform subcalls this precompile relies on a new feature of the EVM implementation
that allows to returns bytecode, which is currently the only way to do subcalls from a
precompile.

## Process to get the precompile bytecode

1. Paste `Batch.sol` into Remix
2. Build and deploy
3. Debug transaction and go to last step.
4. Copy return value (the deployed bytecode) into a file (`bytecode.hex`)
5. Convert it to binary format using this command: `xxd -r -p bytecode.hex bytecode.bin`