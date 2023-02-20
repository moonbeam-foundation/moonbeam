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
    struct TrackInfo {
        string name;
        uint256 maxDeciding;
        uint256 decisionDeposit;
        uint256 preparePeriod;
        uint256 decisionPeriod;
        uint256 confirmPeriod;
        uint256 minEnactmentPeriod;
        bytes minApproval;
        bytes minSupport;
    }
    struct OngoingReferendumInfo {
        /// The track of this referendum.
        uint16 trackId;
        /// The origin for this referendum.
        bytes origin;
        /// The hash of the proposal up for referendum.
        bytes proposal;
        /// Whether proposal is scheduled for enactment at or after `enactment_time`.
        bool enactmentType;
        /// The time the proposal should be scheduled for enactment.
        uint256 enactmentTime;
        /// The time of submission. Once `UndecidingTimeout` passes, it may be closed by anyone if
        /// `deciding` is `None`.
        uint256 submissionTime;
        address submissionDepositor;
        uint256 submissionDeposit;
        address decisionDepositor;
        uint256 decisionDeposit;
        /// When this referendum began being "decided". If confirming, then the
        /// end will actually be delayed until the end of the confirmation period.
        uint256 decidingSince;
        /// If nonzero, then the referendum has entered confirmation stage and will end at
        /// the block number as long as it doesn't lose its approval in the meantime.
        uint256 decidingConfirmingEnd;
        /// The number of aye votes, expressed in terms of post-conviction lock-vote.
        uint256 ayes;
        /// Percent of aye votes, expressed pre-conviction, over total votes in the class.
        uint32 support;
        /// Percent of aye votes over aye + nay votes.
        uint32 approval;
        /// Whether we have been placed in the queue for being decided or not.
        bool inQueue;
        /// The next scheduled wake-up
        uint256 alarmTime;
        /// Scheduler task address if scheduled
        bytes taskAddress;
    }
    struct ClosedReferendumInfo {
        ReferendumStatus status;
        uint256 end;
        address submissionDepositor;
        uint256 submissionDeposit;
        address decisionDepositor;
        uint256 decisionDeposit;
    }

    /// Return the total referendum count
    /// @custom:selector 3a42ee31
    function referendumCount() external view returns (uint32);

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
    function trackInfo(uint16 trackId) external view returns (TrackInfo memory);

    /// Return the ReferendumStatus for the input referendumIndex
    /// @param referendumIndex The index of the referendum
    /// @custom:selector 8d407c0b
    function referendumStatus(uint32 referendumIndex)
        external
        view
        returns (ReferendumStatus);

    /// Return the referendumInfo for an ongoing referendum
    /// @param referendumIndex The index of the referendum
    /// @custom:selector f033b7cd
    function ongoingReferendumInfo(uint32 referendumIndex)
        external
        view
        returns (OngoingReferendumInfo memory);

    /// Return the referendumInfo for a closed referendum
    /// @param referendumIndex The index of the referendum
    /// @custom:selector 14febfbf
    function closedReferendumInfo(uint32 referendumIndex)
        external
        view
        returns (ClosedReferendumInfo memory);

    /// Return the block the referendum was killed
    /// @param referendumIndex The index of the referendum
    /// @custom:selector 6414ddc5
    function killedReferendumBlock(uint32 referendumIndex)
        external
        view
        returns (uint256);

    /// @dev Submit a referenda
    /// @custom:selector 131f3468
    /// @param trackId The trackId corresponding to the origin from which the proposal is to be
    /// dispatched. The trackId => origin mapping lives in `runtime/governance/tracks.rs`
    /// @param proposalHash The proposed runtime call hash stored in the preimage pallet
    /// @param proposalLen The proposed runtime call length
    /// @param block Block number at which this will be executed
    /// @return referendumIndex Index of submitted referenda
    function submitAt(
        uint16 trackId,
        bytes32 proposalHash,
        uint32 proposalLen,
        uint32 block
    ) external returns (uint32 referendumIndex);

    /// @dev Submit a referenda
    /// @custom:selector 5b2479db
    /// @param trackId The trackId corresponding to the origin from which the proposal is to be
    /// dispatched. The trackId => origin mapping lives in `runtime/governance/tracks.rs`
    /// @param proposalHash The proposed runtime call hash stored in the preimage pallet
    /// @param proposalLen The proposed runtime call length
    /// @param block Block number after which this will be executed
    /// @return referendumIndex Index of submitted referenda
    function submitAfter(
        uint16 trackId,
        bytes32 proposalHash,
        uint32 proposalLen,
        uint32 block
    ) external returns (uint32 referendumIndex);

    /// @dev Post the Decision Deposit for a referendum
    /// @custom:selector 245ce18d
    /// @param index The index of the ongoing referendum that is not yet deciding
    function placeDecisionDeposit(uint32 index) external;

    /// @dev Refund the Decision Deposit for a closed referendum back to the depositor
    /// @custom:selector 1325d528
    /// @param  index The index of a closed referendum with decision deposit still locked
    function refundDecisionDeposit(uint32 index) external;

    /// @dev Refund the Submission Deposit for a closed referendum back to the depositor
    /// @custom:selector c28307ca
    /// @param  index The index of a closed referendum with submission deposit still locked
    function refundSubmissionDeposit(uint32 index) external;

    /// @dev A referenda has been submitted at a given block
    /// @custom:selector e02a819ecfc92874b5016c6a0e26f56a5cb08771f32ab818bf548d84ca3ae94d
    /// @param trackId uint16 The trackId
    /// @param blockNumber uint32 Block number at which it was set to be submitted
    /// @param hash bytes32 The hash of the proposal preimage
    event SubmittedAt(uint16 indexed trackId, uint32 blockNumber, bytes32 hash);

    /// @dev A referenda has been submitted after a given block
    /// @custom:selector a5117efbf0f4aa9e08dd135e69aa8ee4978f99fca86fc5154b5bd1b363eafdcf
    /// @param trackId uint16 The trackId
    /// @param blockNumber uint32 Block number after which it was set to be submitted
    /// @param hash bytes32 The hash of the proposal preimage
    event SubmittedAfter(
        uint16 indexed trackId,
        uint32 blockNumber,
        bytes32 hash
    );

    /// @dev Decision Deposit for a referendum has been placed
    /// @custom:selector 87e691fb2e6a679435f578d43cd67e1af825294e56064a9de0522b312b8e9a60
    /// @param index uint32 The index of the ongoing referendum that is not yet deciding
    event DecisionDepositPlaced(uint32 index);

    /// @dev Decision Deposit for a closed referendum has been refunded
    /// @custom:selector 61f241739b215680a33261f1dee7646d0e840d5e498c1142c1a534987d9b8ed8
    /// @param index uint32 The index of the closed referendum
    event DecisionDepositRefunded(uint32 index);
}
