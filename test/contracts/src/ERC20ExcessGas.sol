// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "./ERC20WithInitialSupply.sol";

contract ERC20ExcessGas is ERC20WithInitialSupply {
    uint public _gasHog;

    constructor(
        string memory name_,
        string memory symbol_,
        address initialAccount,
        uint256 initialSupply
    ) ERC20WithInitialSupply(name_, symbol_, initialAccount, initialSupply) {}

    function transfer(
        address to,
        uint256 amount
    ) public override returns (bool) {
        // Consume gas to over Erc20XcmBridgeTransferGasLimit
        for (uint i = 0; i < 2200; i++) {
            _gasHog += i;
        }

        address owner = msg.sender;
        _transfer(owner, to, amount);
        return true;
    }
}
