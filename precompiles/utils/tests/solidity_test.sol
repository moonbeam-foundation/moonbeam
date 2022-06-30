// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

interface SolidityTest {
    /// Function without params
    function fnNoArgs() external;

    /// @dev Interface for all randomness consumers
    ///
    /// @param refund_address Address to refund with fee less cost of subcall
    /// Selector: c4921133
    function fnOneArg(address refund_address) external;

    /// @param refund_address Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// Selector: 67ea837e
    function fnTwoArgs(address refund_address, uint256 fee) external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param epoch_index Relay chain epoch for which randomness is requested
    /// Selector: d6b423d9
    function fnSameArgs(uint64 gas_limit, uint64 epoch_index) external;

    /// @param request_id Request to be fulfilled by caller
    /// Selector: b9904a86
    function fnOneArgSameLine(uint64 request_id) external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 28f0c44e
    function fnTwoArgsSameLine(uint64 gas_limit, bytes32 salt) external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 28f0c44e
    function fnTwoArgsSameLineExternalSplit(uint64 gas_limit, bytes32 salt)
        external;
}
