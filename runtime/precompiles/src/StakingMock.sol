// SPDX-License-Identifier: GPL-3.0-only

import "./StakingInterface.sol";

/// An in-EVM mock implementation of the parachain staking pallet.
/// This is ueful to test contracts such as the nomination DAO when you're unsure if the
/// precompile implementation is causing problems.
contract StakingMock is ParachainStaking {
    event JoinCandidatesCalled(uint256);

    function is_nominator(address) external view override returns (bool) {
        return true;
    }

    // Now the dispatchables

    /// Join the set of collator candidates
    function join_candidates(uint256 amount) external override {
        emit JoinCandidatesCalled(amount);
    }

    /// Request to leave the set of candidates. If successful, the account is immediately
    /// removed from the candidate pool to prevent selection as a collator, but unbonding is
    /// executed with a delay of `BondDuration` rounds.
    function leave_candidates() external override {}

    /// Temporarily leave the set of collator candidates without unbonding
    function go_offline() external override {}

    /// Rejoin the set of collator candidates if previously had called `go_offline`
    function go_online() external override {}

    /// Bond more for collator candidates
    function candidate_bond_more(uint256 more) external override {}

    /// Bond less for collator candidates
    function candidate_bond_less(uint256 less) external override {}

    /// If caller is not a nominator, then join the set of nominators
    /// If caller is a nominator, then makes nomination to change their nomination state
    function nominate(address collator, uint256 amount) external override {}

    /// Leave the set of nominators and, by implication, revoke all ongoing nominations
    function leave_nominators() external override {}

    /// Revoke an existing nomination
    function revoke_nomination(address collator) external override {}

    /// Bond more for nominators with respect to a specific collator candidate
    function nominator_bond_more(address candidate, uint256 more)
        external
        override
    {}

    /// Bond less for nominators with respect to a specific nominator candidate
    function nominator_bond_less(address candidate, uint256 less)
        external
        override
    {}
}
