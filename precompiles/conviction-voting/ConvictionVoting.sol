// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Conviction Voting contract's address. TODO: UPDATE ADDRESS
address constant Conviction_Voting_ADDRESS = 0x000000000000000000000000000000000000080b;

/// @dev The Conviction Voting contract's instance.
ConvictionVoting constant Conviction_Voting_CONTRACT = ConvictionVoting(
    Conviction_Voting_ADDRESS
);

/// @author The Moonbeam Team
/// @title Pallet Conviction Voting Interface
/// @title The interface through which solidity contracts will interact with the Conviction Voting pallet
/// @custom:address 0x000000000000000000000000000000000000080b TODO: UPDATE ADDRESS
interface ConvictionVoting {
    /// @dev Vote in a poll.
    /// @custom:selector 74a34dd3
    /// @param pollIndex Index of poll
    /// @param aye Yes or no vote
    /// @param voteAmount Balance locked for vote
    /// @param conviction Conviction multiplier for length of vote lock
    function standard_vote(
        uint256 pollIndex,
        bool aye,
        uint256 voteAmount,
        uint256 conviction
    ) external;

    /// @dev Remove vote in poll
    /// @custom:selector 74a34dd3
    /// @param pollIndex Index of the poll
    function remove_vote(uint256 pollIndex) external;

    /// @dev Remove vote in poll for other voter
    /// @custom:selector 74a34dd3
    /// @param  class The class
    /// @param pollIndex the poll index
    function remove_other_vote(uint256 class, uint256 pollIndex) external;

    /// @dev Delegate to a representative for the vote class
    /// @custom:selector 74a34dd3
    /// @param class The class
    /// @param representative The representative for the class
    /// @param conviction The conviction multiplier
    /// @param amount delegated to representative for this vote class
    function delegate(
        uint256 class,
        address representative,
        uint256 conviction,
        uint256 amount
    ) external;

    /// @dev Undelegate for the vote class
    /// @custom:selector 74a34dd3
    /// @param class The class
    function undelegate(uint256 class) external;

    /// @dev Unlock tokens locked for vote class
    /// @custom:selector 74a34dd3
    /// @param class The class
    /// @param target The target address
    function unlock(uint256 class, address target) external;
}
