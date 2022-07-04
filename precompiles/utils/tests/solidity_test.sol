// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @title Solidity test file
interface SolidityTest {
    /// Function without params
    function fnNoArgs() external;

    /// @dev Function info
    ///
    /// @param arg0 Arg0 Description
    /// Selector: c4921133
    function fnOneArg(address arg0) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// Selector: 67ea837e
    function fnTwoArgs(address arg0, uint256 arg1) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// Selector: d6b423d9
    function fnSameArgs(uint64 arg0, uint64 arg1) external;

    /// @param arg0 Arg0 Description
    /// Selector: b9904a86
    function fnOneArgSameLine(uint64 arg0) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// Selector: 28f0c44e
    function fnTwoArgsSameLine(uint64 arg0, bytes32 arg1) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// Selector: 06f0c1ce
    function fnTwoArgsSameLineExternalSplit(uint64 arg0, bytes32 arg1) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @param arg2 Arg2 Description
    /// Selector: 18001a4e
    function fnMemoryArrayArgs(
        address[] memory arg0,
        uint256[] memory arg1,
        bytes[] arg2,
    ) external;
}
