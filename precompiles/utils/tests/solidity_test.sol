// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @title Solidity test file with incorrectly defined selectors
interface SolidityTest {
    /// A custom type
    struct CustomArg0 {
        uint8 p0;
        bytes[] p1;
    }

    /// A custom type
    struct CustomArg1 {
        address[] p0;
        uint256[] p1;
        bytes[] p2;
    }

    /// @dev Function without params and no selector
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
        bytes[] memory arg2
    ) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @param arg2 Arg2 Description
    /// Selector: d8af1a4e
    function fnCustomArgs(
        CustomArg0 memory arg0,
        bytes[] memory arg1,
        uint64 arg2
    ) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @param arg2 Arg2 Description
    /// @param arg3 Arg3 Description
    /// Selector: 550c1a4e
    function fnCustomArgsMultiple(
        CustomArg0 memory arg0,
        CustomArg1 memory arg1,
        bytes[] memory arg2,
        uint64 arg3
    ) external;
}
