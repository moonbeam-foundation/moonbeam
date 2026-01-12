---
name: implementing-eips
description: Implements Ethereum Improvement Proposals (EIPs) in the Moonbeam runtime to maintain Ethereum compatibility. Use when adding support for new opcodes, implementing precompile standards, supporting new transaction types, updating gas costs per EIP specs, or implementing account abstraction features.
---

# Implementing Ethereum EIPs

## Contents
- [EIP Implementation Workflow](#eip-implementation-workflow)
- [Common EIP Categories](#3-common-eip-categories)
- [Testing EIP Implementation](#4-testing-eip-implementation)
- [Common EIPs and Implementation Status](#common-eips-and-implementation-status)

## EIP Implementation Workflow

### 1. Analyze the EIP

Before implementing, thoroughly understand:
- What the EIP changes (opcodes, precompiles, behavior)
- Dependencies on other EIPs
- Gas cost implications
- Security considerations
- Backward compatibility requirements

### 2. Identify Implementation Location

| EIP Type            | Implementation Location     |
|---------------------|-----------------------------|
| New opcode          | Frontier (SputnikVM fork)   |
| New precompile      | `/precompiles/`             |
| Transaction type    | `pallet-ethereum`, Frontier |
| Gas schedule change | Runtime configuration       |
| RPC method          | `/client/rpc/`              |
| State change        | Runtime migration           |

### 3. Common EIP Categories

#### A. New Precompiles (e.g., EIP-4844 point evaluation)

```rust
// precompiles/new-precompile/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

use fp_evm::PrecompileHandle;
use precompile_utils::prelude::*;

/// Standard precompile address per EIP
pub const PRECOMPILE_ADDRESS: u64 = 0x0a; // Example: Point evaluation

pub struct NewPrecompile;

impl NewPrecompile {
    pub fn execute(handle: &mut impl PrecompileHandle) -> EvmResult<Vec<u8>> {
        // Record gas cost per EIP specification
        handle.record_cost(GAS_COST)?;

        let input = handle.input();

        // Validate input format per EIP
        if input.len() != EXPECTED_INPUT_SIZE {
            return Err(revert("Invalid input length"));
        }

        // Implement the precompile logic
        let result = do_computation(input)?;

        Ok(result)
    }
}
```

#### B. New Transaction Types (e.g., EIP-1559, EIP-4844)

Transaction types are primarily handled in Frontier. For Moonbeam-specific adjustments:

```rust
// In runtime/common/src/impl_on_charge_evm_transaction.rs
impl<T> OnChargeEVMTransaction<T> for OnChargeEVMTransactionImpl<T>
where
    T: pallet_evm::Config + pallet_balances::Config,
{
    fn withdraw_fee(
        who: &H160,
        fee: U256,
    ) -> Result<Option<Self::LiquidityInfo>, pallet_evm::Error<T>> {
        // Handle fee logic per EIP-1559
        // max_fee_per_gas, max_priority_fee_per_gas
    }
}
```

#### C. New Opcodes

Opcodes require changes to the EVM implementation (Frontier/SputnikVM). Coordinate with upstream.

```rust
// Configuration in runtime
parameter_types! {
    pub const ChainId: u64 = 1284; // Moonbeam
    pub BlockGasLimit: U256 = U256::from(BLOCK_GAS_LIMIT);
    pub PrecompilesValue: MoonbeamPrecompiles<Runtime> = MoonbeamPrecompiles::<_>::new();
    pub WeightPerGas: Weight = WEIGHT_PER_GAS;
}

impl pallet_evm::Config for Runtime {
    // EVM configuration including hardfork settings
    type ChainId = ChainId;
    type BlockGasLimit = BlockGasLimit;
    // ...
}
```

#### D. Gas Schedule Changes

```rust
// Update gas costs in runtime configuration
parameter_types! {
    pub const GasLimitPovSizeRatio: u64 = 16; // Per EIP specifications
}
```

### 4. Testing EIP Implementation

#### Unit Tests

```rust
#[test]
fn test_eip_xxxx_basic_functionality() {
    new_test_ext().execute_with(|| {
        // Test the basic EIP behavior
    });
}

#[test]
fn test_eip_xxxx_edge_cases() {
    new_test_ext().execute_with(|| {
        // Test edge cases mentioned in EIP
    });
}

#[test]
fn test_eip_xxxx_gas_costs() {
    new_test_ext().execute_with(|| {
        // Verify gas costs match EIP specification
    });
}
```

#### Integration Tests

```typescript
// test/suites/dev/moonbase/test-eip-xxxx/test-eip-xxxx-basic.ts
import { describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "Dxxxxxx",
  title: "EIP-XXXX Implementation",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Should support EIP-XXXX feature",
      test: async () => {
        // Test EIP functionality via RPC
        const result = await context.viem().call({
          to: "0x...", // precompile address
          data: "0x...", // encoded call
        });
        expect(result).toBe("0x...");
      },
    });
  },
});
```

#### Ethereum Compatibility Tests

```typescript
// Compare behavior with Ethereum reference
it({
  id: "T02",
  title: "Should match Ethereum reference behavior",
  test: async () => {
    // Use test vectors from EIP
    const testVectors = [
      { input: "0x...", expected: "0x..." },
      // ...
    ];

    for (const vector of testVectors) {
      const result = await context.viem().call({
        to: PRECOMPILE_ADDRESS,
        data: vector.input,
      });
      expect(result).toBe(vector.expected);
    }
  },
});
```

### 5. Checklist for EIP Implementation

- [ ] Read and understand the EIP specification
- [ ] Identify affected components (precompiles, opcodes, transaction handling)
- [ ] Check if Frontier already supports it
- [ ] Implement the feature in appropriate location
- [ ] Match gas costs exactly per EIP specification
- [ ] Add comprehensive unit tests
- [ ] Add integration tests with EIP test vectors
- [ ] Test backward compatibility
- [ ] Document any Moonbeam-specific deviations
- [ ] Update runtime version if needed
- [ ] Create migration if state changes required

## Common EIPs and Implementation Status

### Implemented
- EIP-1559: Fee market (base fee burning)
- EIP-2930: Access lists
- EIP-2718: Typed transactions
- EIP-2612: Permit (in call-permit precompile)
- EIP-165: Interface detection (in precompiles)
- EIP-712: Typed structured data hashing

### Implementation Patterns

#### EIP-2612 Permit Pattern
```rust
// precompiles/call-permit/src/lib.rs
#[precompile::public("permit(address,address,uint256,uint256,uint8,bytes32,bytes32)")]
fn permit(
    handle: &mut impl PrecompileHandle,
    owner: Address,
    spender: Address,
    value: U256,
    deadline: U256,
    v: u8,
    r: H256,
    s: H256,
) -> EvmResult {
    // Verify signature per EIP-712
    // Set allowance
}
```

#### EIP-165 Interface Detection
```rust
#[precompile::public("supportsInterface(bytes4)")]
#[precompile::view]
fn supports_interface(
    handle: &mut impl PrecompileHandle,
    interface_id: [u8; 4],
) -> EvmResult<bool> {
    Ok(matches!(
        interface_id,
        [0x01, 0xff, 0xc9, 0xa7] | // ERC165
        [0x..., 0x..., 0x..., 0x...] // Our interface
    ))
}
```

## Resources

- EIP Repository: https://eips.ethereum.org/
- Frontier: https://github.com/polkadot-evm/frontier
- Ethereum Yellow Paper: https://ethereum.github.io/yellowpaper/paper.pdf
- EVM Opcodes: https://www.evm.codes/
- Moonbeam MBIPs: `/MBIPS/` directory for Moonbeam-specific proposals
