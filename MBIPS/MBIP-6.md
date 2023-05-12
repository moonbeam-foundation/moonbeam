---
mbip: 6
title: External metric based refund
author(s): 
status: Draft
created: 2023-05-11
---

## Abstract

With the upcoming introduction of POV size metering for evm executions in Moonbeam, refunds will be
calculated over the highest recorded metric. This will become a fundamental difference between how
refunds are currently calculated in Ethereum vs. Moonbeam. 

## Motivation

Proof-of-validity (POV) size is a concept native to Polkadot parachains. In order to proof that the
state transition of a parachain block is valid, the relay chain needs a merkle proof: a partial
state containing only the required to data to execute all the extrinsics included in the block.

Because there is a target block time in the relay chain and the POV has to be transfered between
collators and validators, the POV size must be restricted to a certain amount, currently measured
to be 5MB. That's why Moonbeam is soon to introduce POV metering, so POV consumption can be measured
within the EVM execution, correctly accounting for this block space metric and satisfy the relay
chain imposed limits since the introduction of Weight V2

See: https://forum.polkadot.network/t/weight-v2-discussion-and-updates.

## Goals

Fundamentally, we are adding an additional dimension to EVM gas metering that needs to be charged
accordingly. In result this will produce:

- Prior to execution: the need of providing a gas_limit that is enough to pay for whaveter the most
used metric will be during the EVM execution.
- After the execution: the refund will be calculated over the most used metric on exiting the EVM.

## Specification

We introduce a new constant `GasLimitPovSizeRatio` that will be multiplied over
the recorded proof size usage after the evm execution (A). We still capture the standard gasometer
used gas during the evm execution (B). The _effective_ used gas will be `MAX(A,B)`, and it will be
used to calculate the refund.

```
BLOCK_GAS_LIMIT = 15_000_000;
POV_SIZE_LIMIT = 5_242_880;
GasLimitPovSizeRatio = BLOCK_GAS_LIMIT / POV_SIZE_LIMIT; // 2.86

// For example, a transfer is done to a big contract (bytecode size 10Kb.)
GAS_LIMIT = 30_000;
GAS_USED = 26_000;
POV_SIZE_USED = 10_000;

EFFECTIVE_GAS_USED = MAX(GAS_USED, POV_SIZE_USED * GasLimitPovSizeRatio); // 28_600
GAS_REFUND = GAS_LIMIT - EFFECTIVE_GAS_USED; // 1400
```

## Impact

This introduces a substantial difference on how refunds work in Moonbeam vs. Ethereum and, in some
cases, might break assumptions done purely on what the cost is supposed to be metered by the
traditional gasometer when successfully exiting the EVM.

In the example above, a big contract is read during a transaction, and the ratio of the size of the
contract is bigger than the computational equivalent cost of the transaction itself.  

## Security Considerations

This is a necessary security-related change in any EVM-compatible parachain. If not
introduced, it opens up for spam vectors like being able to abuse block space and even filling
full blocks worth of POV for a fraction of a cost.
