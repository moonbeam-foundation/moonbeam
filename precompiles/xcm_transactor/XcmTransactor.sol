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
         *
         * @param account The account of which we want to retrieve the index
         */
        function account_index(address account) external view returns(uint16);

        /** Transfer a token through XCM based on its currencyId
         *
         * @dev The token transfer burns/transfers the corresponding amount before sending
         * @param transactor The transactor to be used
         * @param index The index to be used
         * @param fee_asset The asset in which we want to pay fees. It has to be a reserve of the destination chain
         * @param amount The amount of tokens we want to transfer
         * @param weight The weight we want to buy in the destination chain
         * @param inner_call The inner call to be executed in the destination chain
         */
        function transact_through_derivative(
            uint8 transactor,
            uint16 index,
            Multilocation memory fee_asset,
            uint256 amount,
            uint64 weight,
            bytes memory inner_call
        ) external;
    }