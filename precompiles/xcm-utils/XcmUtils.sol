// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @title Xcm Utils Interface
 * The interface through which solidity contracts will interact with xcm utils pallet
 * Address :    0x000000000000000000000000000000000000080B
 */

interface XcmUtils {

    // A multilocation is defined by its number of parents and the encoded junctions (interior)
    struct Multilocation {
        uint8 parents;
        bytes [] interior;
    }

     /** Retrieve the account associated to a given MultiLocation
     *
     * @param multilocation The multilocation that we want to know to which account maps to
    * @return account The account the multilocation maps to in this chain

     * Selector: 343b3e00
     */
    function multilocationToAccount(Multilocation memory multilocation) external view returns(address account);

}