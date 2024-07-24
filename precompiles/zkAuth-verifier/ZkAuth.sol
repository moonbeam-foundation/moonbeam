// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The ZkAuth contract's address.
// address constant ZK_AUTH_ADDRESS = 0x000000000000000000000000000000000000081B;

/// @dev The ZkAuth contract's instance.
// ZkAuth constant ZK_AUTH_CONTRACT = ZkAuth(ZK_AUTH_ADDRESS);

/// @author The Moonbeam Team
/// @title ZkAuth verifier precompile
/// @dev Allows to execute an evm call after verifying the integrity and validity of
/// a risc0 zk-proof receipt.
/// @custom:address 0x000000000000000000000000000000000000081B
interface ZkAuth {
    /// @dev Verifies a risc0 zk-proof receipt and executes an evm call if valid.
    ///
    /// @param receipt Risc0 zk-proof encoded receipt.
    /// @param to Address to call.
    /// @param value Value to use inside the call.
    /// @param callData Call data for `to` address.
    /// @param gasLimit Gas limit for the execution.
    /// @custom:selector 242ee93b
    function verifyAndExecute(
        bytes memory receipt,
        address to,
        uint256 value,
        bytes memory callData,
        uint64 gasLimit
    ) external;

    // Needs to be compatible with account abstraction ERC-4337
}
