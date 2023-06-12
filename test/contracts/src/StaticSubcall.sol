// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/randomness/Randomness.sol";

contract StaticSubcall {
    Randomness public randomness =
        Randomness(0x0000000000000000000000000000000000000809);

    function staticFulfill(uint id) external {
        (bool success, bytes memory result) = address(this).staticcall(
            abi.encodeWithSelector(StaticSubcall.innerFulfill.selector, id)
        );

        require(success, string(result));
    }

    function innerFulfill(uint id) external {
        randomness.fulfillRequest(id);
    }
}
