// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Contracts ^5.0.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts@5.0.2/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts@5.0.2/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts@5.0.2/token/ERC20/extensions/ERC20Pausable.sol";
import "@openzeppelin/contracts@5.0.2/access/Ownable.sol";
import "@openzeppelin/contracts@5.0.2/token/ERC20/extensions/ERC20Permit.sol";

contract MyToken is ERC20, ERC20Pausable, Ownable, ERC20Permit {
    constructor(address initialOwner, uint8 tokenDecimals, string memory ticker, string memory tokenName)
        ERC20(tokenName, ticker)
        Ownable(initialOwner)
        ERC20Permit(tokenName)
    {
      _decimals = tokenDecimals;
    }
    
    uint8 private _decimals;

    function pause() public onlyOwner {
        _pause();
    }

    function unpause() public onlyOwner {
        _unpause();
    }

    function mintInto(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }

    function burnFrom(address from, uint256 amount) public onlyOwner {
        _burn(from, amount);
    }

    function burnAllFrom(address account) public onlyOwner {
        _burn(account, balanceOf(account));
    }
    
    function decimals() public view override returns (uint8) {
        return _decimals;
    }

    // The following functions are overrides required by Solidity.

    function _update(address from, address to, uint256 value)
        internal
        override(ERC20, ERC20Pausable)
    {
        super._update(from, to, value);
    }
}