// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

import "../../../precompiles/relay-encoder/RelayEncoder.sol";

// We only use this to be able to generate the input data, since we need a compiled instance
contract RelayEncoderInstance is RelayEncoder {
    /// The Relay Encoder wrapper at the known pre-compile address.
    RelayEncoder public relayencoder =
        RelayEncoder(0x0000000000000000000000000000000000000805);

    function encode_bond(
        uint256 controller_address,
        uint256 amount,
        bytes memory reward_destination
    ) external pure override returns (bytes memory result) {
        return "0x00";
    }

    function encode_bond_extra(uint256 amount)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encode_unbond(uint256 amount)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encode_withdraw_unbonded(uint32 slashes)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encode_validate(uint256 comission, bool blocked)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encode_nominate(uint256[] memory nominees)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encode_chill()
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encode_set_payee(bytes memory reward_destination)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encode_set_controller(uint256 controller)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encode_rebond(uint256 amount)
        external
        pure
        override
        returns (bytes memory result)
    {
        return "0x00";
    }
}
