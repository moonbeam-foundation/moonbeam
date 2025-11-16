// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The AssetHubEncoder contract's address.
address constant ASSETHUB_ENCODER_ADDRESS = 0x000000000000000000000000000000000000081B;

/// @dev The AssetHubEncoder contract's instance.
AssetHubEncoder constant ASSETHUB_ENCODER_CONTRACT = AssetHubEncoder(
    ASSETHUB_ENCODER_ADDRESS
);

/// @author The Moonbeam Team
/// @title Pallet AssetHub Encoder Interface
/// @dev The interface through which solidity contracts will interact with AssetHub Encoder
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
/// @custom:address 0x000000000000000000000000000000000000081B
interface AssetHubEncoder {
    /// @dev Encode 'bond' AssetHub call
    /// @custom:selector 72a9fbc6
    /// @param amount: The amount to bond
    /// @param rewardDestination: the account that should receive the reward
    /// @return result The bytes associated with the encoded call
    function encodeBond(
        uint256 amount,
        bytes memory rewardDestination
    ) external pure returns (bytes memory result);

    /// @dev Encode 'bondExtra' AssetHub call
    /// @custom:selector 813667a0
    /// @param amount: The extra amount to bond
    /// @return result The bytes associated with the encoded call
    function encodeBondExtra(
        uint256 amount
    ) external pure returns (bytes memory result);

    /// @dev Encode 'unbond' AssetHub call
    /// @custom:selector 51b14e57
    /// @param amount The amount to unbond
    /// @return result The bytes associated with the encoded call
    function encodeUnbond(
        uint256 amount
    ) external pure returns (bytes memory result);

    /// @dev Encode 'withdrawUnbonded' AssetHub call
    /// @custom:selector d5ad108e
    /// @param slashes Weight hint, number of slashing spans
    /// @return result The bytes associated with the encoded call
    function encodeWithdrawUnbonded(
        uint32 slashes
    ) external pure returns (bytes memory result);

    /// @dev Encode 'validate' AssetHub call
    /// @custom:selector bb64ca0c
    /// @param commission: Commission of the validator as partsPerBillion
    /// @param blocked: Whether or not the validator is accepting more nominations
    /// @return result The bytes associated with the encoded call
    function encodeValidate(
        uint256 commission,
        bool blocked
    ) external pure returns (bytes memory result);

    /// @dev Encode 'nominate' AssetHub call
    /// @custom:selector dcf06883
    /// @param nominees: An array of AccountIds corresponding to the accounts we will nominate
    /// @return result The bytes associated with the encoded call
    function encodeNominate(
        bytes32[] memory nominees
    ) external pure returns (bytes memory result);

    /// @dev Encode 'chill' AssetHub call
    /// @custom:selector b5eaac43
    /// @return result The bytes associated with the encoded call
    function encodeChill() external pure returns (bytes memory result);

    /// @dev Encode 'setPayee' AssetHub call
    /// @custom:selector 414be337
    /// @param rewardDestination: the account that should receive the reward
    /// @return result The bytes associated with the encoded call
    function encodeSetPayee(
        bytes memory rewardDestination
    ) external pure returns (bytes memory result);

    /// @dev Encode 'setController' AssetHub call
    /// @custom:selector 15490616
    /// @return result The bytes associated with the encoded call
    function encodeSetController() external pure returns (bytes memory result);

    /// @dev Encode 'rebond' AssetHub call
    /// @custom:selector 0922ee17
    /// @param amount: The amount to rebond
    /// @return result The bytes associated with the encoded call
    function encodeRebond(
        uint256 amount
    ) external pure returns (bytes memory result);
}
