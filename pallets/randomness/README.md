# Randomness Solutions Tradeoff Analysis

This pallet provides access to 2 sources of randomness:

1. The **local parachain VRF** is produced by collators per block. It signs the last block's VrfOutput as input to the VRF for this block.

2. The **BABE epoch randomness** is produced by the relay chain per relay chain epoch. It is based on **all the VRF produced** by the relay chain validators **during** a complete **epoch**.(~600 blocks on Kusama, ~2400 blocks on Polkadot). At the beginning of a new Epoch, those VRFs are **mixed together** and **hashed** in order to produce a **pseudo-random word**.

## CAP Theorem

The CAP Theorem says we can have at most 2 of the following properties: Consistency, Availability, and Partition Tolerance.

### Consistency

At any block, the random word is consistent between the nodes.

In practice it means that there **is only 1 possible random word generated** and so the node cannot choose between multiple ones.

### Example

Having multiple actors **sending a VRF proof** of the randomly generated word, and **revealing** their random words after all the proofs have been published. (the random word would be the hash of all the secrets)

By doing so, once the VRF proofs have all been published, there can only be 1 generated word.
_(This example however doesn't have the "Availability" property as actors can "fail" to reveal their secret preventing the random word to be available)_

### Availability

Every request to generate a random word receives a response.

In practice, it means a node **cannot withhold** any information **preventing** the random word to receive **a response**.

#### Example

Using the mandatory VRF of the current collator as a pseudo-random word makes it "available" as it is always present when the block is produced.

If the collator skips the block production, the next collator producing the block will be able to include its VRF to the block, allowing to provide the pseudo-random word.
_(This example however breaks the "consistency" because for the given block, there were the possibility of 2 different pseudo-random)_

### Partition Tolerance

The network continues to operate, even if an arbitrary number of nodes are failing.

In practice, it means that the randomness process cannot rely on a designated node but must, like the blockchain consensus, continue to work with a subset of the collators.

#### Example

Ex: Using the current block collator to produce the randomness output is partition tolerant.
If the current block collator fails to produce the block, the consensus will pick another collator allowing to produce the pseudo-random word.

## Breaking down in 2 categories

Because we can't get rid of the partition tolerance, we can only provide solutions that are compromising the Consistency or the Availability.

### Category 1: Availability Over Consistency

The solutions in category 1 provide a pseudo-random process that is guaranteed to provide a pseudo-random word but cannot ensure it hasn't been tampered before being revealed.

This is the case for the [Babe Epoch Randomness] which ensure each epoch provides a pseudo-random word but also allows the last validator of an epoch to known and pick 2 different pseudo-random words by skipping the block production (At the cost of a relay chain block reward)

This is also the case of the [local Relay VRF] which ensure that each parachain block will contain the pseudo-random word, but a collator can know and pick different pseudo-random words (by skipping the block production, or by choosing which relay block data to use once Asynchronous backing is there)

The [local Parachain VRF] is also of Category 1, as it guarantee that a collator will provide the VRF needed to know the pseudo-random word, but also allows the collator to know and pick a different pseudo-random word by skipping the block production (also at the cost of a parachain block rewards)

While those solutions are part of the same category, they have different trade-offs to guarantee an almost consistent pseudo-random.

### Category 2: Consistency over Availability

The solutions in this category provide a pseudo-random that cannot be tampered and that is unique but cannot provide the guarantee it will be always be possible to retrieve it.

This is the case of the [Mixed Delayed Secret] (not yet described), which will require collators to provide a VRF proof of locally generated secret, and to reveal it later once all the VRF proofs have been published. Such a solution will guarantee that, once the VRF proofs are published, it is impossible to provide a different pseudo-random word. It also guarantees that if at least 1 collator is a good actor, it will be impossible to know the pseudo-random word until all the secrets are revealed. However such solution cannot guarantee that a node will always be able to provide its secret (it can be lost, it can be attacked, or the node can be malicious and refuses to publish it).

## Local Parachain VRF

The local Parachain VRF randomness is based on the **VRF generated by the collator of the block** based on the **previous VRF output** and similarly to ChainLink requires the Smart Contract to provide a hardly predictable salt (a mix of the block parent hash and a user input for example).
At the beginning of each parachain block, the collator **must** produce a VRF and the request is fulfilled using the VRF at the fulfillment hashed with the salt given at the request time.

### Properties

- A new pseudo-random word must be generated at each block. It doesn't require more than 1 block delay
- The security resides in the improbability of a collator to predict which collator will author

### Risks

The **danger** comes from the possibility for a collator to **skip** the block production in order to have a different pseudo-random word generated by the next collator, at the cost of the block reward.
Another **danger** can exist if the salt being used by the Smart Contract has been used before, it is possible that if the collator at the fulfillment block is the same as before, the pseudo-random word will be known in advance.

### Asynchronous Backing

This solution is **not impacted** by asynchronous backing as all the material used as a seed is on the parachain and independent of the relay data.

## Babe Epoch Randomness

The Babe epoch randomness is based on **all the VRF produced** by the validators **during** a complete **epoch**.(~600 blocks on Kusama, ~2400 blocks on Polkadot)
At the beginning of a new Epoch, those VRFs are **mixed together** and **hashed** in order to produce a **pseudo-random word**.
To ensure each pseudo-random word generated during an epoch is different, the Smart Contract must provide a unique salt each time.

### Properties

- This randomness is totally **independent of the parachain**, preventing a malicious actor on the parachain to influence the randomness value.
- This randomness is **constant during a full epoch range** (~250 blocks on Kusama, ~2300 blocks on Polkadot) making it **resilient enough against censorship**. If a collator prevents fulfillment at a given block, another collator can fulfill it at the next block with the same random value.
- This randomness **requires** at last 1 epoch after the current epoch (**~1h30** on Kusama, **~6h** on Polkadot) to ensure the pseudo-random word cannot be predicted at the time of the request.

### Risks

The **danger** in this process comes from the knowledge that the **last validator** (Validator Y in the schema) has when producing the last block of an Epoch. The process being deterministic and all the material to generate the pseudo random word being known, the validator can decide to **skip producing the block** in order to not include its VRF, which would result in a different pseudo-random word.

Because epoch are time-based, if the block is skipped, there won't be any additional block produced for that epoch. So the last validator of the block knows both possible output:

1. When **producing the block** including its VRF => pseudo-random word **AAAA**
2. When **skipping the block** and using already known previous VRFs => pseudo-random word **BBBB**

The only **incentive** to prevent the validator from skipping the block is the **block rewards**. So the randomness value is only **economically safe if the value at stake is lower than a block reward**.

```sequence
note over Validator: Validator A
note over Relay: Epoch 1: Block #2399
Relay->Para: (Relay Block #2399)
note over Para: Block #111\nRequest Randomness (@Epoch 3)
note left of Relay: No knowledge of epoch 2 randomness\nexists yet
Validator->Relay: (Relay Block #2400)
note over Relay: Epoch 2: Block #2400\n(random epoch 1: 0xAAAAAA...)
note over Relay: .\n.\n.
note over Para: .\n.\n.
note over Validator: Validator X
Validator->Relay: Produces #4798\n(influences Epoch 2 Randomness\nbut doesn't know the result)
note over Validator: Validator Y
Validator->Relay: Produces #4799\n(knows/influences Epoch 2 Randomness)\ncan choose 0xBBBBBB... or 0xCCCCCC...
note over Relay: Epoch 3: Block #4800\n(random epoch 2: 0xBBBBBB...or 0xCCCCCC...)
Relay->Para: (Relay Block #4800)
note over Para: Block #222\nFulFill Randomness using\n0xBBBBBB...or 0xCCCCCC...
```

_In this schema, we can see that validator Y can decide the epoch 2 randomness by producing or skipping its block._

### Multiple slot leaders

Additionally, the Babe consensus can sometime allow multiple validator to produce a block at the same slot. If that is the last slot of an Epoch,the selected validators coordinate in order to decide which one is producing the block, offering the choice of even more pseudo-random words.

### Asynchronous Backing

This solution is **safe** even after the asynchronous backing is supported as the pseudo-random is not dependant on which relay block the parachain block is referencing.
A collator being able to choose the relay block on top of which it builds the parachain block will not influence the pseudo-random word.
