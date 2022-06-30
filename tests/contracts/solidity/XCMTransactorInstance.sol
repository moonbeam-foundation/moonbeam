// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

import "../../../precompiles/xcm-transactor/XcmTransactor.sol";

contract XcmTransactorInstance is XcmTransactor {
    /// The Xcm Transactor wrapper at the known pre-compile address.
    XcmTransactor public xcmtransactor =
        XcmTransactor(0x0000000000000000000000000000000000000806);

    function index_to_account(uint16 index)
        external
        view
        override
        returns (address)
    {
        // We nominate our target collator with all the tokens provided
        return xcmtransactor.index_to_account(index);
    }

    function transact_info(Multilocation memory multilocation)
        external
        view
        override
        returns (
            uint64,
            uint256,
            uint64
        )
    {
        return xcmtransactor.transact_info(multilocation);
    }

    function transact_info_with_signed(Multilocation memory multilocation)
        external
        view
        override
        returns (
            uint64,
            uint64,
            uint64
        )
    {
        return xcmtransactor.transact_info_with_signed(multilocation);
    }

    function fee_per_second(Multilocation memory multilocation)
        external
        view
        override
        returns (uint256)
    {
        return xcmtransactor.fee_per_second(multilocation);
    }

    function transact_through_derivative_multilocation(
        uint8 transactor,
        uint16 index,
        Multilocation memory fee_asset,
        uint64 weight,
        bytes memory inner_call
    ) external override {
        xcmtransactor.transact_through_derivative_multilocation(
            transactor,
            index,
            fee_asset,
            weight,
            inner_call
        );
    }

    function transact_through_derivative(
        uint8 transactor,
        uint16 index,
        address currency_id,
        uint64 weight,
        bytes memory inner_call
    ) external override {
        xcmtransactor.transact_through_derivative(
            transactor,
            index,
            currency_id,
            weight,
            inner_call
        );
    }

    function transact_through_signed(
        Multilocation memory dest,
        address fee_location_address,
        uint64 weight,
        bytes memory call
    ) external override {
        xcmtransactor.transact_through_signed(
            dest,
            fee_location_address,
            weight,
            call
        );
    }

    function transact_through_signed_multilocation(
        Multilocation memory dest,
        Multilocation memory fee_location,
        uint64 weight,
        bytes memory call
    ) external override {
        xcmtransactor.transact_through_signed_multilocation(
            dest,
            fee_location,
            weight,
            call
        );
    }
}
