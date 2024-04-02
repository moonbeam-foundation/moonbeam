/// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The Storage Cleaner Precompile contract's address.
address constant STORAGE_CLEANER_PRECOMPILE_ADDRESS = 0x0000000000000000000000000000000000000403;

/// @dev The Storage Cleaner Precompile contract's instance.
StorageCleaner constant STORAGE_CLEANER_PRECOMPILE_CONTRACT = StorageCleaner(
    STORAGE_CLEANER_PRECOMPILE_ADDRESS
);

/// @title Storage Cleaner Precompile Interface
/// @dev Interface for interacting with the Storage Cleaner precompile
/// @custom:address 0x0000000000000000000000000000000000000403
interface StorageCleaner {
    /// @dev Cleans storage entries based on the provided keys.
    /// @param contracts The addresses of the contracts to clean storage for.
    /// @param limit The maximum number of entries to clean in a single call.
    function clearSuicidedStorage(
        address[] calldata contracts,
        uint64 limit
    ) external;
}
