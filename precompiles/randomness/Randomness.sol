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
    /// Selector: c92142bc
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
    /// Selector: 73257347
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
    /// Selector: 8ef48c72
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
    /// Selector: 3dbc0d19
    function requestLocalRandomness(
        address refund_address,
        uint256 fee,
        uint64 gas_limit,
        bytes32 salt,
        uint64 block_number
    ) external;

    /// @param request_id Request to be fulfilled by caller
    /// Selector: b5983332
    function fulfillRequest(uint64 request_id) external;

    /// @param request_id Request to be increased fee by caller
    /// Selector: f35d8354
    function increaseRequestFee(uint64 request_id) external;

    /// @param request_id Request to be purged by caller
    /// Selector: 536b9ef1
    function executeRequestExpiration(uint64 request_id) external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: b0ea3938
    function instantBabeRandomnessCurrentBlock(uint64 gas_limit, bytes32 salt)
        external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 0cd3aa4a
    function instantBabeRandomnessOneEpochAgo(uint64 gas_limit, bytes32 salt)
        external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: bc88ee5f
    function instantBabeRandomnessTwoEpochsAgo(uint64 gas_limit, bytes32 salt)
        external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: f71c715f
    function instantLocalRandomness(uint64 gas_limit, bytes32 salt) external;
}
