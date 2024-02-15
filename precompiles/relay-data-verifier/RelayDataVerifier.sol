// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

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
///    is the relay block hash obtained in step 2 to get the storage proof for the keys.
/// 4. Moonbeam RPC Call: Submit an ethereum transaction (direclty or through a SC) to call the 
///    `verifyEntry` or `verifyEntries` function to verify the data against the latest relay block 
///    number. The call data contain the relay block number obtained in step 1, and the storage 
///    proof generated in step 3.
/// @custom:address 0x
interface RelayProofVerifier {
    /// @dev Verify Relay Chain Data
    /// @param relayBlockNumber: The relay block number against which the data is being verified.
    /// @param storageProof: The storage proof used to verify the data.
    /// @return value The value associated with the key.
    function verifyEntry(
        uint32 relayBlockNumber,
        bytes calldata storageProof,
        bytes calldata key
    ) external returns (bytes memory value);

    /// @dev Verify Relay Chain Proof for a batch of keys and return the corresponding values
    /// @param relayBlockNumber the relay block number for which the data is being verified
    /// @param storageProof the storage proof used to verify the data
    /// @param keys the keys to verify
    /// @return values the values associated with the keys
    function verifyEntries(
        uint32 relayBlockNumber,
        bytes calldata storageProof,
        bytes[] calldata keys
    ) external returns (bytes[] memory values);

    /// @dev Returns the latest relay block number that has a storage root stored on-chain.
    /// @return relayBlockNumber the lastest relay block number
    function latestRelayBlockNumber(
    ) external view returns (uint32 relayBlockNumber);
}
