// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

contract Batch {
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
    /// @return successes Number of subcalls executed.
    /// @return outputs Data returned by each subcall.
    function batch_some(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data
    ) external payable returns (uint256 successes, bytes[] memory outputs) {
        outputs = new bytes[](to.length);

        for (uint i = 0; i < to.length; i++) {
            uint256 i_value = (i < value.length)
                ? value[i]
                : 0;
            
            bytes memory i_call_data = (i < call_data.length)
                ? call_data[i]
                : new bytes(0);

            // TODO: Execute with slightly less gas to allow this code to handle OOG reverts without
            // being OOG itself ?
            (bool i_success, bytes memory i_outputs) = to[i].call{value: i_value}(i_call_data);
            outputs[i] = i_outputs;

            if (!i_success) {
                return (i, outputs);
            }
        }

        return (to.length, outputs);
    }

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
    function batch_all(
        address[] memory to,
        uint256[] memory value,
        bytes[] memory call_data
    ) external payable returns (bytes[] memory outputs) {
        outputs = new bytes[](to.length);

        for (uint i = 0; i < to.length; i++) {
            uint256 i_value = (i < value.length)
                ? value[i]
                : 0;
            
            bytes memory i_call_data = (i < call_data.length)
                ? call_data[i]
                : new bytes(0);

            (bool i_success, bytes memory i_outputs) = to[i].call{value: i_value}(i_call_data);
            require(i_success);
            outputs[i] = i_outputs;
        }
    }
}