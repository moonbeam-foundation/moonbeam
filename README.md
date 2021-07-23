## Starting the node

```
yarn run launch --parachain local --port-prefix 58 --parachain-chain alphanet-9.2-raw-specs.json
```

## Debugging

VsCode Debugger => `Debug Launch moonbase-0.9.2 (Linux)` (takes 10min to build on ferrari)

## Launch the test

```
cd tools
yarn
npx ts-node test-tracing.ts
```
