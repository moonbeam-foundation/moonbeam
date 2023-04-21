// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Gmp contract's address.
address constant GMP_ADDRESS = 0x0000000000000000000000000000000000000816;

/// @dev The Gmp contract's instance.
Gmp constant GMP_CONTRACT = Gmp(GMP_ADDRESS);

/// @author The Moonbeam Team
/// @title Gmp precompile
/// @dev Provides an endpoint to Gmp protocols which can automatically forward to XCM
/// @custom:address 0x0000000000000000000000000000000000000815
interface Gmp {
    // TODO: Here we would specify the endpoints for each GMP protocol on a case by case basis.
    //       These endpoints are basically the hand offs for each protocol -- where they delegate to
    //       the target contract.
    //
    //       This design should allow users to interact with this precompile with no changes to the
    //       underlying GMP protocols by simply specifying the correct precompile as the target.

    /// Receive a wormhole VAA and process it
    ///
    /// @custom:selector f53774ab
    function wormholeTransferERC20(bytes memory vaa) external;
}
