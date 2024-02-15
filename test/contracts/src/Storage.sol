pragma solidity ^0.8.0;

contract Storage {
  mapping(uint256 => uint256) public map;
  
    function store(uint256 lower, uint256 upper) public {
        require(upper > lower, "Upper bound must be greater than or equal to lower bound");
        for(uint i = lower; i < upper; i++) {
            map[i] = i;
        }
    }

    function retrieve(uint256 index) public view returns (uint256) {
        return map[index];
    }

    function destroy() public {
        selfdestruct(payable(msg.sender));
    }
}