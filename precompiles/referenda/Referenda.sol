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
    /// @dev Defines the referendum status.
    /// The values start at `0` (most permissive) and are represented as `uint8`
    enum ReferendumStatus {
        Ongoing,
        Approved,
        Rejected,
        Cancelled,
        TimedOut,
        Killed
    }

    /// Return the total referendum count
    /// @custom:selector 3a42ee31
    function referendumCount() external view returns (uint256);

    /// Return the submission deposit for all referenda
    /// @custom:selector aa14c39a
    function submissionDeposit() external view returns (uint256);

    /// Return the total count of deciding referenda per track
    /// @param trackId The track identifier
    /// @custom:selector 983d6425
    function decidingCount(uint16 trackId) external view returns (uint256);

    /// Return the trackIds
    /// @return trackIds Identifiers for all tracks (and origins)
    /// @custom:selector cc17da14
    function trackIds() external view returns (uint16[] memory trackIds);

    /// Return the governance parameters configured for the input TrackId
    /// @param trackId The track identifier
    /// @custom:selector 34038146
    function trackInfo(uint16 trackId)
        external
        view
        returns (
            string memory,
            uint256,
            uint256,
            uint256,
            uint256,
            uint256,
            uint256,
            bytes memory,
            bytes memory
        );

    /// Return the ReferendumStatus for the input referendumIndex
    /// @param referendumIndex The index of the referendum
    /// @custom:selector 8d407c0b
    function referendumStatus(uint32 referendumIndex)
        external
        view
        returns (ReferendumStatus);

    // /// Return the referendumInfo for an approved referendum
    // /// @param referendumIndex The index of the referendum
    // /// @custom:selector 078e5678
    // function approvedReferendumInfo(uint32 referendumIndex)
    //     external
    //     view
    //     returns (
    //         uint256,
    //         address,
    //         uint256,
    //         address,
    //         uint256
    //     );

    /// Return the block the referendum was killed
    /// @param referendumIndex The index of the referendum
    /// @custom:selector 6414ddc5
    function killedReferendumBlock(uint32 referendumIndex)
        external
        view
        returns (uint256);

    /// @dev Submit a referenda
    /// @custom:selector 95f9ed68
    /// @param trackId The trackId corresponding to the origin from which the proposal is to be
    /// dispatched. The trackId => origin mapping lives in `runtime/governance/tracks.rs`
    /// @param hash Hash of the proposal preimage
    /// @param block Block number at which this will be executed
    function submitAt(
        uint16 trackId,
        bytes memory hash,
        uint32 block
    ) external;

    /// @dev Submit a referenda
    /// @custom:selector 0a1ecbe9
    /// @param trackId The trackId corresponding to the origin from which the proposal is to be
    /// dispatched. The trackId => origin mapping lives in `runtime/governance/tracks.rs`
    /// @param hash Hash of the proposal preimage
    /// @param block Block number after which this will be executed
    function submitAfter(
        uint16 trackId,
        bytes memory hash,
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
