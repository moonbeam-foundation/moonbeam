// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @notice Smart contract to help test state override
contract StateOverrideTest {
    /// @notice The maxmium allowed value
    uint256 public MAX_ALLOWED = 3;

    uint256 public availableFunds;

    /// @notice The owner of the contract
    address owner;

    constructor(uint256 intialAmount) payable {
        owner = msg.sender;
        availableFunds = intialAmount;
    }

    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }
}
