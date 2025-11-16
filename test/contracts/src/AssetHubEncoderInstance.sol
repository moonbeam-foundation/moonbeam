// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/assethub-encoder/AssetHubEncoder.sol";

// We only use this to be able to generate the input data, since we need a compiled instance
contract AssetHubEncoderInstance is AssetHubEncoder {
    function encodeBond(
        uint256 amount,
        bytes memory reward_destination
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }

    function encodeBondExtra(
        uint256 amount
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }

    function encodeUnbond(
        uint256 amount
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }

    function encodeWithdrawUnbonded(
        uint32 slashes
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }

    function encodeValidate(
        uint256 commission,
        bool blocked
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }

    function encodeNominate(
        bytes32[] memory nominees
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }

    function encodeChill()
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeSetPayee(
        bytes memory rewardDestination
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }

    function encodeSetController()
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeRebond(
        uint256 amount
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }
}
