// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.4.21 <0.7.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract Migrations is ERC20 {
  address public owner;
  uint public last_completed_migration;
  
  constructor() ERC20() public {
    owner = msg.sender;
    _mint(msg.sender,  2110000);
  }

  function setCompleted(uint completed) public restricted {
    last_completed_migration = completed;
  }
  modifier restricted() {
    if (msg.sender == owner) _;
  }

  function upgrade(address new_address) public restricted {
    Migrations upgraded = Migrations(new_address);
    upgraded.setCompleted(last_completed_migration);
  }
}