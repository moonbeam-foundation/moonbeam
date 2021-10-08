// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @title Xtokens Interface
 *
 * The interface through which solidity contracts will interact with xtokens pallet
 *
 */
interface Xtokens {
    // A multilocation is defined by its number of parents and the encoded junctions (interior)
    struct Multilocation {
        uint8 parents;
        bytes [] interior;
    }

    /** Transfer a token through XCM based on its currencyId
     *
     * @dev The token transfer burns/transfers the corresponding amount before sending
     * @param currency_address The ERC20 address of the currency we want to transfer
     * @param amount The amount of tokens we want to transfer
     * @param destination The Multilocation to which we want to send the tokens
     * @param destination The weight we want to buy in the destination chain
     */
    function transfer(address currency_address, uint256 amount, Multilocation memory destination, uint64 weight) external;

    /** Transfer a token through XCM based on its currencyId
     *
     * @dev The token transfer burns/transfers the corresponding amount before sending
     * @param asset The asset we want to transfer, defined by its multilocation. Currently only Concrete Fungible assets
     * @param amount The amount of tokens we want to transfer
     * @param destination The Multilocation to which we want to send the tokens
     * @param destination The weight we want to buy in the destination chain
     */
    function transfer_multiasset(Multilocation memory asset, uint256 amount, Multilocation memory destination, uint64 weight) external;
}

// Function selector reference
// {
// "b9f813ff": "transfer(address,uint256,(uint8,bytes[]),uint64)",
// "b38c60fa": "transfer_multiasset((uint8,bytes[]),uint256,(uint8,bytes[]),uint64)"
//}
