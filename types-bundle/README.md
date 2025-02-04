# Moonbeam Types Bundle

Exports npm package `@moonbeam-network/types-bundle`, formatted as per polkadot-js specification to use
with the app or the API.

# Development

`typesBundlePre900` is of type OverrideBundleType to associate runtime names with correct definitions.

`moonbeamDefinitions` is of types OverrideBundleDefinition and returns a different set of types for
each runtime version.

## Print Types

To print and save types JSON for a specific version run:
`ts-node generateJSON.ts <version number>`

To print and save the latest:
`ts-node generateJSON.ts latest`
