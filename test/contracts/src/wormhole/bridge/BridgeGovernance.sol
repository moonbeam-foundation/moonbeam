// contracts/Bridge.sol
// skip-compilation
// SPDX-License-Identifier: Apache 2

pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Utils.sol";

import "../libraries/external/BytesLib.sol";

import "./BridgeGetters.sol";
import "./BridgeSetters.sol";
import "./BridgeStructs.sol";

import "./token/Token.sol";
import "./token/TokenImplementation.sol";

import "../interfaces/IWormhole.sol";

// Crystalin: Custom version (less functions) to reduce Bytecode size while enabling debug
contract BridgeGovernance is BridgeGetters, BridgeSetters {
    using BytesLib for bytes;

    // "TokenBridge" (left padded)
    bytes32 constant module =
        0x000000000000000000000000000000000000000000546f6b656e427269646765;

    // Execute a RegisterChain governance message
    function registerChain(bytes memory encodedVM) public {
        (
            IWormhole.VM memory vm,
            bool valid,
            string memory reason
        ) = verifyGovernanceVM(encodedVM);
        require(valid, reason);

        setGovernanceActionConsumed(vm.hash);

        BridgeStructs.RegisterChain memory chain = parseRegisterChain(
            vm.payload
        );

        require(
            (chain.chainId == chainId() && !isFork()) || chain.chainId == 0,
            "invalid chain id"
        );
        require(
            bridgeContracts(chain.emitterChainID) == bytes32(0),
            "chain already registered"
        );

        setBridgeImplementation(chain.emitterChainID, chain.emitterAddress);
    }

    function verifyGovernanceVM(
        bytes memory encodedVM
    )
        internal
        view
        returns (
            IWormhole.VM memory parsedVM,
            bool isValid,
            string memory invalidReason
        )
    {
        (IWormhole.VM memory vm, bool valid, string memory reason) = wormhole()
            .parseAndVerifyVM(encodedVM);

        if (!valid) {
            return (vm, valid, reason);
        }

        if (vm.emitterChainId != governanceChainId()) {
            return (vm, false, "wrong governance chain");
        }
        if (vm.emitterAddress != governanceContract()) {
            return (vm, false, "wrong governance contract");
        }

        if (governanceActionIsConsumed(vm.hash)) {
            return (vm, false, "governance action already consumed");
        }

        return (vm, true, "");
    }

    event ContractUpgraded(
        address indexed oldContract,
        address indexed newContract
    );

    function parseRegisterChain(
        bytes memory encoded
    ) public pure returns (BridgeStructs.RegisterChain memory chain) {
        uint index = 0;

        // governance header

        chain.module = encoded.toBytes32(index);
        index += 32;
        require(chain.module == module, "wrong module");

        chain.action = encoded.toUint8(index);
        index += 1;
        require(chain.action == 1, "wrong action");

        chain.chainId = encoded.toUint16(index);
        index += 2;

        // payload

        chain.emitterChainID = encoded.toUint16(index);
        index += 2;

        chain.emitterAddress = encoded.toBytes32(index);
        index += 32;

        require(encoded.length == index, "wrong length");
    }
}
