// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Conviction Voting contract's address.
address constant CONVICTION_VOTING_ADDRESS = 0x0000000000000000000000000000000000000812;

/// @dev The Conviction Voting contract's instance.
ConvictionVoting constant CONVICTION_VOTING_CONTRACT = ConvictionVoting(
    CONVICTION_VOTING_ADDRESS
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

    struct ClassLock {
        uint16 trackId;
        uint256 amount;
    }

    struct VotingFor {
        bool isCasting;
        bool isDelegating;
        Casting casting;
        Delegating delegating;
    }

    struct PollAccountVote {
        uint32 pollIndex;
        AccountVote accountVote;
    }

    struct Casting {
        PollAccountVote[] votes;
        Delegations delegations;
        PriorLock prior;
    }

    struct Delegating {
        uint256 balance;
        address target;
        Conviction conviction;
        Delegations delegations;
        PriorLock prior;
    }

    struct AccountVote {
        bool is_standard;
        bool is_split;
        bool is_split_abstain;
        StandardVote standard;
        SplitVote split;
        SplitAbstainVote split_abstain;
    }

    struct StandardVote {
        Vote vote;
        uint256 balance;
    }

    struct Vote {
        bool aye;
        Conviction conviction;
    }

    struct SplitVote {
        uint256 aye;
        uint256 nay;
    }

    struct SplitAbstainVote {
        uint256 aye;
        uint256 nay;
        uint256 abstain;
    }

    struct Delegations {
        uint256 votes;
        uint256 capital;
    }

    struct PriorLock {
        uint256 balance;
    }

    /// @dev Retrieve votings for a given account and track.
    /// @custom:selector 501447ee
    /// @param who The requested account
    /// @param trackId The requested track
    function votingFor(
        address who,
        uint16 trackId
    ) external view returns (ClassLock[] memory);

    /// @dev Retrieve class locks for a given account.
    /// @custom:selector 7ae8ac92
    /// @param who The requested account
    function classLocksFor(address who) external view returns (uint256);

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

    /// @dev Vote split in a poll.
    /// @custom:selector dd6c52a4
    /// @param pollIndex Index of poll
    /// @param aye Balance locked for aye vote
    /// @param nay Balance locked for nay vote
    function voteSplit(uint32 pollIndex, uint256 aye, uint256 nay) external;

    /// @dev Vote split abstain in a poll.
    /// @custom:selector 52004540
    /// @param pollIndex Index of poll
    /// @param aye Balance locked for aye vote
    /// @param nay Balance locked for nay vote
    /// @param abstain Balance locked for abstain vote (support)
    function voteSplitAbstain(
        uint32 pollIndex,
        uint256 aye,
        uint256 nay,
        uint256 abstain
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
    /// @custom:selector 3839f7832b2a6263aa1fd5040f37d10fd4f9e9c4a9ef07ec384cb1cef9fb4c0e
    /// @param pollIndex uint32 Index of the poll.
    /// @param voter address Address of the voter.
    /// @param aye bool Is it a vote for or against the poll.
    /// @param voteAmount uint256 Amount used to vote.
    /// @param conviction uint8 Conviction of the vote.
    event Voted(
        uint32 indexed pollIndex,
        address voter,
        bool aye,
        uint256 voteAmount,
        uint8 conviction
    );

    /// @dev An account made a split vote in a poll.
    /// @custom:selector 022787093a8aa26fe59d28969068711f73e0e78ae67d9359c71058b6a21f7ef0
    /// @param pollIndex uint32 Index of the poll.
    /// @param voter address Address of the voter.
    /// @param aye uint256 Amount for aye vote.
    /// @param nay uint256 Amount for nay vote.
    event VoteSplit(
        uint32 indexed pollIndex,
        address voter,
        uint256 aye,
        uint256 nay
    );

    /// @dev An account made a split abstain vote in a poll.
    /// @custom:selector 476e687ab5e38fc714552f3acc083d7d83ccaa12ea11dd5f3393478d158c6fd4
    /// @param pollIndex uint32 Index of the poll.
    /// @param voter address Address of the voter.
    /// @param aye uint256 Amount for aye vote.
    /// @param nay uint256 Amount for nay vote.
    /// @param abstain uint256 Amount for abstained.
    event VoteSplitAbstained(
        uint32 indexed pollIndex,
        address voter,
        uint256 aye,
        uint256 nay,
        uint256 abstain
    );

    /// @dev An account removed its vote from an ongoing poll.
    /// @custom:selector 49fc1dd929f126e1d88cbb9c135625e30c2deba291adeea4740e446098b9957b
    /// @param pollIndex uint32 Index of the poll.
    /// @param voter address Address of the voter.
    event VoteRemoved(uint32 indexed pollIndex, address voter);

    /// @dev An account removed a vote from a poll.
    /// @custom:selector c1d068675720ab00d0c8792a0cbc7e198c0d2202111f0280f039f2c09c50491b
    /// @param pollIndex uint32 Index of the poll.
    /// @param caller address Address of the origin caller.
    /// @param target address Address of the address which's vote is being removed.
    /// @param trackId uint16 The trackId.
    event VoteRemovedOther(
        uint32 indexed pollIndex,
        address caller,
        address target,
        uint16 trackId
    );

    /// @dev An account delegated for the given trackId.
    /// @custom:selector 6cc151d547592e227b1e85a264ac3699c6f1014112b08bb3832de1f23b9c66db
    /// @param trackId uint16 The trackId.
    /// @param from address Address of the caller.
    /// @param to address Address of the representative.
    /// @param delegatedAmount uint256 Amount being delegated.
    /// @param conviction uint8 Conviction being delegated.
    event Delegated(
        uint16 indexed trackId,
        address from,
        address to,
        uint256 delegatedAmount,
        uint8 conviction
    );

    /// @dev An account undelegated for the given trackId.
    /// @custom:selector 1053303328f6db14014ccced6297bcad2b3897157ce46070711ab995a05dfa14
    /// @param trackId uint16 The trackId.
    /// @param caller address Address of the caller.
    event Undelegated(uint16 indexed trackId, address caller);

    /// @dev An account called to unlock tokens for the given trackId.
    /// @custom:selector dcf72fa65ca7fb720b9ccc8ee28e0188edc3d943115124cdd4086c49f836a128
    /// @param trackId uint16 The trackId.
    /// @param caller address Address of the caller.
    event Unlocked(uint16 indexed trackId, address caller);
}
