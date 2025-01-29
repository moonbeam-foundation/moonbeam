# Moonbeam Types Bundle

Exports npm package `moonbeam-types-bundle`, formatted as per polkadot-js specification to use
with the app or the API.

## ⚠️Warning: Types deprecation⚠️

Following runtime upgrade 900 (include substrate v0.9.11), types are now retrieved from the node, in
a **camelCase** format

A **new version** has been released `moonbeam-types-bundle@2.0.0`.

The default export `typesBundle` has **been removed** to avoid confusion.  

**2 new typesBundles** are available:

* `import { typesBundlePre900 } from "moonbeam-types-bundle"` to use the new naming convention
* `import { typesBundleDeprecated } from "moonbeam-types-bundle"` to keep using old naming convention that isn't camelCase (This will break at runtime 1000)

The following package versions have been tested:

```
"@polkadot/api": "^6.9.1",
"moonbeam-types-bundle": "^2.0.1",
"typescript": "4.3.2"
```

Running the latest TypeScript version will not work.

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
    scheduledRevocationsCount: "u32", // was scheduled_revocations_count
    scheduledRevocationsTotal: "Balance", // was scheduled_revocations_total
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

## How to upgrade your tools/scripts using moonbeam-types-bundle

*(If your tool/script is not requesting past blocks, you can use the `typesBundleDeprecated` 
for now and fully remove it once the network has been upgraded to runtime 900, 
around Nov 18th 2021)*

The following package versions have been tested:

```
"@polkadot/api": "^6.9.1",
"moonbeam-types-bundle": "^2.0.1",
"typescript": "4.3.2"
```

Running the latest TypeScript version will not work.

Ultimately it is necessary to use the new type naming as the previous one won't be supported, but
you can import `typesBundleDeprecated` to buy yourself some time.

* moonbeam-types-bundle v1.x.x will break on runtime upgrade 900
(planned Thursday 18th November 2021 on Moonriver)
* moonbeam-types-bundle v2.x.x `typesBundleDeprecated` (using previous naming case) 
will **break on runtime 1000**
* **moonbeam-types-bundle v2.x.x** `typesBundlePre900` (using new naming case) 
will be **maintained**.

### Step 1: Install new package

```
npm install moonbeam-types-bundle@2
```

### Step 2: Change your import

```
import { typesBundlePre900 } from "moonbeam-types-bundle"

const api = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundlePre900,
});
```

### Step 3: Updates the object property names

For example:

```
console.log(collatorState2.unwrap().top_nominators);
                                    ^^^^^^^^^^^^^^
```

Becomes:

```
console.log(collatorState2.unwrap().topNominators);
                                    ^^^^^^^^^^^^^
```

All changes were listed [previously](#breaking-changes-in-typesbundlepre900).

## Support for ethereum encoded (MiXedCaSe) addresses

In runtime 900, addresses are represented in lower case by PolkadotJs SDK (this should be fixed
in runtime 1000).  
However it is possible to manually encode the address in Ethereum encoded (MiXedCaSe) format using:

```
import { ethereumEncode } from "@polkadot/util-crypto";

...
console.log(address);
// 0xb5af23c862df4ba2114276594a6ac851674cdf1e

console.log(ethereumEncode(address));
// 0xB5Af23c862dF4ba2114276594a6AC851674cDf1e
```

# Development

`typesBundlePre900` is of type OverrideBundleType to associate runtime names with correct definitions.

`moonbeamDefinitions` is of types OverrideBundleDefinition and returns a different set of types for
each runtime version.

## Print Types

To print and save types JSON for a specific version run:
`ts-node generateJSON.ts <version number>`

To print and save the latest:
`ts-node generateJSON.ts latest`
