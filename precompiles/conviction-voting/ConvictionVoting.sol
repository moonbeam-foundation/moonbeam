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

    /// @dev An account removed its vote from an ongoing poll.
    /// @custom:selector a820291b6a38b0fe8789b35869564f6337ab2efc897e234c5cc269042b728f8f
    /// @param pollIndex uint32 Index of the poll.
    /// @param voter address Address of the voter.
    event VoteRemove(
        uint32 indexed pollIndex,
        address voter
    );

    /// @dev An account removed a vote from a poll.
    /// @custom:selector 9f94fe7d53f8c8e609724084f81da9d41be0505a8d5363953727a488d4f1250f
    /// @param pollIndex uint32 Index of the poll.
    /// @param caller address Address of the origin caller.
    /// @param target address Address of the address which's vote is being removed.
    /// @param trackId uint16 The trackId.
    event VoteRemoveOther(
        uint32 indexed pollIndex,
        address caller,
        address target,
        uint16 trackId
    );

    /// @dev An account delegated for the given trackId.
    /// @custom:selector b1be766844177ed1f32c721f8d5c85a66fbb5ba6f18235aaeed18bf1856d529d
    /// @param trackId uint16 The trackId.
    /// @param from address Address of the caller.
    /// @param to address Address of the representative.
    /// @param delegatedAmount uint256 Amount being delegated.
    /// @param conviction uint8 Conviction being delegated.
    event Delegate(
        uint16 indexed trackId,
        address from,
        address to,
        uint256 delegatedAmount,
        uint8 conviction
    );

    /// @dev An account undelegated for the given trackId.
    /// @custom:selector 8884045fa295d3878585442c6a09c5567e5b30c2a19c8edc03c77c036baccb6e
    /// @param trackId uint16 The trackId.
    /// @param caller address Address of the caller.
    event Undelegate(
        uint16 indexed trackId,
        address caller
    );

    /// @dev An account unlocked freeable tokens for the given trackId.
    /// @custom:selector 5dcf630ebd6c48de9ece59c3378971de3f65f450c0c6e924d9607d80f58cfa79
    /// @param trackId uint16 The trackId.
    /// @param caller address Address of the caller.
    event Unlock(
        uint16 indexed trackId,
        address caller
    );
}
