---
mbip: 4
title: Introduce a (hidden) storage base fee
author(s):
status: Draft
created: 2023-05-11
---

## Abstract

A new Storage Base Fee is included and will increase/decrease based on previous block storage size and a given threshold.

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

- A new Storage Base Fee is included and increase/decrease based on previous block storage size and a given threshold.
- When computing the gas price for a transaction, in also include the storage fee into computation based on how much storage has been increased/decreased

#### Ethereum 

To guarantee the user is not paying more than the gasPrice (or EIP-1559 equivalent) multiplied by the gasLimit, we need to ensure the gasPrice will cover the gas baseFee and also the storage baseFee.

Solution 1:
 - Make the RPC return the baseFee as `gasBaseFee + storageBaseFee`. However, because the storageBaseFee is a lot higher than the gasBaseFee, it would make the return baseFee very high which might scare the user.
- `baseFee == gasBaseFee + storageBaseFee`

Solution 2:
- Assume a transaction max storage target so that we can computed a ratio of the gas limit and the targeted max storage:
- `baseFee == gasBaseFee + storageBaseFee * targetMaxTransactionStorage / maxTransactionGas`
- This allows to reduce the final baseFee to be closer to the gasBaseFee
- In the case of a transaction requiring more than the target, if the gasPrice given is not enough, the transaction would fail (“Gas price lower than required”) and would require the user to increase its gasPrice manually.

### Parameters

- **Storage Base Fee**:
  - Suggested value: **0.0005 GLMR / bytes**
  - Total cost of 1GB: 500,000 GLMR

- **Targeted max transaction storage**:
  - Remark: This is not a limit, a transaction can go over this storage value, but it will require setting the gasPrice manually over the suggested baseFee.
  - Suggested value: **50,000 bytes**
  - The higher the value, the more the baseFee is “visibly” high (the cost itself doesn’t change) but the less likely a small transaction is going to fail because of a high storage/gas ratio.
  - This value impacts mostly small transactions, which might consume low amounts of gas but higher storage. Ex: A Tx using 200,000 gas would fail if it uses more than 666 Bytes

### Example

Using:
* `gasBaseFee`: 200 Gwei
* Max transaction gas: 15,000,000
* `storageBaseFee`: 500,000 Gwei (per Byte)
* Targeted max transaction storage: 50,000 (bytes)

The RPC would return a baseFee of `200 + (500,000 / 15,000,000 * 50,000) => 1,866 Gwei` 

Performing a transaction that uses 340,000 gas and increase the storage by 180 bytes would cost: `200 * 340,000 + 500,000 * 180  => 0.158 GLMR`

By providing a baseFee of 1,866 Gwei in this example, the user is expected to pay `1866 * 340,000 => 0.634 GLMR` and will pay `0.158 GLMR`. 

However a transaction with a gasLimit of 100,000 and using 500 bytes of storage would fail: User willing to pay: `gasLimit (100,000) * 1,866 (baseFee) => 0.186 GLMR`

Cost of the tx: `gasUsed(100,000) * gasBaseFee (200) + storageUsed (500 bytes) * storageBaseFee (500,000) => 0.270 GLM

## Impact

This increases the **displayed** `baseFee` to the users, which might scare them.
It also **break** the assumption that: `paid fee == gas used * gasPrice`

In the case of a transaction with a lot of storage increase, this will report `gasPrice too low` even if the gasPrice provided is over the baseFee. This would require manually increasing the gasPrice.


## Security Considerations

No known security considerations
