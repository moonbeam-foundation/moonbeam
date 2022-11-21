// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Referenda contract's address. TODO: UPDATE ADDRESS
address constant REFERENDA_ADDRESS = 0x000000000000000000000000000000000000080b;

/// @dev The Referenda contract's instance.
Referenda constant REFERENDA_CONTRACT = Referenda(REFERENDA_ADDRESS);

/// @author The Moonbeam Team
/// @title Pallet Referenda Interface
/// @title The interface through which solidity contracts will interact with the Referenda pallet
/// @custom:address 0x000000000000000000000000000000000000080b TODO: UPDATE ADDRESS
interface Referenda {
    /// @dev Defines the referenda origins that have tracks corresponding to uint8 representation
    /// The uint8 representation is defined in pallet-governance-origins Into<u16> for Origin
    /// From top to bottom: 1, 10, 11, 12, 13, 14, 15
    enum Origin {
        WhitelistedCaller,
        Treasurer,
        ReferendumCanceller,
        ReferendumKiller,
        SmallSpender,
        MediumSpender,
        BigSpender
    }

    /// @dev Submit a referenda
    /// @custom:selector 74a34dd3
    /// @param origin The origin from which the proposed referenda would be dispatched
    function submit(Origin origin) external;
}
