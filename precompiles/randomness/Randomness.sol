// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

interface Randomness {
    /// @dev Interface for all randomness consumers
    ///
    /// @dev Get relay block number
    /// @return The relay block number
    /// Selector: edfec347
    function relayBlockNumber() external view returns (uint64);

    /// @dev Get relay epoch index
    /// @return The relay epoch index
    /// Selector: 81797566
    function relayEpochIndex() external view returns (uint64);

    /// @param refund_address Address to refund with fee less cost of subcall
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// @param num_blocks Number of relay chain blocks in the future the request is for
    /// Selector: c4921133
    function requestBabeRandomnessCurrentBlock(
        address refund_address,
        uint256 fee,
        uint64 gas_limit,
        bytes32 salt,
        uint64 num_blocks
    ) external;

    /// @param refund_address Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: bbc9e95f
    function requestBabeRandomnessOneEpochAgo(
        address refund_address,
        uint256 fee,
        uint64 gas_limit,
        bytes32 salt
    ) external;

    /// @param refund_address Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 25b14a0b
    function requestBabeRandomnessTwoEpochsAgo(
        address refund_address,
        uint256 fee,
        uint64 gas_limit,
        bytes32 salt
    ) external;

    /// @param refund_address Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// @param num_blocks Number of blocks in the future the request is for
    /// Selector: b4a11763
    function requestLocalRandomness(
        address refund_address,
        uint256 fee,
        uint64 gas_limit,
        bytes32 salt,
        uint64 num_blocks
    ) external;

    /// @param request_id Request to be fulfilled by caller
    /// Selector: b9904a86
    function fulfillRequest(uint64 request_id) external;

    /// @param request_id Request to be increased fee by caller
    /// Selector: 40ebb605
    function increaseRequestFee(uint64 request_id) external;

    /// @param request_id Request to be purged by caller
    /// Selector: 8fcdcc49
    function executeRequestExpiration(uint64 request_id) external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 28f0c44e
    function instantBabeRandomnessCurrentBlock(uint64 gas_limit, bytes32 salt)
        external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: cde3e7d1
    function instantBabeRandomnessOneEpochAgo(uint64 gas_limit, bytes32 salt)
        external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 1ac580b9
    function instantBabeRandomnessTwoEpochsAgo(uint64 gas_limit, bytes32 salt)
        external;

    /// @param gas_limit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: cb1abe60
    function instantLocalRandomness(uint64 gas_limit, bytes32 salt) external;
}
