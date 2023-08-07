// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The RelayEncoder contract's address.
address constant RELAY_ENCODER_ADDRESS = 0x0000000000000000000000000000000000000805;

/// @dev The RelayEncoder contract's instance.
RelayEncoder constant RELAY_ENCODER_CONTRACT = RelayEncoder(
    RELAY_ENCODER_ADDRESS
);

/// @author The Moonbeam Team
/// @title Pallet Relay Encoder Interface
/// @dev The interface through which solidity contracts will interact with Relay Encoder
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
/// @custom:address 0x0000000000000000000000000000000000000805
interface RelayEncoder {
    /// @dev Encode 'bond' relay call
    /// @custom:selector 72a9fbc6
    /// @param amount: The amount to bond
    /// @param rewardDestination: the account that should receive the reward
    /// @return result The bytes associated with the encoded call
    function encodeBond(
        uint256 amount,
        bytes memory rewardDestination
    ) external pure returns (bytes memory result);

    /// @dev Encode 'bondExtra' relay call
    /// @custom:selector 813667a0
    /// @param amount: The extra amount to bond
    /// @return result The bytes associated with the encoded call
    function encodeBondExtra(
        uint256 amount
    ) external pure returns (bytes memory result);

    /// @dev Encode 'unbond' relay call
    /// @custom:selector 51b14e57
    /// @param amount The amount to unbond
    /// @return result The bytes associated with the encoded call
    function encodeUnbond(
        uint256 amount
    ) external pure returns (bytes memory result);

    /// @dev Encode 'withdrawUnbonded' relay call
    /// @custom:selector d5ad108e
    /// @param slashes Weight hint, number of slashing spans
    /// @return result The bytes associated with the encoded call
    function encodeWithdrawUnbonded(
        uint32 slashes
    ) external pure returns (bytes memory result);

    /// @dev Encode 'validate' relay call
    /// @custom:selector bb64ca0c
    /// @param commission: Commission of the validator as partsPerBillion
    /// @param blocked: Whether or not the validator is accepting more nominations
    /// @return result The bytes associated with the encoded call
    function encodeValidate(
        uint256 commission,
        bool blocked
    ) external pure returns (bytes memory result);

    /// @dev Encode 'nominate' relay call
    /// @custom:selector dcf06883
    /// @param nominees: An array of AccountIds corresponding to the accounts we will nominate
    /// @return result The bytes associated with the encoded call
    function encodeNominate(
        bytes32[] memory nominees
    ) external pure returns (bytes memory result);

    /// @dev Encode 'chill' relay call
    /// @custom:selector b5eaac43
    /// @return result The bytes associated with the encoded call
    function encodeChill() external pure returns (bytes memory result);

    /// @dev Encode 'setPayee' relay call
    /// @custom:selector 414be337
    /// @param rewardDestination: the account that should receive the reward
    /// @return result The bytes associated with the encoded call
    function encodeSetPayee(
        bytes memory rewardDestination
    ) external pure returns (bytes memory result);

    /// @dev Encode 'setController' relay call
    /// @custom:selector 15490616
    /// @return result The bytes associated with the encoded call
    function encodeSetController() external pure returns (bytes memory result);

    /// @dev Encode 'rebond' relay call
    /// @custom:selector 0922ee17
    /// @param amount: The amount to rebond
    /// @return result The bytes associated with the encoded call
    function encodeRebond(
        uint256 amount
    ) external pure returns (bytes memory result);

    /// @dev Encode 'hrmp.init_open_channel' relay call
    /// @custom:selector e5e20a64
    /// @param recipient: The paraId to whom we want to initiate the open channel
    /// @param maxCapacity: The maximum capacity for the channel
    /// @param maxMessageSize: The maximum message size for the channel
    /// @return result The bytes associated with the encoded call
    function encodeHrmpInitOpenChannel(
        uint32 recipient,
        uint32 maxCapacity,
        uint32 maxMessageSize
    ) external pure returns (bytes memory result);

    /// @dev Encode 'hrmp.accept_open_channel' relay call
    /// @custom:selector 98a76477
    /// @param sender: The paraId from which we want to accept the channel
    function encodeHrmpAcceptOpenChannel(
        uint32 sender
    ) external pure returns (bytes memory result);

    /// @dev Encode 'hrmp.close_channel' relay call
    /// @custom:selector 9cfbdfc5
    /// @param sender: The paraId of the sender
    /// @param sender: The paraId of the recipient
    function encodeHrmpCloseChannel(
        uint32 sender,
        uint32 recipient
    ) external pure returns (bytes memory result);

    /// @dev Encode 'hrmp.cancel_open_request' relay call
    /// @custom:selector 8fd5ce49
    /// @param sender: The paraId of the sender
    /// @param recipient: The paraId of the recipient
    /// @param openRequests: The number of open requests
    function encodeHrmpCancelOpenRequest(
        uint32 sender,
        uint32 recipient,
        uint32 openRequests
    ) external pure returns (bytes memory result);
}
