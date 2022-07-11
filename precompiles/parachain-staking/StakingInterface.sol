// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @author The Moonbeam Team
/// @title The interface through which solidity contracts will interact with Parachain Staking
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
/// Address :    0x0000000000000000000000000000000000000800
interface ParachainStaking {
    /// @dev Check whether the specified address is currently a staking delegator
    /// Selector: fd8ab482
    /// @param delegator the address that we want to confirm is a delegator
    /// @return A boolean confirming whether the address is a delegator
    function isDelegator(address delegator) external view returns (bool);

    /// @dev Check whether the specified address is currently a collator candidate
    /// Selector: d51b9e93
    /// @param candidate the address that we want to confirm is a collator andidate
    /// @return A boolean confirming whether the address is a collator candidate
    function isCandidate(address candidate) external view returns (bool);

    /// @dev Check whether the specifies address is currently a part of the active set
    /// Selector: 740d7d2a
    /// @param candidate the address that we want to confirm is a part of the active set
    /// @return A boolean confirming whether the address is a part of the active set
    function isSelectedCandidate(address candidate)
        external
        view
        returns (bool);

    /// @dev Total points awarded to all collators in a particular round
    /// Selector: 9799b4e7
    /// @param round the round for which we are querying the points total
    /// @return The total points awarded to all collators in the round
    function points(uint256 round) external view returns (uint256);

    /// @dev Get the minimum delegation amount
    /// Selector: 02985992
    /// @return The minimum delegation amount
    function minDelegation() external view returns (uint256);

    /// @dev Get the CandidateCount weight hint
    /// Selector: a9a981a3
    /// @return The CandidateCount weight hint
    function candidateCount() external view returns (uint256);

    /// @dev Get the current round number
    /// Selector: 146ca531
    /// @return The current round number
    function round() external view returns (uint256);

    /// @dev Get the CandidateDelegationCount weight hint
    /// Selector: 2ec087eb
    /// @param candidate The address for which we are querying the nomination count
    /// @return The number of nominations backing the collator
    function candidateDelegationCount(address candidate)
        external
        view
        returns (uint256);

    /// @dev Get the DelegatorDelegationCount weight hint
    /// Selector: 067ec822
    /// @param delegator The address for which we are querying the delegation count
    /// @return The number of delegations made by the delegator
    function delegatorDelegationCount(address delegator)
        external
        view
        returns (uint256);

    /// @dev Get the selected candidates for the current round
    /// Selector: bcf868a6
    /// @return The selected candidate accounts
    function selectedCandidates() external view returns (address[] memory);

    /// @dev Whether there exists a pending request for a delegation made by a delegator
    /// Selector: 3b16def8
    /// @param delegator the delegator that made the delegation
    /// @param candidate the candidate for which the delegation was made
    /// @return Whether a pending request exists for such delegation
    function delegationRequestIsPending(address delegator, address candidate)
        external
        view
        returns (bool);

    /// @dev Whether there exists a pending exit for candidate
    /// Selector: 43443682
    /// @param candidate the candidate for which the exit request was made
    /// @return Whether a pending request exists for such delegation
    function candidateExitIsPending(address candidate)
        external
        view
        returns (bool);

    /// @dev Whether there exists a pending bond less request made by a candidate
    /// Selector: d0deec11
    /// @param candidate the candidate which made the request
    /// @return Whether a pending bond less request was made by the candidate
    function candidateRequestIsPending(address candidate)
        external
        view
        returns (bool);

    /// @dev Join the set of collator candidates
    /// Selector: 1f2f83ad
    /// @param amount The amount self-bonded by the caller to become a collator candidate
    /// @param candidateCount The number of candidates in the CandidatePool
    function joinCandidates(uint256 amount, uint256 candidateCount) external;

    /// @dev Request to leave the set of collator candidates
    /// Selector: b1a3c1b7
    /// @param candidateCount The number of candidates in the CandidatePool
    function scheduleLeaveCandidates(uint256 candidateCount) external;

