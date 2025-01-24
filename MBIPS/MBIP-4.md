---
mbip: 4
title: Introduce a (hidden) storage base fee [rejected]
author: Alan Sapède (@crystalin)
status: Rejected
category: Core
created: 2023-05-11
---

## Simple Summary

A change in fee computation to deal with storage congestion.

## Abstract

Introduce a hidden Storage Base Fee based on previous blocks storage growth and that is applied
to the gas base fee.

## Motivation

Moonbeam chain state needs to be sustainable for collators and archive nodes. With its current
fee mechanism, it doesn't account sufficiently for new storage data being added.

In order to reduce the impact on the gas price, an additional mechanism is proposed.

## Specification

A new `storageBaseFee` is included and increased/decreased based on previous block state increase
and a given threshold.

The notion of `baseFee` is split into 2:

- `gasBaseFee` (hidden from the user) which represents the price per gas executed.
- `baseFee` (visible to the user) which represents the total price paid per gas consumed and
  includes the storage base fee into consideration.

A transaction with not enough `gasPrice` to compensate for the gas and storage will revert with
`gasPrice too low`.

### `storageBaseFee`

The storage base fee is the cost of storing 1 byte of data in the chain state. One straightforward
approach to dynamically adjusting this would be to use the same formula used in
[Substrate's pallet-transaction-payment](https://github.com/paritytech/substrate/blob/0046337664b221ff1072fb8f872f13a170babca9/frame/transaction-payment/src/lib.rs#L95)
which has been developed to achieve long-term average targets. Moonbeam uses this for its `base-fee`
implementation based on block weight, but it would be useful here as well.

Since the goal for storage is very long-term growth targets as opposed to short-term congestion
relief in the case of the base-fee, different parameters could be used to smooth it out:

```
given:
    s = previous block growth (cannot be negative)
    s' = ideal block growth = 5_000
    m = max block growth = 50_000
        diff = (s - s') / m
        v = 0.00001
        t1 = (v * diff)
        t2 = (v * diff)^2 / 2
    then:
    next_multiplier = prev_multiplier * (1 + t1 + t2)
```

### `gasBaseFee`

The `gasBaseFee` is the same as the `baseFee` defined before this MBIP. It simply represents the cost
of executing 1 gas.

### Transaction payment

Before this EIP, to compute the cost of a transaction (without tips), the [EIP-1559]
used `gasUsed * baseFee` with the `baseFee` being the same for each transaction of a block.

Using this EIP, the cost of the transaction is:

```
  txCost = (gasUsed * gasBaseFee) + (byteStored * storageBaseFee)
```

### `baseFee` (displayed)

To guarantee the user is not paying more than the gasPrice (or EIP-1559 equivalent) multiplied
by the gasLimit, we need to ensure the gasPrice will cover the gas baseFee
and also the storageBaseFee.

The gas being limited to 15_000_000 gas and the storage soft limit to 50_000 bytes, the final
baseFee needs to be proportionally adapted to each value (otherwise the baseFee would be very high
and would impact the user perception of the price paid)

```
blockStorageSoftLimit = 50_000 bytes
blockGasLimit = 15_000_000 gas
baseFee = gasBaseFee + (storageBaseFee * blockStorageSoftLimit / blockGasLimit)
```

/!\ A transaction requiring proportionally more storage than the provided gasLimit
will require to provide a higher gasPrice than the suggested baseFee.  
 The `blockStorageSoftLimit` is a soft limit, but a transaction
could go over that limit if it pays the correct gasPrice.

## Comments

A minimum storage fee of 0.0005 GLMR / Byte would lead to:  
`1GB => 1_000_000_000 * 0.0005 GLMR => 500_000 GLMR`

A `storageBlockExpectation` of `50_000 bytes` would be equivalent to:

| Gas Limit  | Storage Soft Limit (bytes) |
| ---------- | -------------------------- |
| 40_000     | 133                        |
| 200_000    | 666                        |
| 1_000_000  | 3_333                      |
| 5_000_000  | 16_666                     |
| 12_500_000 | 41_666                     |

## Examples

Using:

- `gasBaseFee`: 200 Gwei
- `storageBaseFee`: 500_000 Gwei (per byte)

The RPC would return a baseFee of `200 + (500_000 / 15_000_000 * 50_000) => 1_866 Gwei`

### 1. Transaction under the storage soft limit

Performing a transaction that uses `340_000 gas`
and increase the storage by `180 bytes` would cost: `200 * 340_000 + 500_000 * 180  => 0.158 GLMR`

By providing a baseFee of 1,866 Gwei in this example, the user is expected to pay `1_866 * 340_000 => 0.634 GLMR` and will pay `0.158 GLMR`.

### 2. Transaction over the storage soft limit

A transaction with a gasLimit of 100,000 and using 500 bytes of storage would fail with a gasPrice
== baseFee.
User is expecting to pay: `gasLimit (100_000) * 1_866 Gwei (baseFee) => 0.186 GLMR`
However the real cost of the transaction is: `gasUsed(100_000) * gasBaseFee (200) + storageUsed (500 bytes) * storageBaseFee (500_000) => 0.270 GLMR`

The user would have to provide a higher gasPrice (`>= 2_700 Gwei`) in order to get the transaction
included.

## Impact

This EIP increases the **displayed** `baseFee` to the users, which might give a false impression of
a higher price to pay.

It also **break** the assumption that: `paid fee == gas used * gas price`
(or `paid fee == gas used * base fee` for EIP-1559)

In the case of a transaction with a lot of storage increase, this will report `gasPrice too low`
even if the gasPrice provided is over the baseFee.
This would require to manually increase the gasPrice.
