// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The RelayDataVerifier contract's address.
address constant RELAY_DATA_VERIFIER_ADDRESS = 0x0000000000000000000000000000000000000819;

/// @dev The RelayDataVerifier contract's instance.
RelayDataVerifier constant RELAY_DATA_VERIFIER_CONTRACT = RelayDataVerifier(
    RELAY_DATA_VERIFIER_ADDRESS
);

/// @author The Moonbeam Team
/// @title Relay Proof Verifier Interface
/// @dev The interface that Solidity contracts use to interact with the Relay Proof Verifier
/// precompile.
/// A typical workflow to verify relay chain data is the following:
/// 1. Moonbeam RPC Call: Call `latestRelayBlockNumber` function to get the latest relay
///    block number tracked by the chain in `pallet-storage-root`.
/// 2. Relay RPC Call: Call `chain_getBlockHash(blockNumber)` RPC method to get the relay block hash
///    for the block number obtained in step 1.
/// 3. Relay RPC Call: Call `state_getReadProof(keys, at)` RPC method where `at`
///    is the relay block hash obtained in step 2 to get the 'ReadProof` of the entries.
/// 4. Moonbeam RPC Call: Submit an ethereum transaction (directly or through a SC) to call the
///    `verifyEntry` or `verifyEntries` function to verify the data against the relay block
///    number. The call data contain the relay block number obtained in step 1, and the read
///    proof generated in step 3, along with the key/s to verify.
/// @custom:address 0x0000000000000000000000000000000000000819
interface RelayDataVerifier {
    /// @dev ReadProof struct returned by the `state_getReadProof` RPC method.
    struct ReadProof {
        // The block hash against which the proof is generated
        bytes32 at;
        /// The storage proof
        bytes[] proof;
    }

    /// @dev Verifies a storage entry in the Relay Chain using a relay block number and a storage
    /// proof. This function takes a relay block number, a storage proof, and the key of the storage
    /// entry to verify. It returns the value associated with the key if the verification is
    /// successful.
    /// @custom:selector 27001faa
    /// @param relayBlockNumber The relay block number against which the entry is being verified.
    /// @param readProof The storage proof used to verify the entry.
    /// @param key The key of the storage entry to verify.
    /// @return value The value associated with the key, returned as a bytes array.
    function verifyEntry(
        uint32 relayBlockNumber,
        ReadProof calldata readProof,
        bytes calldata key
    ) external returns (bytes memory value);

    /// @dev Verifies a set of entries in the Relay Chain and returns the corresponding values.
    /// This function takes a relay block number, a storage proof, and an array of keys for the
    /// storage entries to verify. It returns an array of values associated with the keys, in the
    /// same order as the keys.
    /// @custom:selector 2da33a45
    /// @param relayBlockNumber The relay block number for which the data is being verified.
    /// @param readProof The storage proof used to verify the data.
    /// @param keys The keys of the storage entries to verify.
    /// @return values The values associated with the keys, returned in the same order as the keys.
    function verifyEntries(
        uint32 relayBlockNumber,
        ReadProof calldata readProof,
        bytes[] calldata keys
    ) external returns (bytes[] memory values);

    /// @dev Returns the latest relay block number that has a storage root stored on-chain.
    /// @custom:selector aed36869
    /// @return relayBlockNumber the lastest relay block number
    function latestRelayBlockNumber()
        external
        view
        returns (uint32 relayBlockNumber);
}
