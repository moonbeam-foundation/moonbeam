// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @title Xcm Transactor Interface
 * The interface through which solidity contracts will interact with xcm transactor pallet
 * Address :    0x0000000000000000000000000000000000000806
 */

interface XcmTransactor {

    // A multilocation is defined by its number of parents and the encoded junctions (interior)
    struct Multilocation {
        uint8 parents;
        bytes [] interior;
    }


    function multilocationToAccount(Multilocation memory multilocation) external view returns(address account);

}