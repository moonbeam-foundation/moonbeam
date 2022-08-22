// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/xcm-transactor/XcmTransactor.sol";

contract XcmTransactorInstance is XcmTransactor {
    /// The Xcm Transactor wrapper at the known pre-compile address.
    XcmTransactor public xcmtransactor =
        XcmTransactor(0x0000000000000000000000000000000000000806);

    function indexToAccount(uint16 index)
        external
        view
        override
        returns (address)
    {
        // We nominate our target collator with all the tokens provided
        return xcmtransactor.indexToAccount(index);
    }

    function transactInfo(Multilocation memory multilocation)
        external
        view
        override
        returns (
            uint64,
            uint256,
            uint64
        )
    {
        return xcmtransactor.transactInfo(multilocation);
    }

    function transactInfoWithSigned(Multilocation memory multilocation)
        external
        view
        override
        returns (
            uint64,
            uint64,
            uint64
        )
    {
        return xcmtransactor.transactInfoWithSigned(multilocation);
    }

    function feePerSecond(Multilocation memory multilocation)
        external
        view
        override
        returns (uint256)
    {
        return xcmtransactor.feePerSecond(multilocation);
    }

    function transactThroughDerivativeMultilocation(
        uint8 transactor,
        uint16 index,
        Multilocation memory feeAsset,
        uint64 weight,
        bytes memory innerCall
    ) external override {
        xcmtransactor.transactThroughDerivativeMultilocation(
            transactor,
            index,
            feeAsset,
            weight,
            innerCall
        );
    }

    function transactThroughDerivative(
        uint8 transactor,
        uint16 index,
        address currencyId,
        uint64 weight,
        bytes memory innerCall
    ) external override {
        xcmtransactor.transactThroughDerivative(
            transactor,
            index,
            currencyId,
            weight,
            innerCall
        );
    }

    function transactThroughSigned(
        Multilocation memory dest,
        address feeLocationAddress,
        uint64 weight,
        bytes memory call
    ) external override {
        xcmtransactor.transactThroughSigned(
            dest,
            feeLocationAddress,
            weight,
            call
        );
    }

    function transactThroughSignedMultilocation(
        Multilocation memory dest,
        Multilocation memory feeLocation,
        uint64 weight,
        bytes memory call
    ) external override {
        xcmtransactor.transactThroughSignedMultilocation(
            dest,
            feeLocation,
            weight,
            call
        );
    }
}
