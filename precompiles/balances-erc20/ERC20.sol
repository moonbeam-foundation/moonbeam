pragma solidity ^0.4.24;

/**
 * @title ERC20 interface
 * @dev see https://github.com/ethereum/EIPs/issues/20
 */
interface IERC20 {
  // 7c80aa9f
  function totalSupply() external view returns (uint256);

  // 70a08231
  function balanceOf(address who) external view returns (uint256);

  // dd62ed3e
  function allowance(address owner, address spender)
    external view returns (uint256);

  // a9059cbb
  function transfer(address to, uint256 value) external returns (bool);

  // 095ea7b3
  function approve(address spender, uint256 value)
    external returns (bool);

  // 0c41b033
  function transferFrom(address from, address to, uint256 value)
    external returns (bool);

  // ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
  event Transfer(
    address indexed from,
    address indexed to,
    uint256 value
  );

  // 8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
  event Approval(
    address indexed owner,
    address indexed spender,
    uint256 value
  );
}