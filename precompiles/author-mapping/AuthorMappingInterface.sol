// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

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
}
