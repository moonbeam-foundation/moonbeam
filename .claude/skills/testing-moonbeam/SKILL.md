---
name: testing-moonbeam
description: Runs and writes tests for the Moonbeam parachain including Rust unit tests, TypeScript integration tests, smoke tests, and multi-chain tests. Use when verifying changes, writing tests for new features, debugging test failures, running specific test suites, or creating test fixtures.
---

# Testing Moonbeam

## Contents
- [Test Types Overview](#test-types-overview)
- [Rust Unit Tests](#rust-unit-tests)
- [TypeScript Integration Tests](#typescript-integration-tests)
- [Smoke Tests](#smoke-tests)
- [Multi-Chain Tests](#multi-chain-tests-zombienet)
- [Test Helpers](#test-helpers)
- [Best Practices](#best-practices)

## Test Types Overview

| Type          | Location                     | Purpose                   | Command                               |
|---------------|------------------------------|---------------------------|---------------------------------------|
| Rust Unit     | `*/src/tests.rs`             | Pallet/module logic       | `cargo test`                          |
| Dev Tests     | `test/suites/dev/`           | Development mode features | `pnpm moonwall test dev_moonbase`     |
| Smoke Tests   | `test/suites/smoke/`         | Quick sanity checks       | `pnpm moonwall test smoke_moonbase`   |
| Tracing Tests | `test/suites/tracing-tests/` | EVM tracing               | `pnpm moonwall test tracing_moonbase` |
| Zombienet     | `test/suites/zombie/`        | Multi-chain scenarios     | `pnpm moonwall test zombie_*`         |
| Chopsticks    | `test/suites/chopsticks/`    | Forked state testing      | `pnpm moonwall test chopsticks_*`     |

## Rust Unit Tests

### Running Tests

```bash
# All tests
cargo test

# Specific pallet
cargo test -p pallet-parachain-staking

# Specific test
cargo test -p pallet-parachain-staking test_delegate

# With output
cargo test -p pallet-parachain-staking -- --nocapture

# Release mode (faster)
cargo test --release -p pallet-parachain-staking
```

### Writing Pallet Tests

```rust
// pallets/my-pallet/src/tests.rs
use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn basic_functionality_works() {
    new_test_ext().execute_with(|| {
        // Set block number (required for events)
        System::set_block_number(1);

        // Test the functionality
        assert_ok!(MyPallet::do_something(RuntimeOrigin::signed(1), 42));

        // Verify storage changed
        assert_eq!(MyPallet::something(), Some(42));

        // Verify event was emitted
        System::assert_last_event(Event::SomethingDone { value: 42 }.into());
    });
}

#[test]
fn fails_with_invalid_input() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            MyPallet::do_something(RuntimeOrigin::signed(1), 0),
            Error::<Test>::InvalidValue
        );
    });
}

#[test]
fn only_root_can_call() {
    new_test_ext().execute_with(|| {
        // Signed origin fails
        assert_noop!(
            MyPallet::root_only(RuntimeOrigin::signed(1)),
            DispatchError::BadOrigin
        );

        // Root succeeds
        assert_ok!(MyPallet::root_only(RuntimeOrigin::root()));
    });
}
```

### ExtBuilder Pattern

```rust
// pallets/my-pallet/src/mock.rs
pub struct ExtBuilder {
    initial_balance: Balance,
    some_config: bool,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            initial_balance: 1000,
            some_config: false,
        }
    }
}

impl ExtBuilder {
    pub fn with_balance(mut self, balance: Balance) -> Self {
        self.initial_balance = balance;
        self
    }

    pub fn with_config(mut self) -> Self {
        self.some_config = true;
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap();

        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(1, self.initial_balance)],
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

// Usage in tests
#[test]
fn test_with_custom_setup() {
    ExtBuilder::default()
        .with_balance(5000)
        .with_config()
        .build()
        .execute_with(|| {
            // Test code
        });
}
```

## TypeScript Integration Tests

### Setup

```bash
cd test
pnpm install
pnpm build
```

### Running Tests

```bash
# Run all dev tests
pnpm moonwall test dev_moonbase

# Run specific test file
pnpm moonwall test -d "test-staking" dev_moonbase D010101

# Run single test
pnpm moonwall test -d "test-staking" dev_moonbase D010101T01

# Run with filter
pnpm moonwall test dev_moonbase --grep "delegate"
```

### Writing Dev Tests

```typescript
// test/suites/dev/moonbase/test-feature/test-feature-basic.ts
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { ALITH_ADDRESS, alith, baltathar } from "@moonwall/util";

describeSuite({
  id: "D010101",
  title: "Feature - Basic Tests",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let contractAddress: `0x${string}`;

    beforeAll(async () => {
      // Setup code runs before all tests
      const { contractAddress: addr } = await context.deployContract!("MyContract");
      contractAddress = addr;
    });

    it({
      id: "T01",
      title: "Should do something correctly",
      test: async () => {
        // Create a block with a transaction
        const rawTx = await context.createTxn!({
          to: contractAddress,
          data: "0x...", // encoded call
          gas: 100000n,
        });

        const { result } = await context.createBlock(rawTx);

        expect(result?.successful).toBe(true);

        // Query state
        const value = await context.viem().readContract({
          address: contractAddress,
          abi: MyContractAbi,
          functionName: "getValue",
        });

        expect(value).toBe(42n);
      },
    });

    it({
      id: "T02",
      title: "Should revert on invalid input",
      test: async () => {
        const rawTx = await context.createTxn!({
          to: contractAddress,
          data: "0x...",
        });

        const { result } = await context.createBlock(rawTx);

        expect(result?.successful).toBe(false);
        expect(result?.error?.name).toContain("revert");
      },
    });
  },
});
```

### Testing Precompiles

```typescript
// test/suites/dev/moonbase/test-precompile/test-my-precompile.ts
import { describeSuite, expect } from "@moonwall/cli";
import { PRECOMPILE_MY_ADDRESS } from "../../helpers/constants";

describeSuite({
  id: "D020101",
  title: "MyPrecompile Tests",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Should call precompile successfully",
      test: async () => {
        const rawTx = await context.createTxn!({
          to: PRECOMPILE_MY_ADDRESS,
          data: encodeFunctionData({
            abi: MyPrecompileAbi,
            functionName: "doSomething",
            args: [42n],
          }),
        });

        const { result } = await context.createBlock(rawTx);

        expect(result?.successful).toBe(true);

        // Verify via view function
        const value = await context.viem().readContract({
          address: PRECOMPILE_MY_ADDRESS,
          abi: MyPrecompileAbi,
          functionName: "getSomething",
        });

        expect(value).toBe(42n);
      },
    });
  },
});
```

### Testing Substrate Extrinsics

```typescript
it({
  id: "T03",
  title: "Should execute substrate extrinsic",
  test: async () => {
    const api = context.polkadotJs();

    // Create and sign extrinsic
    const tx = api.tx.myPallet.doSomething(42);

    const { result } = await context.createBlock(
      tx.signAsync(alith)
    );

    expect(result?.successful).toBe(true);

    // Query storage
    const value = await api.query.myPallet.something();
    expect(value.unwrap().toNumber()).toBe(42);
  },
});
```

## Smoke Tests

Smoke tests run against live or dev networks for quick validation.

```typescript
// test/suites/smoke/test-basic/test-basic-health.ts
import { describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "S010101",
  title: "Basic Health Checks",
  foundationMethods: "smoke",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Chain should be producing blocks",
      test: async () => {
        const api = context.polkadotJs();

        const blockNumber = (await api.rpc.chain.getHeader()).number.toNumber();
        expect(blockNumber).toBeGreaterThan(0);
      },
    });

    it({
      id: "T02",
      title: "RPC should be responsive",
      test: async () => {
        const health = await context.viem().request({
          method: "net_version",
        });
        expect(health).toBeDefined();
      },
    });
  },
});
```

## Multi-Chain Tests (Zombienet)

```typescript
// test/suites/zombie/test-xcm/test-xcm-transfer.ts
import { describeSuite, expect, beforeAll } from "@moonwall/cli";

describeSuite({
  id: "Z010101",
  title: "XCM Transfer Tests",
  foundationMethods: "zombie",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Should transfer assets via XCM",
      test: async () => {
        const moonbeamApi = context.moonbeamApi();
        const relayApi = context.relayApi();

        // Execute XCM from relay
        // ...

        // Wait for XCM to be processed
        await context.waitForXcm();

        // Verify on Moonbeam
        // ...
      },
    });
  },
});
```

## Test Helpers

### Common Helpers Location

```
test/helpers/
├── accounts.ts      # Account management
├── assets.ts        # Asset utilities
├── balances.ts      # Balance queries
├── block.ts         # Block utilities
├── constants.ts     # Network constants
├── eth-transactions.ts  # ETH tx helpers
├── precompiles.ts   # Precompile helpers
├── staking.ts       # Staking helpers
└── xcm.ts          # XCM utilities
```

### Using Helpers

```typescript
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  alith,
  createViemTransaction
} from "@moonwall/util";
import { getBalance, expectBalanceChange } from "../../helpers/balances";

it({
  id: "T01",
  title: "Should transfer balance",
  test: async () => {
    const initialBalance = await getBalance(context, BALTATHAR_ADDRESS);

    const tx = await createViemTransaction(context, {
      to: BALTATHAR_ADDRESS,
      value: 1000000000000000000n,
    });

    await context.createBlock(tx);

    await expectBalanceChange(
      context,
      BALTATHAR_ADDRESS,
      initialBalance,
      1000000000000000000n
    );
  },
});
```

## Best Practices

1. **Test IDs**: Use consistent ID format (`D010101T01`)
2. **Isolation**: Each test should be independent
3. **Cleanup**: Reset state if needed between tests
4. **Assertions**: Test both success and failure cases
5. **Gas**: Always specify reasonable gas limits
6. **Events**: Verify events are emitted correctly
7. **Edge Cases**: Test boundary conditions
8. **Documentation**: Clear test titles describe behavior

## Running CI Tests Locally

```bash
# Full test suite (as CI runs)
cargo test --release
cd test && pnpm moonwall test dev_moonbase

# Quick validation
cargo test -p pallet-parachain-staking
cd test && pnpm moonwall test dev_moonbase D010101
```
