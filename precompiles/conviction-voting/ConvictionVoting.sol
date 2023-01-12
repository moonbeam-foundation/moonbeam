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
    /// @dev Defines the conviction multiplier type represented as `uint8`.
    /// The values start at `0` with 0.1x multiplier and votes unlocked.
    enum Conviction {
        None,
        Locked1x,
        Locked2x,
        Locked3x,
        Locked4x,
        Locked5x,
        Locked6x
    }

    /// @dev Vote in a poll.
    /// @custom:selector 35ee6e0e
    /// @param pollIndex Index of poll
    /// @param aye Yes or no vote
    /// @param voteAmount Balance locked for vote
    /// @param conviction Conviction multiplier for length of vote lock
    function vote(
        uint32 pollIndex,
        bool aye,
        uint256 voteAmount,
        Conviction conviction
    ) external;

    /// @dev Remove vote in poll
    /// @custom:selector 79cae220
    /// @param pollIndex Index of the poll
    function removeVote(uint32 pollIndex) external;

    /// @dev Remove vote in poll for other voter
    /// @custom:selector cbcb9276
    //// @param target The voter to have vote removed
    /// @param class The class
    /// @param pollIndex the poll index
    function removeOtherVote(
        address target,
        uint16 class,
        uint32 pollIndex
    ) external;

    /// @dev Delegate to a representative for the vote class
    /// @custom:selector 681750e8
    /// @param class The class
    /// @param representative The representative for the class
    /// @param conviction The conviction multiplier
    /// @param amount delegated to representative for this vote class
    function delegate(
        uint16 class,
        address representative,
        Conviction conviction,
        uint256 amount
    ) external;

    /// @dev Undelegate for the vote class
    /// @custom:selector 98be4094
    /// @param class The class
    function undelegate(uint16 class) external;

    /// @dev Unlock tokens locked for vote class
    /// @custom:selector 4259d98c
    /// @param class The class
    /// @param target The target address
    function unlock(uint16 class, address target) external;
}
