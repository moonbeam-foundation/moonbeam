// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/relay-encoder/RelayEncoder.sol";

// We only use this to be able to generate the input data, since we need a compiled instance
contract RelayEncoderInstance is RelayEncoder {
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

    function encodeHrmpInitOpenChannel(uint32 recipient, uint32 maxCapacity, uint32 maxMessageSize)
        external
        pure
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeHrmpAcceptOpenChannel(uint32 sender)
        external
        pure
        returns (bytes memory result)
    {
        return "0x00";
    }

     function encodeHrmpCloseChannel(uint32 sender, uint32 recipient)
        external
        pure
        returns (bytes memory result)
    {
        return "0x00";
    }

    function encodeHrmpCancelOpenRequest(uint32 sender, uint32 recipient, uint32 openRequests) 
        external 
        pure 
        returns (bytes memory result) 
    {
        return "0x00";
    }

}
