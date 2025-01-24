---
mbip: 6
title: Externally recordable EVM metrics [rejected]
author(s): Telmo G. Michelena (@tgmichel)
status: Rejected
created: 2023-05-11
---

## Abstract

With the introduction of WeightV2 there are fundamental incompatibilities between the native EVM
gasometer and how parachains are required to provide state transition proofs.

## Motivation

Proof-of-validity (POV) size is a concept native to Polkadot parachains. In order to prove that the
state transition of a parachain block is valid, the relay chain needs a merkle proof: a partial
state containing only the required data to execute all the extrinsics included in the parachain
block.

Because there is a target relay block time and the POV has to be transferred between
collators and validators, the POV size must be restricted to a certain amount, currently measured
to be 5MB. That's why Moonbeam is soon to introduce POV metering, so POV consumption can be measured
within the EVM execution, correctly accounting for this block space metric and satisfy the relay
chain imposed limits since the introduction of Weight V2

See: https://forum.polkadot.network/t/weight-v2-discussion-and-updates.

## Goals

EVM gasometer tracks the consumption of available `Gas` across a transaction's execution. `Gas` is
the only metric the gasometer can fail upon. Currently we convert `WeightV1` - the polkadot native
metric all operations are benchmarked for - to `Gas`.

Substrate's `WeightV2` is multidimensional: each benchmarked operation will (still) have a
`ref_time` cost - aka execution time - and a POV size cost - how much data needs to be present in
the merkle proof to validate the operation.

This proposal augments the evm's `Backend` and `StackState` trait implementations' capabilities for
recording additional metrics, that is, POV recording for storage operations performed by the EVM in
addition to native gasometer recording.

Another fundamental difference is that unlike native _target_ `Gas`, the _external_ metrics capacity
is spent transaction wide not on a per-subcall level. This means any subcall can consume up to
all remaining given external capacity even if the native `Gas` capacity is way lower in proportion.

## Specification

We introduce a new constant `GasLimitPovSizeRatio` that will be multiplied over
the recorded proof size usage after the evm execution (A). We still capture the standard gasometer
used `Gas` during the evm execution (B).

`GasLimitPovSizeRatio` is estimated to be 1 byte per 4 `Gas` units.

### Example (S.1): effective `Gas`

A simplified example where a transfer's `to` is a 7kb contract.

```
GasLimitPovSizeRatio = 4;

GAS_LIMIT = 30_000;
POV_LIMIT = 7500;
// evm.transact() -> Succeed
```

The _effective_ used `Gas` will be `MAX(A,B)`, and it will be used to calculate the refund.

```
GAS_USED = 26_000;
// For example, a transfer is done to a big contract (bytecode size 7Kb.)
POV_SIZE_USED = 7000;
EFFECTIVE_GAS_USED = MAX(GAS_USED, POV_SIZE_USED * GasLimitPovSizeRatio); // 28_000
GAS_REFUND = GAS_LIMIT - EFFECTIVE_GAS_USED; // 2000
```

### Example (S.2): `OutOfGas`

A simplified example where a transfer's `to` is an 8kb contract, too big for its POV size to be paid
for using `GAS_LIMIT`.

```
GasLimitPovSizeRatio = 4;

GAS_LIMIT = 30_000;
POV_LIMIT = 7500;
// evm.transact() -> Fail(OutofGas)
```

This example shows one of the main differences introduced by this proposal: one can provide a valid
`GAS_LIMIT`, enough to pay for a transfer following Yellow Paper specification, but will `OutOfGas`
because the `POV_SIZE_USED` would be greater than POV_LIMIT.

## Impact

(S.2) represents the most notable difference between traditional EVM gasometer and Moonbeam's hybrid
multidimensional gasometer being proposed: one can OutOfGas if any configured metric is exhausted.

In (S.1) as refunds will be calculated over the highest recorded metric, this also introduces a
substantial difference on how refunds work in Moonbeam vs. Ethereum and, in some cases, might break
assumptions done purely on what the cost is supposed to be metered by the traditional gasometer when
successfully exiting the EVM:

- Prior to execution: the need of providing a `gas_limit` that is enough to pay for whatever the most
  used metric will be during the EVM execution.
- After the execution: the refund will be calculated over the most used metric on exiting the EVM.

## Security Considerations

This is a necessary security-related change in any EVM-compatible parachain. If not
introduced, it opens up for spam vectors like being able to abuse block space and even filling
full blocks worth of POV for a fraction of the cost.
