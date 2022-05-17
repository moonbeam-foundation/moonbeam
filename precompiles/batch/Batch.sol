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
    /// @return successes Number of subcalls executed.
    /// @return outputs Data returned by each subcall.
    /// Selector: 9205a0ba
    function batchSome(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data
    ) external payable returns (uint256 successes, bytes[] memory outputs);

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
    /// @return successes Number of subcalls executed.
    /// @return outputs Data returned by each subcall.
    /// Selector: c803ba9a
    function batchSomeUntilFailure(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data
    ) external payable returns (uint256 successes, bytes[] memory outputs);

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
    /// @return outputs Data returned by each subcall.
    /// Selector: 2d41531c
    function batchAll(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data
    ) external payable returns (bytes[] memory outputs);

    /// Emitted when a subcall succeeds.
    /// Selector: 1394239457558577f9e943b1c40196059a4bc5075a41a6e33ea3c676a297ee67
    event SubcallSucceeded(uint256 index, bytes output);

    /// Emitted when a subcall fails.
    /// Selector: e0844dd772fe51cb542f007a35cbc42ed46d2ce4a5be7ceacb2623920108fac3
    event SubcallFailed(uint256 index, bytes output);
}
