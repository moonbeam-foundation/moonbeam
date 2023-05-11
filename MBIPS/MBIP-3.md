---
mbip: 3
title: Rent mechanism
author(s):
status: Draft
created: 2023-05-11
---

## Abstract



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

This proposal provides a solution for **[ISSUE-2] Storing a Smart Contract** and **[ISSUE-3] Storing data in a Smart Contract**.

This proposal does NOT provide a solution for **[ISSUE-1] Creating a new account**.

## Specification

### Logic

- Creating a Smart Contract (including CREATE/CREATE2) would require specifying a certain deposit.
   There would be a minimum deposit value defined by the community.
- When executing a transaction, a certain amount of that smart contract deposit 
   would get burnt based on how long since the last time it was used.
- When executing a transaction, if the deposit of the Smart Contract is under the required amount to pay the rent, the transaction gets reverted.
- Any address can deposit GLMR to a Smart Contract to extend the rent.

### Storage Items

- Add fields `last_used_block_number` and `deposit` to `AccountCodeMetadata` to keep track of the amount.

### Parameters

- **Burn rate**:
  - Suggested value: **0.001 GLMR / Bytes / Year**
  - Target growth cost: 1GB => `1,000,000,000 * 0.001 GLMR => 1,000,000 GLMR`. In order to go over the acceptable target, an attacker would need to spend 1M GLMR

### Example

With a rent fee of 0.01 GLMR / Byte / Year, and a minimal deposit of 25kB for 1 year, deploying a contract of 14kB would induce a deposit of `0.01 * 14000 => 140 GLMR`.
When this same contract gets used for the first time 1 month, it would burn `1/12 * 0.01 * 14000 => 11.6 GLMR` from the smart contract deposit.

### New Precompiled Smart Contract

1. `SmartContractManager`
	- `addDeposit(address, amount)` - Adds deposit of given amount to the given Smart Contract.

## Impact

Interacting with smart contracts will require additional steps to check (and possibly increase the deposit) if there's enough deposit for executing the transaction. This will affect most users and projects that interacts with smart contracts, and deviates from the Ethereum behavior.

## Security Considerations

- An attacker could spam storage of a smart contract making it unusable until someone else
increases the deposit

## Addition 1 - Destroy unused contract after some time

Additionally a clean-up process could be put in place when a contract doesn't have a deposit to pay for storage for a given amount of time (ex: 5 years), it can be destroyed. 

**This is dangerous and requires a lot of careful consideration. Questions like “what happens to the Tokens held by the contract?” need to be answered. Impacts of bridge contracts being destroyed and then rebuilt with a nonce of 0 (allowing replay attack) could be catastrophic.**

