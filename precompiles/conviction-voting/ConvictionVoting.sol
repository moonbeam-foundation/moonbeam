// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Conviction Voting contract's address.
address constant Conviction_Voting_ADDRESS = 0x0000000000000000000000000000000000000812;

/// @dev The Conviction Voting contract's instance.
ConvictionVoting constant Conviction_Voting_CONTRACT = ConvictionVoting(
    Conviction_Voting_ADDRESS
);

/// @author The Moonbeam Team
/// @title Pallet Conviction Voting Interface
/// @title The interface through which solidity contracts will interact with the Conviction Voting pallet
/// @custom:address 0x0000000000000000000000000000000000000812
interface ConvictionVoting {
    /// @dev Defines the conviction multiplier type.
    /// The values start at `0` and are represented as `uint8`.
    /// None => 0.1x votes, unlocked.
    /// Locked1x => 1x votes, locked for an enactment period following a successful vote.
    /// Locked2x => 2x votes, locked for 2x enactment periods following a successful vote
    /// Locked3x => 3x votes, locked for 4x...
    /// Locked4x => 4x votes, locked for 8x...,
    /// Locked5x => 5x votes, locked for 16x...
    /// Locked6x => 6x votes, locked for 32x...
    enum Conviction {
        None,
        Locked1x,
        Locked2x,
        Locked3x,
        Locked4x,
        Locked5x,
        Locked6x
    }

    /// @dev Vote yes in a poll.
    /// @custom:selector da9df518
    /// @param pollIndex Index of poll
    /// @param voteAmount Balance locked for vote
    /// @param conviction Conviction multiplier for length of vote lock
    function voteYes(
        uint32 pollIndex,
        uint256 voteAmount,
        Conviction conviction
    ) external;

    /// @dev Vote no in a poll.
    /// @custom:selector cc600eba
    /// @param pollIndex Index of poll
    /// @param voteAmount Balance locked for vote
    /// @param conviction Conviction multiplier for length of vote lock
    function voteNo(
        uint32 pollIndex,
        uint256 voteAmount,
        Conviction conviction
    ) external;

    /// @dev Remove vote in poll
    /// @custom:selector 79cae220
    /// @param pollIndex Index of the poll
    function removeVote(uint32 pollIndex) external;

    /// @dev Remove vote in poll for other voter
    /// @custom:selector cbcb9276
    //// @param target The voter to have vote removed. The removed vote must already be expired.
    /// @param trackId The trackId
    /// @param pollIndex the poll index
    function removeOtherVote(
        address target,
        uint16 trackId,
        uint32 pollIndex
    ) external;

    /// @dev Delegate to a representative for the vote trackId
    /// @custom:selector 681750e8
    /// @param trackId The trackId
    /// @param representative The representative for the trackId
    /// @param conviction The conviction multiplier
    /// @param amount delegated to representative for this vote trackId
    function delegate(
        uint16 trackId,
        address representative,
        Conviction conviction,
        uint256 amount
    ) external;

    /// @dev Undelegate for the trackId
    /// @custom:selector 98be4094
    /// @param trackId The trackId
    function undelegate(uint16 trackId) external;

    /// @dev Unlock tokens locked for trackId
    /// @custom:selector 4259d98c
    /// @param trackId The trackId
    /// @param target The target address
    function unlock(uint16 trackId, address target) external;

    /// @dev An account made a vote in a poll.
    /// @custom:selector 24052a032f75e3589c8c109de4f7b31ed98076e0ff7a9ce8ae548cf6ef45d67b
    /// @param pollIndex uint32 Index of the poll.
    /// @param voter address Address of the voter.
    /// @param aye bool Is it a vote for or against the poll.
    /// @param voteAmount uint256 Amount used to vote.
    /// @param conviction uint8 Conviction of the vote.
    event Vote(
        uint32 indexed pollIndex,
        address voter,
        bool aye,
        uint256 voteAmount,
        uint8 conviction
    );
}
