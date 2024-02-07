// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @author The Moonbeam Team
/// @title Relay Chain Data Verifier Interface
/// @dev The interface that Solidity contracts use to interact with the Relay Data Verifier 
/// precompile
/// @custom:address 0x
interface RelayDataVerifier {
    /// @dev Verify Relay Chain Data
    /// @param relayBlockNumber the relay block number for which the data is being verified
    /// @param storageProof the storage proof used to verify the data
    /// @return value the value associated with the key
    function verify(
        uint32 relayBlockNumber,
        bytes[] calldata storageProof,
        bytes calldata key
    ) external view returns (bytes memory value);

    /// @dev Verify Relay Chain Data
    /// @param relayBlockNumber the relay block number for which the data is being verified
    /// @param storageProof the storage proof used to verify the data
    /// @param keys the keys to verify
    /// @return values the values associated with the keys
    function verifyBatch(
        uint32 relayBlockNumber,
        bytes[] calldata storageProof,
        bytes[] calldata keys
    ) external view returns (bytes[] memory values);

    /// @dev Returns the last block relay number and hash stored on chain
    /// @return relayBlockNumber the last relay block number
    /// @return relayBlockHash the hash of the last relay block
    function getLastRelayBlock(
    ) external view returns (uint32 relayBlockNumber, bytes32 relayBlockHash);
}
