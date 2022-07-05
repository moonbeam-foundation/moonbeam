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

    /// @param refundAddress Address to refund with fee less cost of subcall
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gasLimit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// @param numBlocks Number of relay chain blocks in the future the request is for
    /// Selector: c4921133
    function requestBabeRandomnessCurrentBlock(
        address refundAddress,
        uint256 fee,
        uint64 gasLimit,
        bytes32 salt,
        uint64 numBlocks
    ) external;

    /// @param refundAddress Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gasLimit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: bbc9e95f
    function requestBabeRandomnessOneEpochAgo(
        address refundAddress,
        uint256 fee,
        uint64 gasLimit,
        bytes32 salt
    ) external;

    /// @param refundAddress Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gasLimit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// Selector: 25b14a0b
    function requestBabeRandomnessTwoEpochsAgo(
        address refundAddress,
        uint256 fee,
        uint64 gasLimit,
        bytes32 salt
    ) external;

    /// @param refundAddress Address to refund
    /// @param fee Amount to set aside to pay for the subcall
    /// @param gasLimit Gas limit for the subcall that provides randomness
    /// @param salt Salt to be mixed with raw randomness to get output
    /// @param numBlocks Number of blocks in the future the request is for
    /// Selector: b4a11763
    function requestLocalRandomness(
        address refundAddress,
        uint256 fee,
        uint64 gasLimit,
        bytes32 salt,
        uint64 numBlocks
    ) external;

    /// @param requestId Request to be fulfilled by caller
    /// Selector: b9904a86
    function fulfillRequest(uint64 requestId) external;

    /// @param requestId Request to be increased fee by caller
    /// @param feeIncrease Amount to increase fee
    /// Selector: 6a5b3380
    function increaseRequestFee(uint64 requestId, uint256 feeIncrease) external;

    /// @param requestId Request to be purged by caller
    /// Selector: 8fcdcc49
    function executeRequestExpiration(uint64 requestId) external;
}
