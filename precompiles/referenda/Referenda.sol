// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Referenda contract's address.
address constant REFERENDA_ADDRESS = 0x0000000000000000000000000000000000000811;

/// @dev The Referenda contract's instance.
Referenda constant REFERENDA_CONTRACT = Referenda(REFERENDA_ADDRESS);

/// @author The Moonbeam Team
/// @title Pallet Referenda Interface
/// @title The interface through which solidity contracts will interact with the Referenda pallet
/// @custom:address 0x0000000000000000000000000000000000000811
interface Referenda {
    /// @dev Defines the referenda origins that have tracks corresponding to uint8 representation
    /// The uint8 representation is defined in origins pallet Into<u16> for Origin
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

    /// Return the total referendum count
    /// @custom:selector 3a42ee31
    function referendumCount() external view returns (uint256);

    /// Return the submission deposit for all referenda
    /// @custom:selector aa14c39a
    function submissionDeposit() external view returns (uint256);

    /// Return the total count of deciding referenda per track
    /// @param trackId The track identifier
    /// @custom:selector 23cbda16
    function decidingCount(uint256 trackId) external view returns (uint256);

    /// Return the total count of deciding referenda per track
    /// @param trackId The track identifier
    /// @custom:selector 35028c55
    function trackInfo(uint256 trackId)
        external
        view
        returns (
            uint256,
            uint256,
            uint256,
            uint256,
            uint256,
            uint256
        );

    /// @dev Submit a referenda
    /// @custom:selector d865f8e8
    /// @param origin The origin from which the proposed referenda would be dispatched
    /// @param hash Hash of the proposal preimage
    /// @param at If true then AT block_number, else AFTER block_number
    /// @param block Inner block number for DispatchTime
    function submit(
        Origin origin,
        bytes memory hash,
        bool at,
        uint32 block
    ) external;

    /// @dev Post the Decision Deposit for a referendum
    /// @custom:selector 245ce18d
    /// @param index The index of the ongoing referendum that is not yet deciding
    function placeDecisionDeposit(uint32 index) external;

    /// @dev Refund the Decision Deposit for a closed referendum back to the depositor
    /// @custom:selector 1325d528
    /// @param  index The index of a closed referendum with decision deposit still locked
    function refundDecisionDeposit(uint32 index) external;
}
