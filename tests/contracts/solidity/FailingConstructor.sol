// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

contract FailingConstructor {
    constructor() {
        require(false);
    }
}
