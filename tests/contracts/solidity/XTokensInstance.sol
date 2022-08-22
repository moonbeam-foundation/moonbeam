// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

import "../../../precompiles/xtokens/Xtokens.sol";

contract XtokensInstance is Xtokens {
    /// The Xtokens wrapper at the known pre-compile address.
    Xtokens public xtokens =
        Xtokens(0x0000000000000000000000000000000000000804);

    function transfer(
        address currencyAddress,
        uint256 amount,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transfer(currencyAddress, amount, destination, weight);
    }

    function transferWithFee(
        address currencyAddress,
        uint256 amount,
        uint256 fee,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transferWithFee(
            currencyAddress,
            amount,
            fee,
            destination,
            weight
        );
    }

    function transferMultiasset(
        Multilocation memory asset,
        uint256 amount,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transferMultiasset(asset, amount, destination, weight);
    }

    function transferMultiassetWithFee(
        Multilocation memory asset,
        uint256 amount,
        uint256 fee,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transferMultiassetWithFee(
            asset,
            amount,
            fee,
            destination,
            weight
        );
    }

    function transferMultiCurrencies(
        Currency[] memory currencies,
        uint32 feeItem,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transferMultiCurrencies(
            currencies,
            feeItem,
            destination,
            weight
        );
    }

    function transferMultiAssets(
        MultiAsset[] memory assets,
        uint32 feeItem,
        Multilocation memory destination,
        uint64 weight
    ) external override {
        xtokens.transferMultiAssets(assets, feeItem, destination, weight);
    }
}
