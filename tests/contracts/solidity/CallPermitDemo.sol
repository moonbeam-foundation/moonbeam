// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

import "../../../precompiles/call-permit/CallPermit.sol";

/// @notice Smart contract to demonstrate how to use requestLocalVRFRandomWords
contract CallPermitDemo {
    /// @notice The CallPermit Precompile Interface
    CallPermit public callPermit =
        CallPermit(0x000000000000000000000000000000000000080a);

    /// @notice The total pooled amount
    uint256 public bondedAmount;

    /// @notice Tracks per voting round, per role type, the total votes a participant has received
    mapping(address => uint256) bonds;

    /// @notice The owner of the contract
    address owner;

    constructor() {
        owner = msg.sender;
        bondedAmount = 0;
    }

    function bondFor(
        address from,
        uint256 value,
        bytes memory data,
        uint64 gaslimit,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        callPermit.dispatch(
            from,
            address(this),
            value,
            data,
            gaslimit,
            deadline,
            v,
            r,
            s
        );
    }

    function bond() external payable {
        address sender = msg.sender;
        uint256 amount = msg.value;
        bonds[sender] += amount;
        bondedAmount += amount;
    }

    function unbond(uint256 amount) external onlyOwner {
        address payable sender = payable(msg.sender);

        // check underflow
        bonds[sender] -= amount;
        bondedAmount -= amount;

        sender.transfer(amount);
    }

    modifier onlyOwner() {
        require(msg.sender == owner);
        _;
    }
}
