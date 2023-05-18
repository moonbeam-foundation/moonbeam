---
mbip: 2
title: Storage Data Deposit
author: Alan Sapede (@crystalin)
status: Draft
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

## Storage changes

A new "[named reserve](https://paritytech.github.io/substrate/master/pallet_balances/struct.ReserveData.html)"
is associated to EOA when they need to deposit tokens for storage data.

## Functions

This proposal also adds the RPC endpoint `moon_getStorageDeposit` which accepts a given
`address` (AccountId20) and optionally a given block number or 
the string "latest", "earliest" or "pending" and returns a `CodeDeposit` or null:

```
interface StorageDeposit {
  amount: U256;
}
```

### Comments

A deposit ratio of 0.001 GLMR / Byte would lead to:  
`1GB => 1,000,000,000 * 0.001 GLMR => 1,000,000 GLMR`


### Example

Minting an NFT that requires 3 storage items (116 bytes key + 32 bytes value) would lead to:  
`(116 + 32) * 3 * 0.001 => 0.444 GLMR`

Deploying an heavy contract (24_000 bytes code + 68 bytes overhead) would lead to:  
`(24_000 + 68) * 0.001 => 24.068 GLMR`









## Impact

Introducing a mandatory deposit when increasing the storage might surprise a user when 
sending a transaction as this deposit would not appear in the Ethereum wallets nor in the
Ethereum block. (see [Addition 3](#addition-3---deposit-from-the-value) for a possible solution)

This deposit would break the assumption that a transaction cannot remove more than the “gasLimit * gasPrice” (or their EIP-1559 equivalent). In this proposal also the deposit could be “taken” (it would be reserved, but invisible in the Ethereum RPC) from the account.
(This is already the case with Precompiles. Ex: registering identity or a collator also reserves some amount from the sender)

This impacts most projects and users that interacts with smart contracts.

## Security Considerations

- A possible attack from a bad actor could be done by tricking a user to send a transaction to a smart contract. When checking the transaction, the user would see only X GLMRs being transferred but the smart contract could generate a huge amount of storage data and force the whole account into a deposit that the user cannot retrieve. (see [Addition 3](#addition-3---deposit-from-the-value) for a possible solution)
- It can happen that a sender is able to free more space than he has deposited. In this case, the deposit of the sender is reduced to 0.
- It is possible (and very likely) that a sender is putting a deposit for some storage that will be freed later by another user. If you think of it as a deposit for a specific storage data, it might seem "unfair", but the deposit should considered as storage that you "increased"



## Addition 1 - Including same mechanism for Smart Contract Code

It is possible to apply the deposit when deploying a smart contract (including CREATE/CREATE2) which would also solve **[ISSUE-2] Storing a Smart Contract**.

The deployer would see his GLMR deposited when performing the creation, at the same deposit ratio as the one for Storage Data.

## Addition 2 - Making the deposit dynamic

Additionally, this deposit could be dynamic, like the gas fee, requiring to deposit more when the storage growth fast and less when the growth slows down.
To avoid people betting on the Storage price going up, reducing the refunded part to a percentage would be necessary (To be investigated if this would be enough)

### Example
With a target of 1GB/Year, this would be something around ~400 bytes/block. So if a block increases the storage by more than 400, it would increase it, otherwise it would decrease it to a given minimal value.

## Addition 3 - Deposit from the "Value"

Similar to [Addition 1](#addition-1---including-same-mechanism-for-smart-contract-code), in order to make the amount of deposit required visible to the user, this one could be taken from the "value" field. This requires dapps to increase the amount of the value of their transaction, which might be incompatible with some use cases (to be investigated)
