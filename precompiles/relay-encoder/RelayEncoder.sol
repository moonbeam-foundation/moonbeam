// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @author The Moonbeam Team
/// @title The interface through which solidity contracts will interact with Relay Encoder
/// We follow this same interface including four-byte function selectors, in the precompile that
/// wraps the pallet
/// Address :    0x0000000000000000000000000000000000000805
interface RelayEncoder {
    /// dev Encode 'bond' relay call
    /// Selector: a82948d4
    /// @param controllerAddress: Address of the controller
    /// @param amount: The amount to bond
    /// @param rewardDestination: the account that should receive the reward
    /// @returns The bytes associated with the encoded call
    function encodeBond(
        uint256 controllerAddress,
        uint256 amount,
        bytes memory rewardDestination
    ) external pure returns (bytes memory result);

    /// dev Encode 'bondExtra' relay call
    /// Selector: 813667a0
    /// @param amount: The extra amount to bond
    /// @returns The bytes associated with the encoded call
    function encodeBondExtra(uint256 amount)
        external
        pure
        returns (bytes memory result);

    /// dev Encode 'unbond' relay call
    /// Selector: 51b14e57
    /// @param amount: The amount to unbond
    /// @returns The bytes associated with the encoded call
    function encodeUnbond(uint256 amount)
        external
        pure
        returns (bytes memory result);

    /// dev Encode 'withdrawUnbonded' relay call
    /// Selector: d5ad108e
    /// @param slashes: Weight hint, number of slashing spans
    /// @returns The bytes associated with the encoded call
    function encodeWithdrawUnbonded(uint32 slashes)
        external
        pure
        returns (bytes memory result);

    /// dev Encode 'validate' relay call
    /// Selector: bb64ca0c
    /// @param comission: Comission of the validator as partsPerBillion
    /// @param blocked: Whether or not the validator is accepting more nominations
    /// @returns The bytes associated with the encoded call
    /// selector: 3a0d803a
    function encodeValidate(uint256 comission, bool blocked)
        external
        pure
        returns (bytes memory result);

    /// dev Encode 'nominate' relay call
    /// Selector: d2ea7b08
    /// @param nominees: An array of AccountIds corresponding to the accounts we will nominate
    /// @param blocked: Whether or not the validator is accepting more nominations
    /// @returns The bytes associated with the encoded call
    function encodeNominate(uint256[] memory nominees)
        external
        pure
        returns (bytes memory result);

    /// dev Encode 'chill' relay call
    /// Selector: b5eaac43
    /// @returns The bytes associated with the encoded call
    function encodeChill() external pure returns (bytes memory result);

    /// dev Encode 'setPayee' relay call
    /// Selector: 414be337
    /// @param rewardDestination: the account that should receive the reward
    /// @returns The bytes associated with the encoded call
    function encodeSetPayee(bytes memory rewardDestination)
        external
        pure
        returns (bytes memory result);

    /// dev Encode 'setController' relay call
    /// Selector: 07f7c6dc
    /// @param controller: The controller address
    /// @returns The bytes associated with the encoded call
    function encodeSetController(uint256 controller)
        external
        pure
        returns (bytes memory result);

    /// dev Encode 'rebond' relay call
    /// Selector: 0922ee17
    /// @param amount: The amount to rebond
    /// @returns The bytes associated with the encoded call
    function encodeRebond(uint256 amount)
        external
        pure
        returns (bytes memory result);
}
