---
mbip: 5
title: Introduce a gas-based storage limit
author: Alan Sapède (@crystalin)
status: Accepted
category: Core
created: 2023-05-11
---

## Simple Summary

Increases the used gas based on storage increase in transaction to deal wiuth storage congestion.

## Abstract

Introduce a new mechanism when executing a transaction that will increase the gas used when
a transaction increases the storate stage.

## Motivation

Moonbeam chain state needs to be sustainable for collators and archive nodes. With its current
fee mechanism, it doesn't account sufficiently for new storage data being added.

## Specification

A block will have a limit of 40_000 bytes of storage growth.

A transaction increasing the state by `X bytes` will consume `X * 15_000_000 / 40_000` additional
gas.

## Comments

Considering the baseFee is only increasing when the block has 25% or more gas consumed, it is
possible to `25% * 40kB * 2628000 blocks => 26.2GB`

### Example

Deploying a Smart Contract of 24kB using 3_300_000 gas would require:  
`24_000 * 15_000_000 / 40_000 + 3_300_000 => 12_300_000 gas`

Execution a transaction using 48_000 gas and adding 3 storage items (444 bytes) would require:  
`444 * 15_000_000 / 40_000 + 48_000 => 214_500 gas`

## Impact

Increase the gas being used in the block, even if the computational part stays the same.
This can lead to blocks being full because of the storage being used without any computation at all.

This will cause smart contracts which assume fixed gas limit on subcall to fail. Those are however
rare.
