// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The ParachainStaking contract's address.
address constant PARACHAIN_STAKING_ADDRESS = 0x0000000000000000000000000000000000000800;

/// @dev The ParachainStaking contract's instance.
ParachainStaking constant PARACHAIN_STAKING_CONTRACT = ParachainStaking(
    PARACHAIN_STAKING_ADDRESS
);

/// @author The Moonbeam Team
/// @title Pallet Parachain Staking Interface
/// @dev The interface through which solidity contracts will interact with Parachain Staking
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
/// @custom:address 0x0000000000000000000000000000000000000800
interface ParachainStaking {
    /// @dev Check whether the specified address is currently a staking delegator
    /// @custom:selector fd8ab482
    /// @param delegator the address that we want to confirm is a delegator
    /// @return A boolean confirming whether the address is a delegator
    function isDelegator(address delegator) external view returns (bool);

    /// @dev Check whether the specified address is currently a collator candidate
    /// @custom:selector d51b9e93
    /// @param candidate the address that we want to confirm is a collator andidate
    /// @return A boolean confirming whether the address is a collator candidate
    function isCandidate(address candidate) external view returns (bool);

    /// @dev Check whether the specifies address is currently a part of the active set
    /// @custom:selector 740d7d2a
    /// @param candidate the address that we want to confirm is a part of the active set
    /// @return A boolean confirming whether the address is a part of the active set
    function isSelectedCandidate(
        address candidate
    ) external view returns (bool);

    /// @dev Total points awarded to all collators in a particular round
    /// @custom:selector 9799b4e7
    /// @param round the round for which we are querying the points total
    /// @return The total points awarded to all collators in the round
    function points(uint256 round) external view returns (uint256);

    /// @dev Total points awarded to a specific collator in a particular round.
    /// A value of `0` may signify that no blocks were produced or that the storage for that round has been removed
    /// @custom:selector bfea66ac
    /// @param round the round for which we are querying the awarded points
    /// @param candidate The candidate to whom the points are awarded
    /// @return The total points awarded to the collator for the provided round
    function awardedPoints(
        uint32 round,
        address candidate
    ) external view returns (uint32);

    /// @dev The amount delegated in support of the candidate by the delegator
    /// @custom:selector a73e51bc
    /// @param delegator Who made this delegation
    /// @param candidate The candidate for which the delegation is in support of
    /// @return The amount of the delegation in support of the candidate by the delegator
    function delegationAmount(
        address delegator,
        address candidate
    ) external view returns (uint256);

    /// @dev Whether the delegation is in the top delegations
    /// @custom:selector 91cc8657
    /// @param delegator Who made this delegation
    /// @param candidate The candidate for which the delegation is in support of
    /// @return If delegation is in top delegations (is counted)
    function isInTopDelegations(
        address delegator,
        address candidate
    ) external view returns (bool);

    /// @dev Get the minimum delegation amount
    /// @custom:selector 02985992
    /// @return The minimum delegation amount
    function minDelegation() external view returns (uint256);

    /// @dev Get the CandidateCount weight hint
    /// @custom:selector a9a981a3
    /// @return The CandidateCount weight hint
    function candidateCount() external view returns (uint256);

    /// @dev Get the current round number
    /// @custom:selector 146ca531
    /// @return The current round number
    function round() external view returns (uint256);

    /// @dev Get the CandidateDelegationCount weight hint
    /// @custom:selector 2ec087eb
    /// @param candidate The address for which we are querying the nomination count
    /// @return The number of nominations backing the collator
    function candidateDelegationCount(
        address candidate
    ) external view returns (uint32);

    /// @dev Get the CandidateAutoCompoundingDelegationCount weight hint
    /// @custom:selector 905f0806
    /// @param candidate The address for which we are querying the auto compounding
    ///     delegation count
    /// @return The number of auto compounding delegations
    function candidateAutoCompoundingDelegationCount(
        address candidate
    ) external view returns (uint32);

    /// @dev Get the DelegatorDelegationCount weight hint
    /// @custom:selector 067ec822
    /// @param delegator The address for which we are querying the delegation count
    /// @return The number of delegations made by the delegator
    function delegatorDelegationCount(
        address delegator
    ) external view returns (uint256);

    /// @dev Get the selected candidates for the current round
    /// @custom:selector bcf868a6
    /// @return The selected candidate accounts
    function selectedCandidates() external view returns (address[] memory);

    /// @dev Whether there exists a pending request for a delegation made by a delegator
    /// @custom:selector 3b16def8
    /// @param delegator the delegator that made the delegation
    /// @param candidate the candidate for which the delegation was made
    /// @return Whether a pending request exists for such delegation
    function delegationRequestIsPending(
        address delegator,
        address candidate
    ) external view returns (bool);

    /// @dev Whether there exists a pending exit for candidate
    /// @custom:selector 43443682
    /// @param candidate the candidate for which the exit request was made
    /// @return Whether a pending request exists for such delegation
    function candidateExitIsPending(
        address candidate
    ) external view returns (bool);

    /// @dev Whether there exists a pending bond less request made by a candidate
    /// @custom:selector d0deec11
    /// @param candidate the candidate which made the request
    /// @return Whether a pending bond less request was made by the candidate
    function candidateRequestIsPending(
        address candidate
    ) external view returns (bool);

    /// @dev Returns the percent value of auto-compound set for a delegation
    /// @custom:selector b4d4c7fd
    /// @param delegator the delegator that made the delegation
    /// @param candidate the candidate for which the delegation was made
    /// @return Percent of rewarded amount that is auto-compounded on each payout
    function delegationAutoCompound(
        address delegator,
        address candidate
    ) external view returns (uint8);

