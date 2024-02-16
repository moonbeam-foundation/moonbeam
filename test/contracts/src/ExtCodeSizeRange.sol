
// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

contract ExtCodeSizeRange {
    event readContractSizeEvent(
        address contractAddress,
        uint size
    );

    function range(address first, address last) public {
        require(first < last, "invalid range");
        while (first < last) {
            uint size;
            assembly {
                size := extcodesize(first)
            }
            emit readContractSizeEvent(first, size);
            
            first = address(uint160(first) + 1);
        }
    }
}
