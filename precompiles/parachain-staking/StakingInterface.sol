// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @author The Moonbeam Team
/// @title The interface through which solidity contracts will interact with Parachain Staking
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
interface ParachainStaking {
    // First some simple accessors

    /// @dev Check whether the specified address is currently a staking nominator
    /// @param nominator the address that we want to confirm is a nominator
    /// @return A boolean confirming whether the address is a nominator
    function is_nominator(address nominator) external view returns (bool);

    /// @dev Check whether the specified address is currently a collator candidate
    /// @param collator the address that we want to confirm is a collator andidate
    /// @return A boolean confirming whether the address is a collator candidate
    function is_candidate(address collator) external view returns (bool);

    /// @dev Check whether the specifies address is currently a part of the active set
    /// @param collator the address that we want to confirm is a part of the active set
    /// @return A boolean confirming whether the address is a part of the active set
    function is_selected_candidate(address collator)
        external
        view
        returns (bool);

    /// @dev Total points awarded to all collators in a particular round
    /// @param round the round for which we are querying the points total
    /// @return The total points awarded to all collators in the round
    function points(uint256 round) external view returns (uint256);

    /// @dev Get the minimum nomination amount
    /// @return The minimum nomination amount
    function min_nomination() external view returns (uint256);

    /// @dev Get the CandidateCount weight hint
    /// @return The CandidateCount weight hint
    function candidate_count() external view returns (uint256);

    /// @dev Get the CollatorNominationCount weight hint
    /// @param collator The address for which we are querying the nomination count
    /// @return The number of nominations backing the collator
    function collator_nomination_count(address collator)
        external
        view
        returns (uint256);

    /// @dev Get the NominatorNominationCount weight hint
    /// @param nominator The address for which we are querying the nomination count
    /// @return The number of nominations made by the nominator
    function nominator_nomination_count(address nominator)
        external
        view
        returns (uint256);

    // Now the dispatchables

    /// @dev Join the set of collator candidates
    /// @param amount The amount self-bonded by the caller to become a collator candidate
    /// @param candidateCount The number of candidates in the CandidatePool
    function join_candidates(uint256 amount, uint256 candidateCount) external;

    /// @dev Leave the set of collator candidates
    /// @param candidateCount The number of candidates in the CandidatePool
    function leave_candidates(uint256 amount, uint256 candidateCount) external;

    /// @dev Temporarily leave the set of collator candidates without unbonding
    function go_offline() external;

    /// @dev Rejoin the set of collator candidates if previously had called `go_offline`
    function go_online() external;

    /// @dev Bond more for collator candidates
    /// @param more The additional amount self-bonded
    function candidate_bond_more(uint256 more) external;

    /// @dev Bond less for collator candidates
    /// @param less The amount to be subtracted from self-bond and unreserved
    function candidate_bond_less(uint256 less) external;

    /// @dev Make a nomination in support of a collator candidate
    /// @param collator The address of the supported collator candidate
    /// @param amount The amount bonded in support of the collator candidate
    /// @param collatorNominationCount The number of nominations in support of the candidate
    /// @param nominatorNominationCount The number of existing nominations by the caller
    function nominate(
        address collator,
        uint256 amount,
        uint256 collatorNominationCount,
        uint256 nominatorNominationCount
    ) external;

    /// @dev Leave the set of nominators and, by implication, revoke all ongoing nominations
    /// @param nominatorNominationCount The number of existing nominations to be revoked by caller
    function leave_nominators(uint256 nominatorNominationCount) external;

    /// @dev Revoke an existing nomination
    /// @param collator The address of the collator candidate which will no longer be supported
    function revoke_nomination(address collator) external;

    /// @dev Bond more for nominators with respect to a specific collator candidate
    /// @param candidate The address of the collator candidate for which nomination is increased
    /// @param more The amount by which the nomination is increased
    function nominator_bond_more(address candidate, uint256 more) external;

    /// @dev Bond less for nominators with respect to a specific collator candidate
    /// @param candidate The address of the collator candidate for which nomination is decreased
    /// @param less The amount by which the nomination is decreased
    function nominator_bond_less(address candidate, uint256 less) external;
}
