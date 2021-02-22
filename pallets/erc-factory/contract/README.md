# Solidity Code

## Current, Temporary Contract

This ERC-20 contract mints the maximum amount of tokens to the contract creator.

```
pragma solidity ^0.5.0;
import "https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v2.5.1/contracts/token/ERC20/ERC20.sol";
contract MyToken is ERC20 {
	constructor() public { _mint(msg.sender, 2**256 - 1); }
}
```

## Contract I'd like to Use

[in fork of open-zeppelin](https://github.com/4meta5/openzeppelin-contracts/commit/397945c6bb92fc5ead947772ac89f3a04c1562e5)

solidity code in `./ERC20PresetMinterBurner.sol`
