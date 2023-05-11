---
mbip: 1
title: Smart Contract Creation Deposit
author(s):
status: Draft
created: 2023-05-11
---

## Abstract

When deploying a Smart Contract a constant deposit will automatically be taken from the account sending the transaction. This deposit will be proportional to the full storage size.

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

This proposal provides a solution for **[ISSUE-2] Storing a Smart Contract**, and is compatible with [MBIP-2](MBIP-2.md).

This proposal does NOT provide a solution for [ISSUE-1] and [ISSUE-3]

## Specification

### Logic

- When deploying a Smart Contract (including using CREATE/CREATE2 operations), a constant deposit will automatically be taken from the account sending the transaction. This deposit will be proportional to the full storage size, which covers:
  1. The size of the stored contract. (number of bytes after calling the constructor)
  2. The overhead of storing a smart contract:
      - AccountCodes key (68 bytes)
      - System.Account: key (68 bytes) + value (80 bytes)

- When the Smart Contract gets destroyed, the deposit is restored to the depositor.

- When the sender doesn't have enough to provide for the deposit, the transaction gets reverted.

### Storage Items

- Add field `deposit` to `AccountCodeMetadata` to keep track of the amount and deposit owner.

### Parameters

- **Deposit ratio**:
  - Suggested initial value: **0.01 GLMR / Byte**
  - Would get re-evaluated by the community over time.
  - Target growth cost: `1GB => 1,000,000,000 * 0.01 GLMR => 10,000,000 GLMR`. In order to go over the acceptable target, an attacker would need to spend 10M GLMR

### Example:

Using the suggested ratio of 0.01 GLMR per byte, deploying the Wormhole bridge contract (24400 Bytes) would set a deposit of `(24.400 + 68 + 68 + 80)  * 0.01 => 246.16 GLMR`, and deploying a fast proxy (97 Bytes) would set a deposit of `(97 + 68 + 68 + 80)  * 0.01 => 3.13 GLMR`

## Impact

This deposit would break the assumption that a transaction cannot remove more than the “gasLimit * gasPrice” (or their EIP-1559 equivalent). In this proposal also the deposit could be “taken” (it would be reserved, but invisible in the Ethereum RPC) from the account.
(This is already the case with Precompiles. Ex: registering identity or a collator also reserves some amount from the sender)

This impacts mostly projects deploying smart contracts. Users however can also be impacted by smart contract functions using CREATE/CREATE2 which would force the user to put a deposit on the new smart contract being created.

As this deposit is not visible through Ethereum RPC, it will not be directly visible to the users who might think they paid a lot for the transaction.

## Security Considerations

A possible attack from a bad actor could be done by tricking a user to send a transaction to a smart contract which would trigger many CREATE to drain the user account into the deposit that the user won't be able to retrieve. (see [Addition 1](#addition-1-deposit-from-the-value) for a possible solution)


## Addition 1 - Deposit from the "Value"

Additionally, in order to make the amount of deposit required visible to the user, this one could be taken from the "value" field. This requires dapps to increase the amount of the value of their transaction deploying or using CREATE operations.

