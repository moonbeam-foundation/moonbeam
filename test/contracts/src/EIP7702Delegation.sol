// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

contract EIP7702Delegation {
    mapping(address => uint256) public balances;
    uint256 public totalSupply;
    
    event BalanceSet(address indexed account, uint256 newBalance);
    event TotalSupplyChanged(uint256 newTotalSupply);
    
    function setBalance(address account, uint256 newBalance) external {
        balances[account] = newBalance;
        emit BalanceSet(account, newBalance);
    }
    
    function incrementBalance(address account, uint256 amount) external returns (uint256) {
        balances[account] += amount;
        totalSupply += amount;
        emit BalanceSet(account, balances[account]);
        emit TotalSupplyChanged(totalSupply);
        return balances[account];
    }
    
    function getBalance(address account) external view returns (uint256) {
        return balances[account];
    }
    
    function getTotalSupply() external view returns (uint256) {
        return totalSupply;
    }
}