    /// @dev Execute due request to leave the set of collator candidates
    /// Selector: 3867f308
    /// @param candidate The candidate address for which the pending exit request will be executed
    /// @param candidateDelegationCount The number of delegations for the candidate to be revoked
    function executeLeaveCandidates(
        address candidate,
        uint256 candidateDelegationCount
    ) external;

    /// @dev Cancel request to leave the set of collator candidates
    /// Selector: 9c76ebb4
    /// @param candidateCount The number of candidates in the CandidatePool
    function cancelLeaveCandidates(uint256 candidateCount) external;

    /// @dev Temporarily leave the set of collator candidates without unbonding
    /// Selector: a6485ccd
    function goOffline() external;

    /// @dev Rejoin the set of collator candidates if previously had called `goOffline`
    /// Selector: 6e5b676b
    function goOnline() external;

    /// @dev Request to bond more for collator candidates
    /// Selector: a52c8643
    /// @param more The additional amount self-bonded
    function candidateBondMore(uint256 more) external;

    /// @dev Request to bond less for collator candidates
    /// Selector: 60744ae0
    /// @param less The amount to be subtracted from self-bond and unreserved
    function scheduleCandidateBondLess(uint256 less) external;

    /// @dev Execute pending candidate bond request
    /// Selector: 2e290290
    /// @param candidate The address for the candidate for which the request will be executed
    function executeCandidateBondLess(address candidate) external;

    /// @dev Cancel pending candidate bond request
    /// Selector: b5ad5f07
    function cancelCandidateBondLess() external;

    /// @dev Make a delegation in support of a collator candidate
    /// Selector: 829f5ee3
    /// @param candidate The address of the supported collator candidate
    /// @param amount The amount bonded in support of the collator candidate
    /// @param candidateDelegationCount The number of delegations in support of the candidate
    /// @param delegatorDelegationCount The number of existing delegations by the caller
    function delegate(
        address candidate,
        uint256 amount,
        uint256 candidateDelegationCount,
        uint256 delegatorDelegationCount
    ) external;

    /// schedule individual revokes instead
    /// @dev Request to leave the set of delegators
    /// Selector: f939dadb
    function scheduleLeaveDelegators() external;

    /// execute individual revokes instead
    /// @dev Execute request to leave the set of delegators and revoke all delegations
    /// Selector: fb1e2bf9
    /// @param delegator The leaving delegator
    /// @param delegatorDelegationCount The number of active delegations to be revoked by delegator
    function executeLeaveDelegators(
        address delegator,
        uint256 delegatorDelegationCount
    ) external;

    /// cancel individual revokes instead
    /// @dev Cancel request to leave the set of delegators
    /// Selector: f7421284
    function cancelLeaveDelegators() external;

    /// @dev Request to revoke an existing delegation
    /// Selector: 1a1c740c
    /// @param candidate The address of the collator candidate which will no longer be supported
    function scheduleRevokeDelegation(address candidate) external;

    /// @dev Bond more for delegators with respect to a specific collator candidate
    /// Selector: 0465135b
    /// @param candidate The address of the collator candidate for which delegation shall increase
    /// @param more The amount by which the delegation is increased
    function delegatorBondMore(address candidate, uint256 more) external;

    /// @dev Request to bond less for delegators with respect to a specific collator candidate
    /// Selector: c172fd2b
    /// @param candidate The address of the collator candidate for which delegation shall decrease
    /// @param less The amount by which the delegation is decreased (upon execution)
    function scheduleDelegatorBondLess(address candidate, uint256 less)
        external;

    /// @dev Execute pending delegation request (if exists && is due)
    /// Selector: e98c8abe
    /// @param delegator The address of the delegator
    /// @param candidate The address of the candidate
    function executeDelegationRequest(address delegator, address candidate)
        external;

    /// @dev Cancel pending delegation request (already made in support of input by caller)
    /// Selector: c90eee83
    /// @param candidate The address of the candidate
    function cancelDelegationRequest(address candidate) external;
}
