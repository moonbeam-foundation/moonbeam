// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The AuthorMapping contract's address.
address constant AUTHOR_MAPPING_ADDRESS = 0x0000000000000000000000000000000000000807;

/// @dev The AuthorMapping contract's instance.
AuthorMapping constant AUTHOR_MAPPING_CONTRACT = AuthorMapping(
    AUTHOR_MAPPING_ADDRESS
);

/// @author The Moonbeam Team
/// @title Pallet AuthorMapping Interface
/// @dev The interface through which solidity contracts will interact with pallet-author.mapping
/// @custom:address 0x0000000000000000000000000000000000000807
interface AuthorMapping {
    /// @dev Add association
    /// @custom:selector ef8b6cd8
    ///
    /// @param nimbusId The nimbusId to be associated
    function addAssociation(bytes32 nimbusId) external;

    /// @dev Update existing association
    /// @custom:selector 25a39da5
    ///
    /// @param oldNimbusId The old nimbusId to be replaced
    /// @param newNimbusId The new nimbusId to be associated
    function updateAssociation(bytes32 oldNimbusId, bytes32 newNimbusId)
        external;

    /// @dev Clear existing association
    /// @custom:selector 448b54d6
    ///
    /// @param nimbusId The nimbusId to be cleared
    function clearAssociation(bytes32 nimbusId) external;

    /// @dev Remove keys
    /// @custom:selector a36fee17
    ///
    function removeKeys() external;

    /// @dev Set keys
    /// @custom:selector f1ec919c
    ///
    /// @param keys The new session keys
    function setKeys(bytes memory keys) external;

    /// @dev Get the nimbus ID of the given addresss
    ///
    /// @custom:selector 3cb194f2
    /// @param who The address for which we want to know the nimbus id
    /// @return id The nimbus ID, or zero if this address don't have a nimbus ID.
    function nimbusIdOf(address who) external returns (bytes32);

    /// @dev Get the address of the given nimbus ID
    ///
    /// @custom:selector bb34534c
    /// @param nimbusId The nimbus ID for which we want to know the address
    /// @return address The address, or zero if this nimbus ID is unknown.
    function addressOf(bytes32 nimbusId) external returns (address);

    /// @dev Get the keys of the given nimbus ID
    ///
    /// @custom:selector 089b7a68
    /// @param nimbusId The nimbus ID for which we want to know the keys
    /// @return keys Keys, or empty if this nimbus ID is unknown.
    function keysOf(bytes32 nimbusId) external returns (bytes memory keys);
}
