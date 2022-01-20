// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

    /**
     * @title Xcm Transactor Interface
     *
     * The interface through which solidity contracts will interact with xcm transactor pallet
     *
     */
    interface XcmTransactor {
        // A multilocation is defined by its number of parents and the encoded junctions (interior)
        struct Multilocation {
            uint8 parents;
            bytes [] interior;
        }

        /** Get index of an account in xcm transactor
         * Selector 71b0edfa
         * @param index The index of which we want to retrieve the account
         */
        function index_to_account(uint16 index) external view returns(address);

        /** Get transact info of a multilocation
         * Selector f87f493f
         * @param index The index of which we want to retrieve the account
         */
        function transact_info(Multilocation memory multilocation) external view 
            returns(uint64, uint256, uint64);

        /** Transact through XCM using fee based on its multilocation
        * Selector 9f89f03e
        * @dev The token transfer burns/transfers the corresponding amount before sending
        * @param transactor The transactor to be used
        * @param index The index to be used
        * @param fee_asset The asset in which we want to pay fees. 
        * It has to be a reserve of the destination chain
        * @param weight The weight we want to buy in the destination chain
        * @param inner_call The inner call to be executed in the destination chain
        */
        function transact_through_derivative_multilocation(
            uint8 transactor,
            uint16 index,
            Multilocation memory fee_asset,
            uint64 weight,
            bytes memory inner_call
        ) external;

        /** Transact through XCM using fee based on its currency_id
        * Selector 267d4062
        * @dev The token transfer burns/transfers the corresponding amount before sending
        * @param transactor The transactor to be used
        * @param index The index to be used
        * @param currency_id Address of the currencyId of the asset to be used for fees
        * It has to be a reserve of the destination chain
        * @param weight The weight we want to buy in the destination chain
        * @param inner_call The inner call to be executed in the destination chain
        */
        function transact_through_derivative(
            uint8 transactor,
            uint16 index,
            address currency_id,
            uint64 weight,
            bytes memory inner_call
        ) external;
    }