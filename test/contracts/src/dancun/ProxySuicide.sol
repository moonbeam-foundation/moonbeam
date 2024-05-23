// SPDX-License-Identifier: GPL-3.0-only
pragma solidity ^0.8.24;

contract ProxyDeployer {
    event ContractDestroyed(address destroyedAddress);

    // Function to deploy a new Suicide contract
    function deployAndDestroy(address target, uint256 entries) public  {
        Suicide newContract = new Suicide(entries);
        newContract.destroy(target);
        emit ContractDestroyed(address(newContract));
    }

}

contract Suicide {
    mapping(uint256 => uint256) public map;

    constructor(uint256 entries) payable {
        for(uint i = 0; i < entries; i++) {
            map[i] = i;
        }
    }

    function destroy(address target) public {
        selfdestruct(payable(target));
    }
}