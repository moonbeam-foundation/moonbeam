// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @title Solidity test file with incorrectly defined selectors
interface SolidityTest {
    /// A custom enum
    enum CustomEnum0 {
        A,
        B,
        C
    }

    /// A custom type
    struct CustomArg0 {
        CustomEnum0 p0;
        bytes[] p1;
    }

    /// A custom type
    struct CustomArg1 {
        address[] p0;
        uint256[] p1;
        bytes[] p2;
    }

    /// A composed custom type
    struct CustomArg2 {
        CustomArg0 p0;
        CustomArg1[] p1;
    }

    /// @dev Function without params and no selector
    function fnNoArgs() external;

    /// @dev Function info
    ///
    /// @param arg0 Arg0 Description
    /// @custom:selector c4921133
    function fnOneArg(address arg0) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @custom:selector 67ea837e
    function fnTwoArgs(address arg0, uint256 arg1) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @custom:selector d6b423d9
    function fnSameArgs(uint64 arg0, uint64 arg1) external;

    /// @param arg0 Arg0 Description
    /// @custom:selector b9904a86
    function fnOneArgSameLine(uint64 arg0) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @custom:selector 28f0c44e
    function fnTwoArgsSameLine(uint64 arg0, bytes32 arg1) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @custom:selector 06f0c1ce
    function fnTwoArgsSameLineExternalSplit(uint64 arg0, bytes32 arg1) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @param arg2 Arg2 Description
    /// @custom:selector 18001a4e
    function fnMemoryArrayArgs(
        address[] memory arg0,
        uint256[] memory arg1,
        bytes[] memory arg2
    ) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @custom:selector 1ea61a4e
    function fnCalldataArgs(string calldata arg0, bytes[] memory arg1) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @param arg2 Arg2 Description
    /// @custom:selector d8af1a4e
    function fnCustomArgs(
        CustomArg0 memory arg0,
        bytes[] memory arg1,
        uint64 arg2
    ) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @custom:selector e8af1642
    function fnEnumArgs(CustomEnum0 arg0, uint64 arg1) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @param arg2 Arg2 Description
    /// @param arg3 Arg3 Description
    /// @custom:selector 550c1a4e
    function fnCustomArgsMultiple(
        CustomArg0 memory arg0,
        CustomArg1 memory arg1,
        bytes[] memory arg2,
        uint64 arg3
    ) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @custom:selector 77af1a40
    function fnCustomArrayArgs(CustomArg0[] memory arg0, bytes[] memory arg1)
        external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @custom:selector 80af0a40
    function fnCustomComposedArg(CustomArg2 memory arg0, uint64 arg1) external;

    /// @param arg0 Arg0 Description
    /// @param arg1 Arg1 Description
    /// @custom:selector 97baa040
    function fnCustomComposedArrayArg(CustomArg2[] memory arg0, uint64 arg1)
        external;
}
