---
mbip: 2
title: Storage Data Deposit [rejected]
author: Alan Sapède (@crystalin)
status: Rejected
category: Core
created: 2023-05-11
---

## Simple Summary

A deposit mechanism associated to EOA to deal with storage growth

## Abstract

Introduce a deposit assigned to EOA when they increase the state storage. The deposit increases
when the user sends a transaction storing additional data and decreases when the transaction
destroys some storage.

## Motivation

Moonbeam chain state needs to be sustainable for collators and archive nodes. With its current
fee mechanism, it doesn't account sufficiently for new storage data being added.

In order to avoid impacting the gas price, a distinct mechanism is proposed.

## Specification

Sending a transaction which stores additional state data (deploying a contract, adding an item in
a smart contract storage) **MUST** reserve additional tokens from the sender.

Sending a transaction which reduces the state data (destroying a contract, removing an item in
a smart contract storage) **MUST** unreserve additional tokens from the sender.
If the unreserve decreases the deposit amount to 0 or under, the deposit **MUST** be removed.

A sender without enough token to provide the deposit will get its transaction reverted.

Formula to compute the deposit amount:

```
DEPOSIT_RATIO = 0.001 GLMR / Bytes

deposit = (post_tx_storage_size - pre_tx_storage_size) * DEPOSIT_RATIO
```

### Comments

A deposit ratio of 0.001 GLMR / Byte would lead to:  
`1GB => 1,000,000,000 * 0.001 GLMR => 1,000,000 GLMR`

### Example

Minting an NFT that requires 3 storage items (116 bytes key + 32 bytes value) would lead to:  
`(116 + 32) * 3 * 0.001 => 0.444 GLMR`

Deploying an heavy contract (24_000 bytes code + 68 bytes overhead) would lead to:  
`(24_000 + 68) * 0.001 => 24.068 GLMR`

## Storage changes

A new "[named reserve](https://paritytech.github.io/polkadot-sdk/master/pallet_balances/struct.ReserveData.html)"
is associated to EOA when they need to deposit tokens for storage data.

## Functions

This proposal also adds the RPC endpoint `moon_getStorageDeposit` which accepts a given
`address` (AccountId20) and optionally a given block number or
the string "latest", "earliest" or "pending" and returns a `StorageDeposit` or null:

```
interface StorageDeposit {
  amount: U256;
}
```

## Impact

The deposit will not be visible in the transaction fields. This will break the assumption that a transaction cannot remove more than the "gasLimit \* gasPrice" (or their EIP-1559 equivalent).  
_(This is already the case with Precompiles. Ex: registering identity or a collator also reserves some amount from the sender)_

(see [Addition 1](#addition-1---deposit-from-the-value) for a possible solution)

This proposal impacts mostly the users as each one might get a deposit if they
interact with a smart contract that is increasing the state storage size.

## Security Considerations

A possible attack from a bad actor could be done by tricking a user to send a transaction to a smart contract which would trigger many CREATE to drain the user account into the deposit that the user won't be able to retrieve. (see [Addition 1](#addition-1---deposit-from-the-value) for a possible solution)

Additionally some people might gamble that the deposit storage ratio (GLMR/bytes) will increase in the future and start to store more data on-chain in the hope to "resell" that storage in the future. However it is very unlikely for the storage to increase as the cost of storage becomes cheaper over time.

## Addition 1 - Deposit from the "Value"

Instead of having the deposit taken from the user directly, the deposit could be taken from the
given "value" in the deploying transaction. This would make it visible to the user but would
require the application to compute the required value. It might also conflict with contract using
the value of the transaction for other matters.
