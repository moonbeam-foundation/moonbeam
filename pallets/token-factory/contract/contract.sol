// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/release-v4.0/contracts/access/Ownable.sol";
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/release-v4.0/contracts/token/ERC20/extensions/ERC20Burnable.sol";

/**
 * @dev {ERC20} token, including:
 *
 * The account that deploys the contract can mint and burn.
 */
contract ERC20MinterBurner is Ownable, ERC20Burnable {
    /**
     * @dev
     *
     * See {ERC20-constructor}.
     */
    constructor(string memory name, string memory symbol) ERC20(name, symbol) {}

    /**
     * @dev Creates `amount` new tokens for `to`.
     *
     * See {ERC20-_mint}.
     *
     * Requirements:
     *
     * - the caller must be the account that deployed the contract
     */
    function mint(address to, uint256 amount) public virtual onlyOwner {
        _mint(to, amount);
    }

    /**
     * @dev Burns `amount` tokens from `from`
     *
     * See {ERC20Burnable-_burn}.
     *
     * Requirements:
     *
     * - the caller must be the account that deployed the contract
     */
    function burn(address from, uint256 amount) public virtual onlyOwner {
        _burn(from, amount);
    }
}
