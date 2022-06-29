// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

import "../../../precompiles/xtokens/Xtokens.sol";

contract XtokensInstance is Xtokens {
    /// The Xtokens wrapper at the known pre-compile address.
    Xtokens public xtokens =
        Xtokens(0x0000000000000000000000000000000000000804);

    function transfer(
        address currency_address,
        uint256 amount,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transfer(currency_address, amount, destination, weight);
    }

    function transfer_with_fee(
        address currency_address,
        uint256 amount,
        uint256 fee,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transfer_with_fee(
            currency_address,
            amount,
            fee,
            destination,
            weight
        );
    }

    function transfer_multiasset(
        Multilocation memory asset,
        uint256 amount,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transfer_multiasset(asset, amount, destination, weight);
    }

    function transfer_multiasset_with_fee(
        Multilocation memory asset,
        uint256 amount,
        uint256 fee,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transfer_multiasset_with_fee(
            asset,
            amount,
            fee,
            destination,
            weight
        );
    }

    function transfer_multi_currencies(
        Currency[] memory currencies,
        uint32 fee_item,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transfer_multi_currencies(
            currencies,
            fee_item,
            destination,
            weight
        );
    }

    function transfer_multi_assets(
        MultiAsset[] memory assets,
        uint32 fee_item,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transfer_multi_assets(assets, fee_item, destination, weight);
    }
}
