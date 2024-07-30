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
    /// @dev Verifies a risc0 zk-proof receipt.
    ///
    /// @param receipt Risc0 zk-proof encoded receipt.
    /// @custom:selector 55c265fe
    function verifyProof(bytes memory receipt) external;
}
