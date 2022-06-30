// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

interface Randomness {
    /// @dev Interface for all randomness consumers
    ///
    /// @param refund_address Address to refund with fee less cost of subcall
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// @param block_number Relay chain block for which randomness is requested
    /// Selector: 79df4b9c TODO
    function requestBabeRandomnessCurrentBlock(
        address refund_address,
        uint256 fee,
        uint64 gas_limit,
        bytes32 salt,
        uint64 block_number
    ) external;

    /// @param refund_address Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// @param epoch_index Relay chain epoch index for which randomness is requested
    /// Selector: 79df4b9c TODO
    function requestBabeRandomnessOneEpochAgo(
        address refund_address,
        uint256 fee,
        uint64 gas_limit,
        bytes32 salt,
        uint64 epoch_index
    ) external;

    /// @param refund_address Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// @param epoch_index Relay chain epoch for which randomness is requested
    /// Selector: 79df4b9c TODO
    function requestBabeRandomnessTwoEpochsAgo(
        address refund_address,
        uint256 fee,
        uint64 gas_limit,
        bytes32 salt,
        uint64 epoch_index
    ) external;

    /// @param refund_address Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// @param block_number Parachain block for which randomness is requested
    /// Selector: 79df4b9c TODO
    function requestLocalRandomness(
        address refund_address,
        uint256 fee,
        uint64 gas_limit,
        bytes32 salt,
        uint64 block_number
    ) external;

    /// @param request_id Request to be fulfilled by caller
    /// Selector: 79df4b9c TODO
    function fulfillRequest(uint64 request_id) external;

    /// @param request_id Request to be increased fee by caller
    /// Selector: 79df4b9c TODO
    function increaseRequestFee(uint64 request_id) external;

    /// @param request_id Request to be purged by caller
    /// Selector: 79df4b9c TODO
    function executeRequestExpiration(uint64 request_id) external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 79df4b9c TODO
    function instantBabeRandomnessCurrentBlock(uint64 gas_limit, bytes32 salt)
        external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 79df4b9c TODO
    function instantBabeRandomnessOneEpochAgo(uint64 gas_limit, bytes32 salt)
        external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 79df4b9c TODO
    function instantBabeRandomnessTwoEpochsAgo(uint64 gas_limit, bytes32 salt)
        external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 79df4b9c TODO
    function instantLocalRandomness(uint64 gas_limit, bytes32 salt) external;
}
