## Overview

This sub-directory holds the saved diffs of runtime outputs produced by Parity's [subxt](https://github.com/paritytech/subxt) CLI tool.

The output contents is the difference between Metadata blobs served by two running nodes - i.e. what has changed between runtime releases.

## Example

`/moonbase/2800.txt` will contain the difference between `runtime-2800` and the previous release `runtime-2700`.

```sh
Pallets:
    ~ AuthorInherent
        Storage Entries:
            + InherentIncluded
    - CouncilCollective
    ~ EVM
    - LocalAssets
    + MoonbeamLazyMigrations
    ~ Multisig
        Calls:
            ~ as_multi
            ~ as_multi_threshold_1
# ... 
```

Where:

- `+` : Item added
- `-` : Item removed
- `~` : Item changed

For example, above you can see that AuthorInherent has changed, in that a new Storage has been added `InherentIncluded`. This can be verified by calling the chain state query `authorInherent.inherentIncluded()`.
Meanwhile, `LocalAssets` pallet has been removed, so you would expect references to storages, extrinsics, events; to be removed.

> [!NOTE]  
> Adding new pallets may not nessecarily correspond to new storages or extrinsics being added - althougth it often does.

## Resources

- [subxt cli readme](https://github.com/paritytech/subxt/tree/master/cli) - Some basic instructions in readme
- [subxt release docs](https://github.com/paritytech/subxt/releases) - Latest releases have usage guide for new CLI features
- [polkadot metadata explorer](https://wiki.polkadot.network/docs/metadata) - An interactive explorer of polkadot metadata
- [substrate docs](https://docs.polkadot.com/polkadot-protocol/basics/chain-data/#expose-runtime-information-as-metadata) - Some background on how metadata blobs get generated with which to serve when requested.
