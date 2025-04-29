// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.10;

import "../../../precompiles/batch/Batch.sol";

contract CallBatchPrecompileFromConstructor {
    constructor(address to, bytes[] memory callData) {
        inner(to, callData);
    }

    function simple(address to, bytes[] memory callData) external {
        inner(to, callData);
    }

    function inner(address to, bytes[] memory callData) internal {
        address[] memory toAddress = new address[](1);
        toAddress[0] = to;
        uint256[] memory value = new uint256[](1);
        value[0] = 0;
        uint64[] memory gasLimit = new uint64[](1);
        gasLimit[0] = 0;
        BATCH_CONTRACT.batchAll(toAddress, value, callData, gasLimit);
    }
}