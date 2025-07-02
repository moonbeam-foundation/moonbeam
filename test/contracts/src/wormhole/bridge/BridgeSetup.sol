// contracts/BridgeSetup.sol
// SPDX-License-Identifier: Apache 2

pragma solidity ^0.8.0;
pragma experimental ABIEncoderV2;

import "./BridgeGovernance.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";

contract BridgeSetup is BridgeSetters {
    function setup(
        address implementation,
        uint16 chainId,
        address wormhole,
        uint16 governanceChainId,
        bytes32 governanceContract,
        address tokenImplementation,
        address WETH,
        uint8 finality,
        uint256 evmChainId
    ) public {
        setChainId(chainId);

        setWormhole(wormhole);

        setGovernanceChainId(governanceChainId);
        setGovernanceContract(governanceContract);

        setTokenImplementation(tokenImplementation);

        setWETH(WETH);

        setFinality(finality);

        setEvmChainId(evmChainId);

        ERC1967Utils.upgradeToAndCall(implementation, "");

        // See https://github.com/wormhole-foundation/wormhole/issues/1930 for
        // why we set this here
        setInitialized(implementation);
    }
}
