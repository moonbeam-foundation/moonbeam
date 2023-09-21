// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.2 <0.9.0;

contract Storage {
    mapping(uint256 => uint256) public map;

    function store(uint16 m, uint16 n) public {
        for (uint16 i = m; i < n; i++) {
            map[i] = i;
        }
    }

    function destroy() public {
        selfdestruct(payable(msg.sender));
    }
}
