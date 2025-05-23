// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

contract MultiplyBy7Fat {

    uint256 private immutable _tag;       // stored IN CODE

    constructor(uint256 tag_) {
        _tag = tag_;
    }

    function multiply(uint256 a) public pure returns (uint256 d) {
        return a * 7;
    }
}
