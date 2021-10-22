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

        /** Transfer a token through XCM based on its currencyId
         *
         * @dev The token transfer burns/transfers the corresponding amount before sending
         * Selector 93a8f668
         * @param transactor The transactor to be used
         * @param index The index to be used
         * @param fee_asset The asset in which we want to pay fees. It has to be a reserve of the destination chain
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

        /** Transfer a token through XCM based on its currencyId
         *
         * @dev The token transfer burns/transfers the corresponding amount before sending
         * Selector 93a8f668
         * @param transactor The transactor to be used
         * @param index The index to be used
         * @param currency_id The address of the ERC20 of the asset we want to use to pay for fees
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