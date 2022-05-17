// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

interface Batch {
    /// @dev Batch multiple calls into a single transaction.
    /// All calls are performed from the address calling this precompile, as
    /// if it was called as a DELEGATECALL (which is normally not possible by
    /// an EOA address).
    ///
    /// In case of one subcall reverting following subcalls will still be attempted.
    ///
    /// @param to List of addresses to call.
    /// @param value List of values for each subcall. If array is shorter than "to" then additional
    /// calls will be performed with a value of 0.
    /// @param call_data Call data for each `to` address. If array is shorter than "to" then
    /// additional calls will be performed with an empty call data.
    /// @param gas_limit Gas limit for each `to` address. If array is shorter than "to" then
    /// the remaining gas available will be used.
    /// @param emitLogs Should the precompile emit logs for each call?
    /// Increases slightly the cost
    /// Selector: b1d4c0a7
    function batchSome(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data,
        uint64[] memory gas_limit,
        bool emitLogs
    ) external;

    /// @dev Batch multiple calls into a single transaction.
    /// All calls are performed from the address calling this precompile, as
    /// if it was called as a DELEGATECALL (which is normally not possible by
    /// an EOA address).
    ///
    /// In case of one subcall reverting, no more subcalls will be executed but
    /// the batch transaction will succeed. Use batchAll to revert on any subcall revert.
    ///
    /// @param to List of addresses to call.
    /// @param value List of values for each subcall. If array is shorter than "to" then additional
    /// calls will be performed with a value of 0.
    /// @param call_data Call data for each `to` address. If array is shorter than "to" then
    /// additional calls will be performed with an empty call data.
    /// @param gas_limit Gas limit for each `to` address. If array is shorter than "to" then
    /// the remaining gas available will be used.
    /// @param emitLogs Should the precompile emit logs for each call? (increase cost).
    /// Selector: b4b8481a
    function batchSomeUntilFailure(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data,
        uint64[] memory gas_limit,
        bool emitLogs
    ) external;

    /// @dev Batch multiple calls into a single transaction.
    /// All calls are performed from the address calling this precompile, as
    /// if it was called as a DELEGATECALL (which is normally not possible by
    /// an EOA address).
    ///
    /// In case of one subcall reverting, the entire batch will revert.
    ///
    /// @param to List of addresses to call.
    /// @param value List of values for each subcall. If array is shorter than "to" then additional
    /// calls will be performed with a value of 0.
    /// @param call_data Call data for each `to` address. If array is shorter than "to" then
    /// additional calls will be performed with an empty call data.
    /// @param gas_limit Gas limit for each `to` address. If array is shorter than "to" then
    /// the remaining gas available will be used.
    /// Selector: 96e292b8
    function batchAll(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data,
        uint64[] memory gas_limit
    ) external;

    /// Emitted when a subcall succeeds.
    event SubcallSucceeded(uint256 index);

    /// Emitted when a subcall fails.
    event SubcallFailed(uint256 index);
}
