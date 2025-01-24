---
mbip: 3
title: Rent mechanism [rejected]
author: Alan Sapède (@crystalin)
status: Rejected
category: Core
created: 2023-05-11
---

## Simple Summary

A renting mechanism using a deposit for smart contracts to deal with storage congestion.

## Abstract

Introduce a deposit assigned to a smart contract when being deployed. The deposit is consumed every
time the smart contract executes, proportionally to the elapsed time since it was executed.

## Motivation

Moonbeam chain state needs to be sustainable for collators and archive nodes. With its current
fee mechanism, it doesn't account sufficiently for new storage data being added.

In order to avoid impacting the gas price, a distinct mechanism is proposed.

## Specification

Users **CAN** send tokens to any address "rent deposit".

The destination address **MUST** have sufficient "rent deposit"
for a Smart Contract to be deployed. The sufficient amount corresponds to 1 year of rent
based on the size of the contract plus initial storage data.

Executing a transaction **MUST** burn part of the "rent deposit"
based on the elapsed number of blocks since the last time it was executed.

Executing a smart contract with not enough "rent deposit" **MUST** get reverted.

Formula to compute the "rent deposit" amount when deploying a smart contract:

```
YEAR = 5 * 60 * 24 * 365
BURN_RATIO = 0.0001 GLMR / Bytes / YEAR

token_burnt = (bytes of AccountCodes storage key (68) +
                 bytes of stored contract code (variable) +
                 (bytes of SystemAccount storage key (68) +
                  bytes of stored contract code (32))
                  * number of storage item accessed)
                * (current block number - last time used block number) * BURN_RATIO
```

### Comments

A deposit ratio of 0.00001 GLMR / Byte / Year would lead to:  
`1GB => 1_000_000_000 * 0.0001 GLMR => 100_000 GLMR / Year`

Executing a Smart contract within the same block would trigger the "rent deposit" to be burnt only
at the first execution.

The "rent deposit" can be refilled by anyone including the chain treasury through Governance.

### Example:

Deploying the complex contract (24_400 Bytes and 5 storage data) would require a deposit of:

```
bytes = 24_400 + 5 * (68 + 32)
      = 24_900
deposit_required = bytes * 0.0001 GLMR
                 = 2.49 GLMR
burn_rate = bytes * 0.0001 GLMR * elapsed_blocks / YEAR
          = 2.49 GLMR / year
          = 947 Gwei / block
```

Deploying a small contract with many storage data (1_000 bytes + 100 storage data) would require a deposit of:

```
bytes = 1_000 + 100 * (68 + 32)
      = 11_000
deposit_required = bytes * 0.0001 GLMR
                 = 1.1 GLMR
burn_rate = bytes * 0.0001 GLMR * elapsed_blocks / YEAR
          = 1.1 GLMR / year
          = 418.5 Gwei / block
```

Using a heavily used contract (10_000 bytes code size) using 10_000 storage data would burn:

```
bytes = 10_000 + 10_000 * (68 + 32)
      = 1_010_000
burn_rate = bytes * 0.0001 GLMR * elapsed_blocks / YEAR
          = 101 GLMR / year
          = 38432 Gwei / block
```

## Storage changes

New fields `last_used_block_number` and `rent_deposit` to `AccountCodeMetadata` structure
to keep track of the last block the smart contract was used and the amount of "rent deposit":

```
  last_used_block_number: u32
  rent_deposit: Balance
```

## Functions

This proposal also adds the RPC endpoint `moon_getRentDeposit` which accepts a given
`address` (AccountId20) and optionally a given block number or
the string "latest", "earliest" or "pending" and returns a `U256` or null

## New Precompiled Smart Contract

1. `SmartContractManager`
   - `addDeposit(address, amount)` - Adds deposit of a given amount to the given Smart Contract.
   - `lastBlockUsed(address) view returns (U256)` - Returns the last used block by the given Smart Contract.

## Impact

Smart contract "rent deposit" will need to be "refilled" after some time, depending on their growth
and previous deposit.

## Security Considerations

An attacker could spam storage of a smart contract making it unusable until someone else
increases the deposit. This is not sustainable in the long term because of the gas cost but can
be used when a contract has only a small "rent deposit" left, in order to burn it before maintainers
refill it. This would make the contract unusable until the "rent deposit" is refilled.

## Follow-up

A purge mechanism could be put in place when a contract doesn't have a deposit to pay for storage
for a given amount of time (ex: 5 years). The contract and the "rent deposit" would get destroyed.
This would require a lot of careful consideration. Questions like “what happens to the Tokens
held by the contract?” need to be answered.
Impacts of bridge contracts being destroyed and then rebuilt with a nonce of 0
(allowing replay attack) could be catastrophic.
