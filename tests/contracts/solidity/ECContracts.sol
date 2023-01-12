// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.3;

contract RecoveryChecker {
    constructor() {}

    function checkRecovery(
        bytes32 msgHash,
        uint256 v,
        bytes32 r,
        bytes32 s
    ) public view returns (address) {
        assembly {
            let pointer := mload(0x40)

            mstore(pointer, msgHash)
            mstore(add(pointer, 0x20), v)
            mstore(add(pointer, 0x40), r)
            mstore(add(pointer, 0x60), s)

            if iszero(staticcall(not(0), 0x01, pointer, 0x80, pointer, 0x20)) {
                revert(0, 0)
            }

            let size := returndatasize()
            returndatacopy(pointer, 0, size)
            return(pointer, size)
        }
    }
}

contract PairingChecker {

    bool public status = false;

    function callBn256Pairing(bytes memory input) public returns (bytes32 result) {
        uint256 len = input.length;
        assembly {
            let memPtr := mload(0x40)
            let success := call(gas(), 0x08, 0, add(input, 0x20), len, memPtr, 0x20)
            switch success
            case 0 {
                revert(0,0)
            } default {
                result := mload(memPtr)
            }
        }
        status = result == 0x0000000000000000000000000000000000000000000000000000000000000001;
    }
}
