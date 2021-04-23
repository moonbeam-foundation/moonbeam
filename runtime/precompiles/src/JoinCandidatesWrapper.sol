// SPDX-License-Identifier: GPL-3.0-only.

pragma solidity >=0.8.0;
import "./StakingInterface.sol";

/// An even more dead simple example to call the precompile
contract JoinCandidatesWrapper {
    /// The ParachainStaking wrapper at the known pre-compile address. This will be used to make all calls
    /// to the underlying staking solution
    ParachainStaking public staking;

    /// Solely for debugging purposes
    event Trace(uint256);

    constructor(address _staking) {
        staking = ParachainStaking(_staking);
    }

    receive() external payable {}

    function join() public {
        emit Trace(1 << 248); // Shift them to the left-most byte so I can see them in Apps
        staking.join_candidates(1234 ether);
        emit Trace(2 << 248);
    }
}
