// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

interface Batch {
    /// @dev Batch multiple calls into a single transaction.
    /// All calls are performed from the address calling this precompile, as
    /// if it was called as a DELEGATECALL (which is normally not possible by
    /// an EOA address). 
    ///
    /// In case of one subcall reverting, no more subcalls will be executed but
    /// the batch transaction will succeed. Use batch_all to revert on any subcall revert.
    ///
    /// @param to List of addresses to call.
    /// @param value List of values for each subcall. If array is shorter than "to" then additional
    /// calls will be performed with a value of 0.
    /// @param call_data Call data for each `to` address. If array is shorter than "to" then
    /// additional calls will be performed with an empty call data.
    /// @return Data returned by each subcall.
    function batch_some(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data
    ) external payable returns (bytes[] memory);

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
    /// @return Data returned by each subcall.
    function batch_all(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data
    ) external payable returns (bytes[] memory);
}