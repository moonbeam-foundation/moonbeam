// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The CrowdloanRewards contract's address.
address constant CROWDLOAN_REWARDS_ADDRESS = 0x0000000000000000000000000000000000000801;

/// @dev The CrowdloanRewards contract's instance.
CrowdloanRewards constant CROWDLOAN_REWARDS_CONTRACT = CrowdloanRewards(
    CROWDLOAN_REWARDS_ADDRESS
);

/// @author The Moonbeam Team
/// @title Pallet Crowdloan Rewards Interface
/// @dev The interface through which solidity contracts will interact with Crowdloan Rewards. We
/// follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
/// @custom:address 0x0000000000000000000000000000000000000801
interface CrowdloanRewards {
    // First some simple accessors

    /// @dev Checks whether the address is a contributor
    /// @param contributor the address that we want to confirm is a contributor
    /// @return A boolean confirming whether the address is a contributor
    /// @custom:selector 1d0d35f5
    function isContributor(address contributor) external view returns (bool);

    /// @dev Retrieve total reward and claimed reward for an address
    /// @param contributor the address for which we want to retrieve the information
    /// @return A u256 tuple, reflecting (totalRewards, claimedRewards)
    /// @custom:selector cbecf6b5
    function rewardInfo(address contributor)
        external
        view
        returns (uint256, uint256);

    // Now the dispatchables

    /// @dev Claim the vested amount from the crowdloan rewards
    /// @custom:selector 4e71d92d
    function claim() external;

    /// @dev Update reward address to receive crowdloan rewards
    /// @param newAddress, the newAddress where to receive the rewards from now on
    /// @custom:selector 944dd5a2
    function updateRewardAddress(address newAddress) external;
}

// These are the selectors generated by remix following this advice
// https://ethereum.stackexchange.com/a/73405/9963
// Eventually we will probably want a better way of generating these and copying them to Rust

//{
//    "53440c90": "isContributor(address)"
//    "76f70249": "rewardInfo(address)"
//    "4e71d92d": "claim()"
//    "aaac61d6": "updateRewardAddress(address)"
//}