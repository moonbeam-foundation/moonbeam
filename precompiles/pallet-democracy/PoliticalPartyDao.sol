// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// Example contract to use and test the democracy interface.
///
/// A simple DAO where you can contribute eth and vote on a referendum. The DAO holds its own
/// internal vote on a pallet-democracy style referendum, and votes all DAO tokens accordingly.
///
/// Contributors who lock tokens in the contract are sacrificing the ability to otherwise
/// use the tokens (opportunity cost) as well as sacrificing their freedom to vote with those tokens.
/// They are accepting these costs for the potential of gaining more overall political influence by
/// means of non-member citizens delegating their own votes to the party.
///
/// This example contract is meant to explore the democracy interface more than innovate in terms of
/// elections and political parties, so there are a few limitations.
/// 1. This DAO is for a _single_ referendum.
/// 2. It uses a simple majority rules mechanism. Definitely not production ready...
/// 3. Early withdrawal penalties are just locked in the contract forever.
/// 4. We just use a simple mapping to store everything. A better idea might be fungible dao tokens
contract PoliticalPartyDao {
    mapping(address => Contribution) public contributions;

    constructor(uint256 proposalHash) {
        // Make sure the proposal exists
    }

    /// @dev Contribute funds to the dao and specify your initial vote
    ///
    /// @param vote Whether you support the referendum (true).
    /// @custom:selector 40656963
    function contribute(bool vote) external payable {}

    /// @dev Change your currently registered vote in the contract
    /// @custom:selector bc55efbf
    function changeVote() external {}

    /// @dev Withdraw your tokens after the referendum finishes.
    /// @custom:selector 3ccfd60b
    function withdraw() external {}

    /// @dev Allows a user who is unhappy with the status of the internal party vote to withdraw tokens.
    /// However there is a 10% penalty for being fickle. The 10% of tokens that remain in the DAO
    /// are jsut locked here forever. A real-life impl could explicitly burn or redistribute.
    /// @custom:selector 2fc54f73
    function withdrawEarly() external {
        // Does this actually help explore the democracy interface? If not, don't implement it.
    }
}

/// Information about a contribution to the DAO including the amount contributed and the vote.
struct Contribution {
    uint256 amount;
    bool vote;
}
