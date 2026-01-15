// SPDX-License-Identifier: GPL-3.0-only
pragma solidity ^0.8.31;

/// @title CLZ - Count Leading Zeros (EIP-7939)
/// @notice Contract that exposes the CLZ opcode introduced in EIP-7939
/// @dev The CLZ opcode counts the number of leading zero bits in a 256-bit value
contract CLZ {
    /// @notice Count the number of leading zero bits in a 256-bit value
    /// @param x The value to count leading zeros for
    /// @return result The number of leading zero bits (0-256)
    function countLeadingZeros(uint256 x) public pure returns (uint256 result) {
        assembly {
            result := clz(x)
        }
    }
}
