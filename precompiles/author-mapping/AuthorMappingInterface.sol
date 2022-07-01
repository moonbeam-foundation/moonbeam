// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @title Pallet AuthorMapping Interface
///
/// The interface through which solidity contracts will interact with pallet-author.mapping
/// Address :    0x0000000000000000000000000000000000000807
interface AuthorMapping {
    /// @dev Add association
    /// Selector: ef8b6cd8
    ///
    /// @param nimbusId The nimbusId to be associated
    function addAssociation(bytes32 nimbusId) external;

    /// @dev Update existing association
    /// Selector: 25a39da5
    ///
    /// @param oldNimbusId The old nimbusId to be replaced
    /// @param newNimbusId The new nimbusId to be associated
    function updateAssociation(bytes32 oldNimbusId, bytes32 newNimbusId)
        external;

    /// @dev Clear existing association
    /// Selector: 448b54d6
    ///
    /// @param nimbusId The nimbusId to be cleared
    function clearAssociation(bytes32 nimbusId) external;

    /// @dev Remove keys
    /// Selector: a36fee17
    ///
    function removeKeys() external;

    /// @dev Set keys
    /// Selector: f1ec919c
    ///
    /// @param keys The new session keys
    function setKeys(bytes memory keys) external;
}
