// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @author The Moonbeam Team
 * @title The interface through which solidity contracts will interact with Relay Encoder
 * We follow this same interface including four-byte function selectors, in the precompile that
 * wraps the pallet
 * Address :    0x0000000000000000000000000000000000000805
 */

interface RelayEncoder {
    // dev Encode 'bond' relay call
    // Selector: 31627376
    // @param controller_address: Address of the controller
    // @param amount: The amount to bond
    // @param reward_destination: the account that should receive the reward
    // @returns The bytes associated with the encoded call
    function encode_bond(
        uint256 controller_address,
        uint256 amount,
        bytes memory reward_destination
    ) external pure returns (bytes memory result);

    // dev Encode 'bond_extra' relay call
    // Selector: 49def326
    // @param amount: The extra amount to bond
    // @returns The bytes associated with the encoded call
    function encode_bond_extra(uint256 amount)
        external
        pure
        returns (bytes memory result);

    // dev Encode 'unbond' relay call
    // Selector: 2cd61217
    // @param amount: The amount to unbond
    // @returns The bytes associated with the encoded call
    function encode_unbond(uint256 amount)
        external
        pure
        returns (bytes memory result);

    // dev Encode 'withdraw_unbonded' relay call
    // Selector: 2d220331
    // @param slashes: Weight hint, number of slashing spans
    // @returns The bytes associated with the encoded call
    function encode_withdraw_unbonded(uint32 slashes)
        external
        pure
        returns (bytes memory result);

    // dev Encode 'validate' relay call
    // Selector: 3a0d803a
    // @param comission: Comission of the validator as parts_per_billion
    // @param blocked: Whether or not the validator is accepting more nominations
    // @returns The bytes associated with the encoded call
    // selector: 3a0d803a
    function encode_validate(uint256 comission, bool blocked)
        external
        pure
        returns (bytes memory result);

    // dev Encode 'nominate' relay call
    // Selector: a7cb124b
    // @param nominees: An array of AccountIds corresponding to the accounts we will nominate
    // @param blocked: Whether or not the validator is accepting more nominations
    // @returns The bytes associated with the encoded call
    function encode_nominate(uint256[] memory nominees)
        external
        pure
        returns (bytes memory result);

    // dev Encode 'chill' relay call
    // Selector: bc4b2187
    // @returns The bytes associated with the encoded call
    function encode_chill() external pure returns (bytes memory result);

    // dev Encode 'set_payee' relay call
    // Selector: 9801b147
    // @param reward_destination: the account that should receive the reward
    // @returns The bytes associated with the encoded call
    function encode_set_payee(bytes memory reward_destination)
        external
        pure
        returns (bytes memory result);

    // dev Encode 'set_controller' relay call
    // Selector: 7a8f48c2
    // @param controller: The controller address
    // @returns The bytes associated with the encoded call
    function encode_set_controller(uint256 controller)
        external
        pure
        returns (bytes memory result);

    // dev Encode 'rebond' relay call
    // Selector: add6b3bf
    // @param amount: The amount to rebond
    // @returns The bytes associated with the encoded call
    function encode_rebond(uint256 amount)
        external
        pure
        returns (bytes memory result);
}
