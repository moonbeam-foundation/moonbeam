---
mbip: 5
title: Introduce a gas-based storage limit
author: Alan Sapède (@crystalin)
status: Accepted
category: Core
created: 2023-05-11
---

## Simple Summary

Increases the gas used based on storage increase in transactions to deal with storage congestion.

## Abstract

Introduce a new mechanism for executing transactions that increases the gas used when
the transaction increases the storage state.

## Motivation

The Moonbeam chain state needs to be sustainable for collators and archive nodes. The current
fee mechanism does not adequately account for new storage data being added.

## Specification

A block will have a storage growth limit of 40,960 bytes.

A transaction that increases the state by `X` bytes will consume `X * 15,000,000 / 40,960` gas.

## Comments

Considering that the base fee only increases when the block has consumed 25% or more gas, it is
possible to have `25% * 40KB * 2,628,000 blocks => 25.06 GB`.

### Example

Deploying a smart contract with 20kB of bytecode stored on the chain would require:
`20KB * 15,000,000 / 40KB => 7,500,000 gas`

Executing a transaction that adds 3 storage items (116 bytes) would require:
`348 * 15,000,000 / 40,906 => 127,609 gas`

## Impact

Increase the gas used in the block, even if the computational part remains the same.
This can lead to blocks being full due to storage usage without any computation.

This may cause smart contracts that assume a fixed gas limit on subcalls to fail. However, such
cases are rare.
