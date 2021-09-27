# Moonbeam Types Bundle

Exports npm package `moonbeam-types-bundle`, formated as per polkadot-js specification to use
with the app or the api.

## Exports

```
export const typesBundle = {
  spec: {
    moonbeam: moonbeamDefinitions,
    moonbeamDefinitions,
    moonbase: moonbeamDefinitions,
    moonriver: moonbeamDefinitions,
  },
} as OverrideBundleType;
```

`typesBundle` is of type OverrideBundleType to associate runtime names with correct definitions

`moonbeamDefinitions` is of types OverrideBundleDefinition and returns a different set of type for
each runtime version.

## Print Types

To print and save types JSON for a specific version run:
`ts-node generateJSON.ts <verison number>`

To print and save the latest:
`ts-node generateJSON.ts latest`
