// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

interface ICallForwarder {
    function callRange(address first, address last) external;
}

interface ILooper {
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
        try ICallForwarder(target).callRange(first, last) {
            emit SubCallSucceed();
        } catch (bytes memory) {
            emit SubCallFail();
        }
    }

    function subCallLooper(address target, uint256 n) public {
        try ILooper(target).incrementalLoop(n) {
            emit SubCallSucceed();
        } catch (bytes memory) {
            emit SubCallFail();
        }
    }
}
