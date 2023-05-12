---
mbip: 5
title: Introduce a gas-based storage limit
author(s):
status: Draft
created: 2023-05-11
---

## Abstract

A transaction would consume additional gas based on the storage being used and a defined ratio between the max gas per block and the max storage per block.

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

- When applying a transaction, the gas used is increased by the storage growth multiplied by a given storage/gas ratio
- If the transaction reached a point where the storage used is over the gasLimit / given ratio, it would revert with Out of Gas.


### Parameters

- **Storage/Gas Ratio**:
  - Suggested value: **40,000 bytes / 15,000,000 gas**
  - The lower the value the better, but some contract deployments require sufficient storage. A smart contract can be ~25kB but 1 deployment can deploy multiple contracts and also store storage items.
  - At 25% block fullness (which doesn’t increase the fees) this could lead to a maximum of  `25% * 40kB * 2628000 blocks => 26.2GB` per year.
  - At >25% block fullness, the baseFee increase would prevent reaching higher values


### Example

Using a ratio of 50kB storage for 15M gas
- Deploying a Smart Contract of 24kB using 3,300,000 gas would require `24,000 * 15,000,000 / 50,000 + 3,300,000 => 10,500,000 gas`
- Execution a transaction using 48,000 gas and adding 3 storage items (444 bytes) would require `444 * 15,000,000 / 50,000 + 48,000 => 181,200 gas`

## Impact

- This proposal will increase the gas being used in the block, even if the computational part stays the same. This can lead to blocks being full because of the storage being used without any computation at all. 
- This will cause contracts which estimate gas to forward on to fail. Such cases assume a fixed gas cost for some opcodes, which we are breaking.


## Security Considerations

No known security considerations
