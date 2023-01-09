// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Preimage contract's address. TODO: UPDATE ADDRESS
address constant Preimage_ADDRESS = 0x000000000000000000000000000000000000080b;

/// @dev The Preimage contract's instance.
Preimage constant Preimage_CONTRACT = Preimage(Preimage_ADDRESS);

/// @author The Moonbeam Team
/// @title Pallet Preimage Interface
/// @title The interface through which solidity contracts will interact with the Preimage pallet
/// @custom:address 0x000000000000000000000000000000000000080b TODO: UPDATE ADDRESS
interface Preimage {
    /// @dev Register a Preimage on-chain.
    /// @custom:selector 74a34dd3
    /// @param encodedProposal
    function note_preimage(
        bytes memory encodedProposal,
    ) external;

    /// @dev Clear an unrequested preimage from storage.
    /// @custom:selector 74a34dd3
    /// @param hash The preimage to be cleared from storage
    function unnote_preimage(bytes32 hash) external;
}
