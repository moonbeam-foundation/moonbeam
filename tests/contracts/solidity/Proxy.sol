// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

contract Proxy {
    function call(address target, bytes memory data)
        public
        returns (bool, bytes memory)
    {
        return target.call(data);
    }

    function delegateCall(address target, bytes memory data)
        public
        returns (bool, bytes memory)
    {
        return target.delegatecall(data);
    }
}
