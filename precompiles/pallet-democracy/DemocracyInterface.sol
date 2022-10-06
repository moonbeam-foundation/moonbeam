// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @author The Moonbeam Team
/// @title Pallet Democracy Interface
/// @dev The interface through which solidity contracts will interact with pallet-democracy.
/// This interface does not exhaustively wrap pallet democracy, rather it wraps the most
/// important parts and the parts that are expected to be most useful to evm contracts.
/// More exhaustive wrapping can be added later if it is desireable and the pallet interface
/// is deemed sufficiently stable.
/// @custom:address 0x0000000000000000000000000000000000000803
interface Democracy {
    // First some simple accessors

    /// Get The total number of public proposals past and present
    /// @custom:selector 31305462
    ///
    /// @return The total number of public proposals past and present
    function publicPropCount() external view returns (uint256);

    /// Get details about all public porposals.
    /// @custom:selector
    /// @return (prop index, proposal hash, proposer)
    /// TODO This is supposed to be a vec. Let's save this one for later.
    // function publicProps()
    //     external
    //     view
    //     returns (
    //         uint256,
    //         bytes32,
    //         address
    //     );

    /// Get the total amount locked behind a proposal.
    /// @custom:selector 4767142d
    ///
    /// @dev Unlike the similarly-named Rust function this one only returns the value, not the
    /// complete list of backers.
    /// @param propIndex The index of the proposal you are interested in
    /// @return The amount of tokens locked behind the proposal
    function depositOf(uint256 propIndex) external view returns (uint256);

    /// Get the index of the lowest unbaked referendum
    /// @custom:selector d49dccf0
    ///
    /// @return The lowest referendum index representing an unbaked referendum.
    function lowestUnbaked() external view returns (uint256);

    /// Get the details about an ongoing referendum.
    /// @custom:selector f033b7cd
    ///
    /// @dev This, along with `finishedReferendumInfo`, wraps the pallet's `referendumInfo`
    /// function. It is necessary to split it into two here because Solidity only has c-style enums.
    /// @param refIndex The index of the referendum you are interested in
    /// @return A tuple including:
    ///  * The block in which the referendum ended
    ///  * The proposal hash
    ///  * The baising mechanism 0-SuperMajorityApprove, 1-SuperMajorityAgainst, 2-SimpleMajority
    ///  * The delay between passing and launching
    ///  * The total aye vote (including conviction)
    ///  * The total nay vote (including conviction)
    ///  * The total turnout (not including conviction)
    function ongoingReferendumInfo(uint32 refIndex)
        external
        view
        returns (
            uint256,
            bytes32,
            uint8,
            uint256,
            uint256,
            uint256,
            uint256
        );

    /// Get the details about a finished referendum.
    /// @custom:selector c75abcce
    ///
    /// @dev This, along with `ongoingReferendumInfo`, wraps the pallet's `referendumInfo`
    /// function. It is necessary to split it into two here because Solidity only has c-style enums.
    /// @param refIndex The index of the referendum you are interested in
    /// @return A tuple including whether the referendum passed, and the block at which it finished.
    function finishedReferendumInfo(uint32 refIndex)
        external
        view
        returns (bool, uint256);

    // Now the dispatchables

    /// Make a new proposal
    /// @custom:selector 7824e7d1
    ///
    /// @param proposalHash The hash of the proposal you are making
    /// @param value The number of tokens to be locked behind this proposal.
    function propose(bytes32 proposalHash, uint256 value) external;

    /// Signal agreement with a proposal
    /// @custom:selector c7a76601
    ///
    /// @dev No amount is necessary here. Seconds are always for the same amount that the original
    /// proposer locked. You may second multiple times.
    ///
    /// @param propIndex index of the proposal you want to second
    /// @param secondsUpperBound A number greater than or equal to the current number of seconds.
    /// This is necessary for calculating the weight of the call.
    function second(uint256 propIndex, uint256 secondsUpperBound) external;

    //TODO should we have an alternative `simpleSecond` where the upper bound is read from storage?

    /// Vote in a referendum.
    /// @custom:selector 6cd18b0d
    ///
    /// @param refIndex index of the referendum you want to vote in
    /// @param aye `true` is a vote to enact the proposal; `false` is a vote to keep the status quo.
    /// @param voteAmount The number of tokens you are willing to lock if you get your way
    /// @param conviction How strongly you want to vote. Higher conviction means longer lock time.
    /// This must be an interget in the range 0 to 6
    ///
    /// @dev This function only supposrts `Standard` votes where you either vote aye xor nay.
    /// It does not support `Split` votes where you vote on both sides. If such a need
    /// arises, we should add an additional function to this interface called `splitVote`.
    function standardVote(
        uint256 refIndex,
        bool aye,
        uint256 voteAmount,
        uint256 conviction
    ) external;

    /// Remove a vote for a referendum.
    /// @custom:selector 3f68fde4
    ///
    /// @dev Locks get complex when votes are removed. See pallet-democracy's docs for details.
    /// @param refIndex The index of the referendum you are interested in
    function removeVote(uint256 refIndex) external;

    /// Delegate voting power to another account.
    /// @custom:selector 0185921e
    ///
    /// @dev The balance delegated is locked for as long as it is delegated, and thereafter for the
    /// time appropriate for the conviction's lock period.
    /// @param representative The account to whom the vote shall be delegated.
    /// @param conviction The conviction with which you are delegating. This conviction is used for
    /// All_ delegated votes.
    /// @param amount The number of tokens whose voting power shall be delegated.
    function delegate(
        address representative,
        uint256 conviction,
        uint256 amount
    ) external;

    /// Undelegatehe voting power
    /// @custom:selector 1eef225c
    ///
    /// @dev Tokens may be unlocked once the lock period corresponding to the conviction with which
    /// the delegation was issued has elapsed.
    function unDelegate() external;

    /// Unlock tokens that have an expired lock.
    /// @custom:selector 2f6c493c
    ///
    /// @param target The account whose tokens should be unlocked. This may be any account.
    function unlock(address target) external;

    /// Register the preimage for an upcoming proposal. This doesn't require the proposal to be
    /// in the dispatch queue but does require a deposit, returned once enacted.
    /// @custom:selector cb00f603
    ///
    /// @param encodedProposal The scale-encoded proposal whose hash has been submitted on-chain.
    function notePreimage(bytes memory encodedProposal) external;

    /// Register the preimage for an upcoming proposal. This requires the proposal to be
    /// in the dispatch queue. No deposit is needed. When this call is successful, i.e.
    /// the preimage has not been uploaded before and matches some imminent proposal,
    /// no fee is paid.
    /// @custom:selector 974791e3
    ///
    /// @param encodedProposal The scale-encoded proposal whose hash has been submitted on-chain.
    function noteImminentPreimage(bytes memory encodedProposal) external;
}
