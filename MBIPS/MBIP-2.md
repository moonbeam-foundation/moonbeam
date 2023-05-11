---
mbip: 2
title: Storage Data Deposit
author(s):
status: Draft
created: 2023-05-11
---

## Abstract

When a transaction increases the storage size of the chain, a deposit will automatically be taken from the sender of the transaction proportionally to the increase of size. The opposite will happen when a transaction decreases the storage size.

## Motivation

Moonbeam is a Smart Contract chain, offering execution metered by gas.
This gas is associated with a dynamic price that allows control of the resources being used.
However such a control is not applied efficiently to the storage side of the chain. In order to stay compatible with ethereum and to allow simpler onboarding for projects, such control was kept as originally planned by Ethereum.
However, the storage has recently been bloated by some smart contracts and is currently vulnerable to long term storage attacks.

Currently there are 3 ways using the EVM to impact the storage size:
- **[ISSUE-1]** Creating a new account (this is also the case when deploying a new contract)
- **[ISSUE-2]** Storing a Smart Contract
- **[ISSUE-3]** Storing data in the Smart Contract

Storage growth must be limited somehow, but we have to agree on what limit should be used. Instead of thinking of it as a limit, I think we should think of an **acceptable target** that we could sustain forever and from there implement algorithms favoring a usage of the chain toward that target.

## Goals

This proposal provides a solution for **[ISSUE-3] Storing data in a Smart Contract** and optionally (see [Addition 1](#addition-1---including-same-mechanism-for-smart-contract-code)) for **[ISSUE-2] Storing a Smart Contract**. It is also compatible with [MBIP-1](MBIP-1.md).

This proposal does NOT provide a solution for **[ISSUE-1] Creating a new account**.

## Specification

### Logic

- When a transaction increases the storage size of the chain, a deposit will automatically be taken from the sender of the transaction proportionally to the increase of size

- When a transaction decreases the storage size, a part of the sender's current deposit is restored, proportionally to the decrease.

- When the sender does not have enough tokens to do the deposit, the transaction is reverted.


### Storage Items

- Add a named reserve to each account.

### Parameters

- **Deposit ratio**:
  - Suggested initial value: **0.001 GLMR / Bytes**
  - Target growth cost: 1GB => `1,000,000,000 * 0.001 GLMR => 1,000,000 GLMR`. In order to go over the acceptable target, an attacker would need to spend 1M GLMR


### Example

- Using the suggested ratio of 0.001 GLMR per byte, Minting an NFT that requires 3 storage items (116 bytes key * 32 bytes value) would induce a deposit of `(116 + 32) * 3 * 0.001 => 0.444 GLMR`

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
