// SPDX-License-Identifier: GPL-3.0-only
// This is a PoC to use the staking precompile wrapper as a Solidity developer.
pragma solidity >=0.8.0;
import "./StakingInterface.sol";

contract NominationDao {
    // TODO Our interface should have an accessor for this.
    uint256 constant MinNominatorStk = 5 ether;

    /// The collator that this DAO is currently nominating
    address public target;

    /// The ParachainStaking wrapper at the known pre-compile address. This will be used to make all calls
    /// to the underlying staking solution
    ParachainStaking public staking;

    /// Whether this DAO currently has a nomination active
    /// TODO Our precompile should have an accessor for this that wraps the pallet's in_nominator.
    bool public isNominating = false;

    /// Solely for debugging purposes
    event Trace(uint256);

    /// Initialize a new NominationDao dedicated to nominating the given collator target.
    constructor(address _target, address _staking) {
        target = _target;
        staking = ParachainStaking(_staking);
    }

    /// Update the on-chain nomination to reflect any recently-contributed nominations.
    function update_nomination() public {
        emit Trace(1);
        emit Trace(MinNominatorStk);
        emit Trace(address(this).balance);

        //TODO call precompile accessor to check if you're nominating.
        if (isNominating) {
            emit Trace(2);
            //TODO figure out how much more to bond.
            // Maybe we need more accessors, or to keep track of how much we have bonded so far.
            // Nominate more toward the same existing target
            // staking.nominator_bond_more(target, msg.value);
            // TODO it seems like this balance comparison isn't working correctly.
        } else if (address(this).balance > MinNominatorStk) {
            emit Trace(3);
            // Make our nomination
            staking.nominate(target, address(this).balance);
            emit Trace(4);
            // Note that we have an active nomination
            // TODO I guess we should confirm that the precompile call was successful first.
            isNominating = true;
        } else {
            emit Trace(1024);
        }
    }

    /// Calls directly into the interface.
    /// Assumes the contract has atleast 10 ether so that the nomination will be successful.
    function unsafe_attempt_to_nominate() public {
        staking.nominate(target, 10 ether);
    }

    // So the notion of fallback funtion got split
    // https://blog.soliditylang.org/2020/03/26/fallback-receive-split/
    // Maybe I don't need any fallback function at all. Can I just receive ether?
    receive() external payable {
        // It would be nice to call update_nomination here s it happens automatically.
        // but there was some note about limited gas being available. We wouldn't want
        // running out of gas to be the thing that prevented us from accepting a donation.
        // If we still get the funds even when we run out of gas, then I don't see any harm
        // in triggering the update here.
    }
}
