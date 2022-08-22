// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/relay-encoder/RelayEncoder.sol";

// We only use this to be able to generate the input data, since we need a compiled instance
contract RelayEncoderInstance is RelayEncoder {
    /// The Relay Encoder wrapper at the known pre-compile address.
    RelayEncoder public relayencoder =
        RelayEncoder(0x0000000000000000000000000000000000000805);

    function encodeBond(
        uint256 controllerAddress,
        uint256 amount,
        bytes memory reward_destination
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }

    function encodeBondExtra(uint256 amount)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeUnbond(uint256 amount)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeWithdrawUnbonded(uint32 slashes)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeValidate(uint256 comission, bool blocked)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeNominate(uint256[] memory nominees)
        external
        pure
        override
        returns (bytes memory result)
    {
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

    function encodeSetPayee(bytes memory rewardDestination)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeSetController(uint256 controller)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeRebond(uint256 amount)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }
}
