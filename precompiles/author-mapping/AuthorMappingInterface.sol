// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @title Pallet AuthorMapping Interface
 *
 * The interface through which solidity contracts will interact with pallet-author.mapping
 * Moonbase address :    0x0000000000000000000000000000000000000807
 */
 
interface AuthorMapping {
    /**
     * Add association
     * Selector: aa5ac585   
     *
     * @param nimbus_id The nimbusId to be associated
     */
    function add_association(bytes32 nimbus_id) external;

    /**
     * Update existing association
     * Selector: d9cef879
     *
     * @param old_nimbus_id The old nimbusId to be replaced
     * @param new_nimbus_id The new nimbusId to be associated
     */
    function update_association(bytes32 old_nimbus_id, bytes32 new_nimbus_id) external;

     /**
     * Clear existing associationg
     * Selector: 7354c91d
     *
     * @param nimbus_id The nimbusId to be cleared
     */
    function clear_association(bytes32 nimbus_id) external;
}
