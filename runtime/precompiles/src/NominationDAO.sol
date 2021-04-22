// SPDX-License-Identifier: GPL-3.0-only

// This is a PoC to use the staking precompile wrapper as a Solidity developer.

pragma solidity >=0.8.0;

import "./ParachainStakingWrapper.sol";

contract NominationDao {
    // TODO Our precompile should have an accessor for this.
    uint256 constant MinNominatorStk = 5 ether;

    /// The collator that this DAO is currently nominating
    address public target;

    /// The account that deployed this DAO. Currently this deployer chooses the nomination target
    /// Once the basic PoC stuff is done we can add cooler actual DAO logic. But for now we want it simple
    address public deployer;

    /// The ParachainStaking wrapper at the known pre-compile address. This will be used to make all calls
    /// to the underlying staking solution
    ParachainStaking public precompile;

    /// Whether this DAO currently has a nomination active
    /// TODO Our precompile should have an accessor for this that wraps the pallet's in_nominator.
    bool public isNominating;

    //TODO Do we need this constructor at all, or can we initialize this stuff directly.
    constructor() {
        // Initialize the deployer and the
        target = msg.sender;
        deployer = msg.sender;

        // Instantiate the parachain staking wrapper at the known precompile address (decimal 256)
        precompile = ParachainStaking(
            0x0000000000000000000000000000000000000100
        );

        // We won't nominate until
        isNominating = false;
    }

    /// Change the current nomination target.
    /// Maybe this won't be necessary for the demo, but it's here for now.
    function change_target(address new_target) public {
        // Check that the deployer is calling
        require(msg.sender == deployer);

        // Update storage.
        target = new_target;

        // If we already have an active nomination, switch it.
        if (isNominating) {
            // TODO revoke old nomination via precompile
            // TODO submit new nomination via precompile
        }
    }

    /// Contribute some funds to the nomination contract
    /// TODO isn't there a "fallback" function for when people just send eth?
    /// Maybe we should use that instead of this `contribute`
    function contribute() public payable {
        bool min_nomination_met = address(this).balance > MinNominatorStk;

        if (isNominating) {
            //TODO call nominate_more precompile
        } else if (min_nomination_met) {
            //TODO call nominate precompile

            // Note that we have an active nomination
            isNominating = true;
        }
    }
}
