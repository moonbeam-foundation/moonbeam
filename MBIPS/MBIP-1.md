---
mbip: 1
title: Smart Contract Creation Deposit [rejected]
author: Alan Sapède (@crystalin)
status: Rejected
category: Core
created: 2023-05-11
---

## Simple Summary

A deposit mechanism for smart contracts to deal with storage congestion.

## Abstract

Introduce a deposit assigned to a smart contract when being deployed. The deposit is "reserved" from the account sending the transaction and proportional to the storage size of the deployed
smart contract.

## Motivation

Moonbeam chain state needs to be sustainable for collators and archive nodes. With its current
fee mechanism, it doesn't account sufficiently for new storage data being added.

In order to avoid impacting the gas price, a distinct mechanism is proposed.

## Specification

Deploying a smart contract (including using CREATE/CREATE2 operations) **MUST** reserve a
deposit from the sender.

Destroying a smart contract **MUST** restore the deposit to the original depositor.

A sender without enough token to provide the deposit will get its transaction reverted.

Formula to compute the deposit amount when deploying a smart contract:

```
DEPOSIT_RATIO = 0.01 GLMR / Byte

deposit = (bytes of AccountCodes storage key (68) +
           bytes of stored contract code (variable) +
           bytes of SystemAccount storage key (68) +
           bytes of SystemAccount value (80)) * DEPOSIT_RATIO
```

### Comments

A deposit ratio of 0.01 GLMR / Byte would lead to:  
`1GB => 1_000_000_000 * 0.01 GLMR => 10_000_000 GLMR`

### Example:

Deploying the complex contract (24400 Bytes) would require a deposit of:  
`(24_400 + 68 + 68 + 80)  * 0.01 => 246.16 GLMR`,

Deploying a fast proxy (97 Bytes) would require a deposit of:  
`(97 + 68 + 68 + 80)  * 0.01 => 3.13 GLMR`

## Storage changes

A new field `deposit` is added to the `AccountCodeMetadata` structure to keep track of the amount
and the owner of the deposit:

```
  deposit: {
    owner: AccountId20,
    amount: Balance
  }
```

## Functions

This proposal also adds the RPC endpoint `moon_getCodeDeposit` which accepts a given
`address` (AccountId20) and optionally a given block number or
the string "latest", "earliest" or "pending" and returns a `CodeDeposit` or null:

```
interface CodeDeposit {
  owner: AccountId20;
  amount: U256;
}
```

## Impact

The deposit will not be visible in the transaction fields.
This will break the assumption that a transaction cannot remove more than
the "gasLimit \* gasPrice" (or their EIP-1559 equivalent).  
_(This is already the case with Precompiles. Ex: registering identity or a collator also reserves some amount from the sender)_

This proposal impacts mostly projects deploying smart contracts. Users however can also be impacted by smart contract functions using CREATE/CREATE2 which would force the user to put a deposit on the new smart contract being created.

## Security Considerations

A possible attack from a bad actor could be done by tricking a user to send a transaction to a smart contract which would trigger many CREATE to drain the user account into the deposit that the user won't be able to retrieve. (see [Addition 1](#addition-1---deposit-from-the-value) for a possible solution)

## Addition 1 - Deposit from the "Value"

Instead of having the deposit taken from the user directly, the deposit could be taken from the
given "value" in the deploying transaction. This would make it visible to the user but would
require the application to compute the required value. It might also conflict with contract using
the value of the transaction for other matters.

## Follow-up

A purge mechanism could be implemented after enough time to destroy the smart contract that don't
have a deposit associated (those created before this proposal is enacted).
This would allow to reduce the state storage size significantly.

It would require a new precompile function allowing anyone to deposit for any smart contract. This
would allow to keep smart contracts already deployed that are considered useful.
