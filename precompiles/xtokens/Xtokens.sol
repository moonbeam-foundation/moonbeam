// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @title Xtokens Interface
 * The interface through which solidity contracts will interact with xtokens pallet
 * Address :    0x0000000000000000000000000000000000000804
 */

interface Xtokens {
    // A multilocation is defined by its number of parents and the encoded junctions (interior)
    struct Multilocation {
        uint8 parents;
        bytes[] interior;
    }

    // A MultiAsset is defined by a multilocation and an amount
    struct MultiAsset {
        Multilocation location;
        uint256 amount;
    }

    // A Currency is defined by address and the amount to be transferred
    struct Currency {
        address currency_address;
        uint256 amount;
    }

    /** Transfer a token through XCM based on its currencyId
     *
     * @dev The token transfer burns/transfers the corresponding amount before sending
     * @param currency_address The ERC20 address of the currency we want to transfer
     * @param amount The amount of tokens we want to transfer
     * @param destination The Multilocation to which we want to send the tokens
     * @param destination The weight we want to buy in the destination chain
     * Selector: b9f813ff
     */
    function transfer(
        address currency_address,
        uint256 amount,
        Multilocation memory destination,
        uint64 weight
    ) external;

    /** Transfer a token through XCM based on its currencyId specifying fee
     *
     * @dev The token transfer burns/transfers the corresponding amount before sending
     * @param currency_address The ERC20 address of the currency we want to transfer
     * @param amount The amount of tokens we want to transfer
     * @param destination The Multilocation to which we want to send the tokens
     * @param destination The weight we want to buy in the destination chain
     */
    function transfer_with_fee(
        address currency_address,
        uint256 amount,
        uint256 fee,
        Multilocation memory destination,
        uint64 weight
    ) external;

    /** Transfer a token through XCM based on its MultiLocation
     *
     * @dev The token transfer burns/transfers the corresponding amount before sending
     * @param asset The asset we want to transfer, defined by its multilocation.
     * Currently only Concrete Fungible assets
     * @param amount The amount of tokens we want to transfer
     * @param destination The Multilocation to which we want to send the tokens
     * @param destination The weight we want to buy in the destination chain
     * Selector: b38c60fa
     */
    function transfer_multiasset(
        Multilocation memory asset,
        uint256 amount,
        Multilocation memory destination,
        uint64 weight
    ) external;

    /** Transfer a token through XCM based on its MultiLocation specifying fee
     *
     * @dev The token transfer burns/transfers the corresponding amount before sending
     * @param asset The asset we want to transfer, defined by its multilocation.
     * Currently only Concrete Fungible assets
     * @param amount The amount of tokens we want to transfer
     * @param destination The Multilocation to which we want to send the tokens
     * @param destination The weight we want to buy in the destination chain
     * Selector: 89a570fc
     */
    function transfer_multiasset_with_fee(
        Multilocation memory asset,
        uint256 amount,
        uint256 fee,
        Multilocation memory destination,
        uint64 weight
    ) external;

    /** Transfer several tokens at once through XCM based on its address specifying fee
     *
     * @dev The token transfer burns/transfers the corresponding amount before sending
     * @param currencies The currencies we want to transfer, defined by their address and amount.
     * @param fee_item Which of the currencies to be used as fee
     * @param destination The Multilocation to which we want to send the tokens
     * @param weight The weight we want to buy in the destination chain
     * Selector: 8a362d5c
     */
    function transfer_multi_currencies(
        Currency[] memory currencies,
        uint32 fee_item,
        Multilocation memory destination,
        uint64 weight
    ) external;

    /** Transfer several tokens at once through XCM based on its location specifying fee
     *
     * @dev The token transfer burns/transfers the corresponding amount before sending
     * @param assets The assets we want to transfer, defined by their location and amount.
     * @param fee_item Which of the currencies to be used as fee
     * @param destination The Multilocation to which we want to send the tokens
     * @param weight The weight we want to buy in the destination chain
     * Selector: b38c60fa
     */
    function transfer_multi_assets(
        MultiAsset[] memory assets,
        uint32 fee_item,
        Multilocation memory destination,
        uint64 weight
    ) external;
}