    /// @dev Join the set of collator candidates
    /// @custom:selector 1f2f83ad
    /// @param amount The amount self-bonded by the caller to become a collator candidate
    /// @param candidateCount The number of candidates in the CandidatePool
    function joinCandidates(uint256 amount, uint256 candidateCount) external;

    /// @dev Request to leave the set of collator candidates
    /// @custom:selector b1a3c1b7
    /// @param candidateCount The number of candidates in the CandidatePool
    function scheduleLeaveCandidates(uint256 candidateCount) external;

    /// @dev Execute due request to leave the set of collator candidates
    /// @custom:selector 3867f308
    /// @param candidate The candidate address for which the pending exit request will be executed
    /// @param candidateDelegationCount The number of delegations for the candidate to be revoked
    function executeLeaveCandidates(
        address candidate,
        uint256 candidateDelegationCount
    ) external;

    /// @dev Cancel request to leave the set of collator candidates
    /// @custom:selector 9c76ebb4
    /// @param candidateCount The number of candidates in the CandidatePool
    function cancelLeaveCandidates(uint256 candidateCount) external;

    /// @dev Temporarily leave the set of collator candidates without unbonding
    /// @custom:selector a6485ccd
    function goOffline() external;

    /// @dev Rejoin the set of collator candidates if previously had called `goOffline`
    /// @custom:selector 6e5b676b
    function goOnline() external;

    /// @dev Request to bond more for collator candidates
    /// @custom:selector a52c8643
    /// @param more The additional amount self-bonded
    function candidateBondMore(uint256 more) external;

    /// @dev Request to bond less for collator candidates
    /// @custom:selector 60744ae0
    /// @param less The amount to be subtracted from self-bond and unreserved
    function scheduleCandidateBondLess(uint256 less) external;

    /// @dev Execute pending candidate bond request
    /// @custom:selector 2e290290
    /// @param candidate The address for the candidate for which the request will be executed
    function executeCandidateBondLess(address candidate) external;

    /// @dev Cancel pending candidate bond request
    /// @custom:selector b5ad5f07
    function cancelCandidateBondLess() external;

    /// @dev Make a delegation in support of a collator candidate
    /// @custom:selector 4b8bc9bf
    /// @param candidate The address of the supported collator candidate
    /// @param amount The amount bonded in support of the collator candidate
    /// @param autoCompound The percent of reward that should be auto-compounded
    /// @param candidateDelegationCount The number of delegations in support of the candidate
    /// @param candidateAutoCompoundingDelegationCount The number of auto-compounding delegations
    /// in support of the candidate
    /// @param delegatorDelegationCount The number of existing delegations by the caller
    function delegateWithAutoCompound(
        address candidate,
        uint256 amount,
        uint8 autoCompound,
        uint256 candidateDelegationCount,
        uint256 candidateAutoCompoundingDelegationCount,
        uint256 delegatorDelegationCount
    ) external;

    /// @dev Request to revoke an existing delegation
    /// @custom:selector 1a1c740c
    /// @param candidate The address of the collator candidate which will no longer be supported
    function scheduleRevokeDelegation(address candidate) external;

    /// @dev Bond more for delegators with respect to a specific collator candidate
    /// @custom:selector 0465135b
    /// @param candidate The address of the collator candidate for which delegation shall increase
    /// @param more The amount by which the delegation is increased
    function delegatorBondMore(address candidate, uint256 more) external;

    /// @dev Request to bond less for delegators with respect to a specific collator candidate
    /// @custom:selector c172fd2b
    /// @param candidate The address of the collator candidate for which delegation shall decrease
    /// @param less The amount by which the delegation is decreased (upon execution)
    function scheduleDelegatorBondLess(
        address candidate,
        uint256 less
    ) external;

    /// @dev Execute pending delegation request (if exists && is due)
    /// @custom:selector e98c8abe
    /// @param delegator The address of the delegator
    /// @param candidate The address of the candidate
    function executeDelegationRequest(
        address delegator,
        address candidate
    ) external;

    /// @dev Cancel pending delegation request (already made in support of input by caller)
    /// @custom:selector c90eee83
    /// @param candidate The address of the candidate
    function cancelDelegationRequest(address candidate) external;

    /// @dev Sets an auto-compound value for a delegation
    /// @custom:selector faa1786f
    /// @param candidate The address of the supported collator candidate
    /// @param value The percent of reward that should be auto-compounded
    /// @param candidateAutoCompoundingDelegationCount The number of auto-compounding delegations
    /// in support of the candidate
    /// @param delegatorDelegationCount The number of existing delegations by the caller
    function setAutoCompound(
        address candidate,
        uint8 value,
        uint256 candidateAutoCompoundingDelegationCount,
        uint256 delegatorDelegationCount
    ) external;

    /// @dev Fetch the total staked amount of a delegator, regardless of the
    /// candidate.
    /// @custom:selector e6861713
    /// @param delegator Address of the delegator.
    /// @return Total amount of stake.
    function getDelegatorTotalStaked(
        address delegator
    ) external view returns (uint256);

    /// @dev Fetch the total staked towards a candidate.
    /// @custom:selector bc5a1043
    /// @param candidate Address of the candidate.
    /// @return Total amount of stake.
    function getCandidateTotalCounted(
        address candidate
    ) external view returns (uint256);
}
