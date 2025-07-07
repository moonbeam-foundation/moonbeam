// contracts/State.sol
// skip-compilation
// SPDX-License-Identifier: Apache 2

pragma solidity ^0.8.0;

/**
 * @title Counters
 * @author Matt Condon (@shrugs)
 * @dev Provides counters that can only be incremented, decremented or reset. This can be used e.g. to track the number
 * of elements in a mapping, issuing ERC721 ids, or counting request ids.
 *
 * Include with `using Counters for Counters.Counter;`
 */
library Counters {
    struct Counter {
        // This variable should never be directly accessed by users of the library: interactions must be restricted to
        // the library's function. As of Solidity v0.5.2, this cannot be enforced, though there is a proposal to add
        // this feature: see https://github.com/ethereum/solidity/issues/4637
        uint256 _value; // default: 0
    }

    function current(Counter storage counter) internal view returns (uint256) {
        return counter._value;
    }

    function increment(Counter storage counter) internal {
        unchecked {
            counter._value += 1;
        }
    }

    function decrement(Counter storage counter) internal {
        uint256 value = counter._value;
        require(value > 0, "Counter: decrement overflow");
        unchecked {
            counter._value = value - 1;
        }
    }

    function reset(Counter storage counter) internal {
        counter._value = 0;
    }
}

contract TokenStorage {
    struct State {
        string name;
        string symbol;
        uint64 metaLastUpdatedSequence;
        uint256 totalSupply;
        uint8 decimals;
        mapping(address => uint256) balances;
        mapping(address => mapping(address => uint256)) allowances;
        address owner;
        bool initialized;
        uint16 chainId;
        bytes32 nativeContract;
        // EIP712
        // Cache the domain separator and salt, but also store the chain id that
        // it corresponds to, in order to invalidate the cached domain separator
        // if the chain id changes.
        bytes32 cachedDomainSeparator;
        uint256 cachedChainId;
        address cachedThis;
        bytes32 cachedSalt;
        bytes32 cachedHashedName;
        // ERC20Permit draft
        mapping(address => Counters.Counter) nonces;
    }
}

contract TokenState {
    using Counters for Counters.Counter;

    TokenStorage.State _state;

    /**
     * @dev See {IERC20Permit-nonces}.
     */
    function nonces(address owner_) public view returns (uint256) {
        return _state.nonces[owner_].current();
    }

    /**
     * @dev "Consume a nonce": return the current value and increment.
     */
    function _useNonce(address owner_) internal returns (uint256 current) {
        Counters.Counter storage nonce = _state.nonces[owner_];
        current = nonce.current();
        nonce.increment();
    }
}
