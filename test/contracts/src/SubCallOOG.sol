// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface CallForwarder {
    function callRange(address first, address last) external;
}

interface Looper {
    function incrementalLoop(uint256 n) external;
}

contract SubCallOOG {
    event SubCallSucceed();
    event SubCallFail();

    function subCallForwarder(
        address target,
        address first,
        address last
    ) public {
        try CallForwarder(target).callRange(first, last) {
            emit SubCallSucceed();
        } catch (bytes memory) {
            emit SubCallFail();
        }
    }

    function subCallLooper(address target, uint256 n) public {
        try Looper(target).incrementalLoop(n) {
            emit SubCallSucceed();
        } catch (bytes memory) {
            emit SubCallFail();
        }
    }
}
