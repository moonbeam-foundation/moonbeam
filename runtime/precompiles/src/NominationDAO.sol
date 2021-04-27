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

    /// Initialize a new NominationDao dedicated to nominating the given collator target.
    constructor(address _target) {
        target = _target;
        // This is the well-known address of Moonbeam's parachain staking precompile
        staking = ParachainStaking(0x0000000000000000000000000000000000000100);
    }

    /// Update the on-chain nomination to reflect any recently-contributed nominations.
    function update_nomination() public {
        // If we are already nominating, we need to remove the old nomination first
        if (staking.is_nominator(address(this))) {
            staking.revoke_nomination(target);
        }

        // If we have enough funds to nominate, we should start a nomination
        if (address(this).balance > MinNominatorStk) {
            staking.nominate(target, address(this).balance);
        } else {
            revert("NominationBelowMin");
        }
    }

    /// Calls directly into the interface.
    /// Assumes the contract has atleast 10 ether so that the nomination will be successful.
    function unsafe_attempt_to_nominate() public {
        staking.nominate(target, 10 ether);
    }

    // We need a public receive function to accept ether donations as direct transfers
    // https://blog.soliditylang.org/2020/03/26/fallback-receive-split/
    receive() external payable {
        // It would be nice to call update_nomination here so it happens automatically.
        // but there is very little gas available when just sending a normal transfer.
        // So instead we rely on manually calling update_nomination
    }
}
