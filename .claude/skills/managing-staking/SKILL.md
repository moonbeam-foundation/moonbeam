---
name: managing-staking
description: Manages Moonbeam's parachain staking system including collator selection, delegation, and rewards. Use when modifying staking parameters, debugging delegation issues, implementing staking features, understanding collator selection, or working with staking rewards.
---

# Parachain Staking

## Contents
- [Staking Architecture](#staking-architecture)
- [Common Operations](#common-operations)
- [Staking Parameters](#staking-parameters)
- [Reward Distribution](#reward-distribution)
- [Testing Staking](#testing-staking)
- [Debugging Staking Issues](#debugging-staking-issues)

## Staking Architecture

### Core Components

| Component  | Location                         | Purpose            |
|------------|----------------------------------|--------------------|
| Pallet     | `pallets/parachain-staking/`     | Core staking logic |
| Precompile | `precompiles/parachain-staking/` | EVM interface      |
| Tests      | `test/suites/*/test-staking/`    | Integration tests  |

### Key Concepts

- **Collators**: Block producers that stake tokens
- **Delegators**: Token holders who delegate to collators
- **Rounds**: Fixed-length periods for reward distribution
- **Commission**: Percentage collators take from block rewards

### Storage Layout

```rust
// Key storage items in pallet-parachain-staking
pub type CollatorCommission<T> = StorageValue<_, Perbill>;
pub type TotalSelected<T> = StorageValue<_, u32>;
pub type Round<T> = StorageValue<_, RoundInfo>;

pub type CandidateInfo<T> = StorageMap<_, Twox64Concat, AccountId, CandidateMetadata>;
pub type DelegatorState<T> = StorageMap<_, Twox64Concat, AccountId, Delegator>;
pub type TopDelegations<T> = StorageMap<_, Twox64Concat, AccountId, Delegations>;
pub type BottomDelegations<T> = StorageMap<_, Twox64Concat, AccountId, Delegations>;

pub type AtStake<T> = StorageDoubleMap<_, Twox64Concat, RoundIndex, Twox64Concat, AccountId, CollatorSnapshot>;
pub type DelayedPayouts<T> = StorageMap<_, Twox64Concat, RoundIndex, DelayedPayout>;
```

## Common Operations

### Query Staking State

```typescript
// Get collator info
const candidateInfo = await api.query.parachainStaking.candidateInfo(collatorAddress);

// Get delegator state
const delegatorState = await api.query.parachainStaking.delegatorState(delegatorAddress);

// Get current round
const round = await api.query.parachainStaking.round();

// Get selected candidates for current round
const selectedCandidates = await api.query.parachainStaking.selectedCandidates();
```

### Delegate via Precompile

```solidity
// IParachainStaking interface
interface IParachainStaking {
    function delegate(
        address candidate,
        uint256 amount,
        uint256 candidateDelegationCount,
        uint256 delegatorDelegationCount
    ) external;

    function delegatorBondMore(address candidate, uint256 more) external;

    function scheduleDelegatorBondLess(address candidate, uint256 less) external;

    function executeDelegationRequest(address delegator, address candidate) external;

    function cancelDelegationRequest(address candidate) external;
}
```

### Delegate via Substrate

```rust
// Join as delegator
pallet_parachain_staking::Call::delegate {
    candidate: collator_account,
    amount: 1_000_000_000_000_000_000, // 1 GLMR
    candidate_delegation_count: 100,
    delegation_count: 10,
}

// Bond more to existing delegation
pallet_parachain_staking::Call::delegator_bond_more {
    candidate: collator_account,
    more: 500_000_000_000_000_000, // 0.5 GLMR
}
```

## Staking Parameters

### Network-Specific Configuration

| Parameter          | Moonbeam    | Moonriver   | Moonbase   |
|--------------------|-------------|-------------|------------|
| Min Collator Stake | 1M GLMR     | 10K MOVR    | 1K DEV     |
| Min Delegation     | 50 GLMR     | 5 MOVR      | 1 DEV      |
| Max Delegations    | 100         | 100         | 100        |
| Round Length       | 1800 blocks | 1800 blocks | 300 blocks |
| Reward Delay       | 2 rounds    | 2 rounds    | 2 rounds   |

### Runtime Configuration

```rust
// runtime/moonbase/lib.rs
parameter_types! {
    pub const MinCollatorStk: u128 = 1_000 * UNIT;
    pub const MinDelegation: u128 = 1 * UNIT;
    pub const MaxTopDelegationsPerCandidate: u32 = 300;
    pub const MaxBottomDelegationsPerCandidate: u32 = 50;
    pub const MaxDelegationsPerDelegator: u32 = 100;
    pub const DefaultBlocksPerRound: u32 = 300;
    pub const RewardPaymentDelay: u32 = 2;
}

impl pallet_parachain_staking::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
    type MinBlocksPerRound = ConstU32<10>;
    type DefaultBlocksPerRound = DefaultBlocksPerRound;
    type RewardPaymentDelay = RewardPaymentDelay;
    type MinSelectedCandidates = ConstU32<8>;
    type MaxTopDelegationsPerCandidate = MaxTopDelegationsPerCandidate;
    type MaxBottomDelegationsPerCandidate = MaxBottomDelegationsPerCandidate;
    type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
    type MinCollatorStk = MinCollatorStk;
    type MinCandidateStk = MinCollatorStk;
    type MinDelegation = MinDelegation;
    type WeightInfo = pallet_parachain_staking::weights::SubstrateWeight<Runtime>;
    // ...
}
```

## Reward Distribution

### Round Lifecycle

```
Round N Start
    │
    ▼
Block Production (collators earn points)
    │
    ▼
Round N End
    │
    ▼
Round N+1 Start
    │
    ▼
Round N-1 Rewards Distributed (after delay)
```

### Reward Calculation

```rust
// Simplified reward calculation
fn calculate_rewards(
    total_reward: Balance,
    collator: &CollatorSnapshot,
) -> (Balance, Vec<(AccountId, Balance)>) {
    // Collator gets commission
    let commission = total_reward * collator.commission;
    let delegator_pool = total_reward - commission;

    // Self-bond portion
    let collator_self_reward = delegator_pool * collator.bond / collator.total;

    // Delegator portions
    let delegator_rewards: Vec<_> = collator.delegations.iter()
        .map(|d| {
            let share = delegator_pool * d.amount / collator.total;
            (d.owner.clone(), share)
        })
        .collect();

    (commission + collator_self_reward, delegator_rewards)
}
```

## Testing Staking

### Unit Tests

```rust
// pallets/parachain-staking/src/tests.rs
#[test]
fn delegate_works() {
    ExtBuilder::default()
        .with_balances(vec![(1, 1000), (2, 1000)])
        .with_candidates(vec![(1, 500)])
        .build()
        .execute_with(|| {
            assert_ok!(ParachainStaking::delegate(
                RuntimeOrigin::signed(2),
                1,    // candidate
                100,  // amount
                1,    // candidate delegation count
                0,    // delegator delegation count
            ));

            assert_eq!(
                ParachainStaking::delegator_state(2).unwrap().total(),
                100
            );
        });
}
```

### Integration Tests

```typescript
// test/suites/dev/moonbase/test-staking/test-staking-delegate.ts
describeSuite({
  id: "D010501",
  title: "Staking - Delegation",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Should delegate to collator via precompile",
      test: async () => {
        const { result } = await context.createBlock(
          context.createTxn!({
            to: PRECOMPILE_STAKING_ADDRESS,
            data: encodeFunctionData({
              abi: stakingAbi,
              functionName: "delegate",
              args: [collator, amount, candidateCount, delegatorCount],
            }),
          })
        );

        expect(result?.successful).toBe(true);
      },
    });
  },
});
```

## Debugging Staking Issues

### Common Issues

| Issue                 | Cause                  | Solution                           |
|-----------------------|------------------------|------------------------------------|
| Delegation fails      | Below minimum          | Check MinDelegation                |
| Can't unstake         | Pending request        | Execute or cancel existing request |
| Missing rewards       | Not in top delegations | Increase delegation amount         |
| Collator not selected | Below threshold        | Increase self-bond                 |

### Inspect Staking State

```typescript
// Check if delegator is in top delegations
const topDelegations = await api.query.parachainStaking.topDelegations(collator);
const isInTop = topDelegations.unwrap().delegations.some(
  d => d.owner.toString() === delegator
);

// Check pending requests
const requests = await api.query.parachainStaking.delegationScheduledRequests(collator);
const pendingRequest = requests.find(
  r => r.delegator.toString() === delegator
);
```

### Debug Logging

```rust
// Add to pallet for debugging
log::debug!(
    target: "parachain-staking",
    "Delegation: delegator={:?}, candidate={:?}, amount={:?}",
    delegator, candidate, amount
);
```

## Key Files

- Pallet: `pallets/parachain-staking/src/lib.rs`
- Types: `pallets/parachain-staking/src/types.rs`
- Delegation: `pallets/parachain-staking/src/delegation_requests.rs`
- Rewards: `pallets/parachain-staking/src/inflation.rs`
- Precompile: `precompiles/parachain-staking/src/lib.rs`
- Tests: `test/suites/dev/moonbase/test-staking/`
