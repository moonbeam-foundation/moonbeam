// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract ERC20Sample is ERC20, ERC20Burnable, Ownable {
    constructor() ERC20("SampleToken", "SAM") Ownable() {
        _mint(msg.sender, 1000 * 10 ** decimals());
    }

    function greeter() public view returns (string memory) {
        return "Hello, ERC20!";
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }
}
