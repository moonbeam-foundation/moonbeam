// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Preimage contract's address.
address constant Preimage_ADDRESS = 0x0000000000000000000000000000000000000813;

/// @dev The Preimage contract's instance.
Preimage constant Preimage_CONTRACT = Preimage(Preimage_ADDRESS);

/// @author The Moonbeam Team
/// @title Pallet Preimage Interface
/// @title The interface through which solidity contracts will interact with the Preimage pallet
/// @custom:address 0x0000000000000000000000000000000000000813
interface Preimage {
    /// @dev Register a Preimage on-chain.
    /// @custom:selector cb00f603
    /// @param encodedProposal The preimage to be registered on-chain
    function notePreimage(bytes memory encodedProposal) external;

    /// @dev Clear an unrequested preimage from storage.
    /// @custom:selector 02e71b45
    /// @param hash The preimage to be cleared from storage
    function unnotePreimage(bytes32 hash) external;
}
