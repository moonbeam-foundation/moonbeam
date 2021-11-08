# Moonbeam Types Bundle

Exports npm package `moonbeam-types-bundle`, formated as per polkadot-js specification to use
with the app or the api.

## ⚠️Warning: Types deprecation⚠️

Following runtime upgrade 900 (include substrate v0.9.11), types are now retrieved from the node, in
a **camelCase** format.

A **new version** has been released `moonbeam-types-bundle@2.0.0`.  

The default export `typesBundle` has **been removed** in order to avoid confusion.  

**2 new typesBundles** are available:

* `import { typesBundlePre900 } from "moonbeam-types-bundle"` to use the new naming convention (Which will be the default on PolkadotJS SDK)
* `import { typesBundleDeprecated } from "moonbeam-types-bundle"` to keep using old naming convention (This will break at runtime 1000)

### Breaking changes in typesBundlePre900

Those types are being changed:
```
  AssetRegistrarMetadata: {
    ...
    isFrozen: "bool", // was is_frozen
  },
  RewardInfo: {
    totalReward: "Balance", // was total_reward
    claimedReward: "Balance", // was claimed_reward
    contributedRelayAddresses: "Vec<RelayChainAccountId>", // was contributed_relay_addresses
  },
  Nominator2: {
    ...
    scheduledRevocationsCount: "u32", // was claimed_reward
    scheduledRevocationsTotal: "Balance", // was claimed_reward
  },
  ExitQ: {
    ...
    nominatorsLeaving: "Vec<AccountId>", // was nominators_leaving
    candidateSchedule: "Vec<(AccountId, RoundIndex)>", // was candidate_schedule
    nominatorSchedule: "Vec<(AccountId, Option<AccountId>, RoundIndex)>", // was nominator_schedule
  },
  Collator2: {
    ...
    topNominators: "Vec<Bond>", // was top_nominators
    bottomNominators: "Vec<Bond>", // was bottom_nominators
    totalCounted: "Balance", // was total_counted
    totalBacking: "Balance", // was total_backing
  }
```

## Using the new naming convention

```
import { typesBundlePre900 } from "moonbeam-types-bundle"

const api = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundlePre900,
});
```

### More time to transition

To allow more time for transition, you can use `typesBundleDeprecated` for runtime 900 which will
keep the same naming convention as before, but will disappear in runtime 1000.

**This solution is only to have more time to transition to `typesBundlePre900`**

# Development

`typesBundlePre900` is of type OverrideBundleType to associate runtime names with correct definitions

`moonbeamDefinitions` is of types OverrideBundleDefinition and returns a different set of type for
each runtime version.

## Print Types

To print and save types JSON for a specific version run:
`ts-node generateJSON.ts <verison number>`

To print and save the latest:
`ts-node generateJSON.ts latest`
