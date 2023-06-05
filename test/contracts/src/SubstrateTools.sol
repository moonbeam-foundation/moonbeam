// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @notice Set of function needed for manipulating substrate transaction/data
library SubstrateTools {
    /// @notice concatenated bytes of the string, prefixed by the length in big endian
    /// @param text text to convert
    function buildSubstrateString(bytes memory text)
        internal
        pure
        returns (bytes memory)
    {
        // Add 1 for encodings
        uint16 length = uint16(text.length * 4) + 1;
        // conversion to big endian
        uint16 reversedlength = ((length >> 8) | (length << 8));
        // string prefixed by big endian length
        return bytes.concat(bytes2(reversedlength), text);
    }

    /// @notice build the storage key/item following substrate conventions
    /// @param key Storage key
    /// @param value Storage value
    function buildSetStorageItem(bytes memory key, bytes memory value)
        internal
        pure
        returns (bytes memory)
    {
        return
            bytes.concat(
                buildSubstrateString(key),
                buildSubstrateString(value)
            );
    }

    /// @notice build the set storage proposal. It includes the setStorage call + the number of
    /// @notice storages to change and the key/value of each storage.
    /// @param storageKey Storage key to change
    /// @param storageValue Storage value
    function buildSetStorageProposal(
        bytes memory storageKey,
        bytes memory storageValue
    ) internal pure returns (bytes memory) {
        return
            bytes.concat(
                bytes2("\x00\x04"), // Should not be hardcoded
                bytes1(uint8(1 * 4)), // 1 storage item to change, so 4 bytes
                buildSetStorageItem(storageKey, storageValue)
            );
    }

    /// @notice build the system remark proposal
    /// @param message Message to remark
    function buildSystemRemarkProposal(bytes memory message)
        internal
        pure
        returns (bytes memory)
    {
        return
            bytes.concat(
                bytes2("\x00\x01"), // Should not be hardcoded
                buildSubstrateString(bytes(message))
            );
    }
}
