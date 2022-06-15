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

    /** Get index of an account in xcm transactor
     * Selector 71b0edfa
     * @param index The index of which we want to retrieve the account
    * @return owner The owner of the derivative index
     */
    function index_to_account(uint16 index) external view returns(address owner);

    /// DEPRECATED, replaced by transact_info_with_signed
    /** Get transact info of a multilocation
     * Selector f87f493f
     * @param multilocation The location for which we want to know the transact info
     * @return transact_extra_weight The extra weight involved in the XCM message of using derivative
     * @return fee_per_second The amount of fee charged for a second of execution in the dest
     * @return max_weight Maximum allowed weight for a single message in dest
     */
    function transact_info(Multilocation memory multilocation) external view 
        returns(uint64 transact_extra_weight, uint256 fee_per_second, uint64 max_weight);
    
    /** Get transact info of a multilocation
     * Selector cb26bf32
     * @param multilocation The location for which we want to know the transact info
     * @return transact_extra_weight The extra weight involved in the XCM message of using derivative
     * @return transact_extra_weight_signed The extra weight involved in the XCM message of using signed
     * @return max_weight Maximum allowed weight for a single message in dest
     */
    function transact_info_with_signed(Multilocation memory multilocation) external view 
        returns(uint64 transact_extra_weight, uint64 transact_extra_weight_signed, uint64 max_weight);

    /** Get fee per second charged in its reserve chain for an asset
     * Selector 83f09082
     * @param multilocation The asset location for which we want to know the fee per second value
     * @return fee_per_second The fee per second that the reserve chain charges for this asset
     */
    function fee_per_second(Multilocation memory multilocation) external view 
        returns(uint256 fee_per_second);

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

    /** Transact through XCM using fee based on its multilocation through signed origins
    * Selector 19760407
    * @dev No token is burnt before sending the message. The caller must ensure the destination
    * is able to undertand the DescendOrigin message, and create a unique account from which
    * dispatch the call
    * @param dest The destination chain (as multilocation) where to send the message
    * @param fee_location The asset multilocation that indentifies the fee payment currency
    * It has to be a reserve of the destination chain
    * @param weight The weight we want to buy in the destination chain for the call to be made
    * @param call The call to be executed in the destination chain
    */
    function transact_through_signed_multilocation(
        Multilocation memory dest,
        Multilocation memory fee_location,
        uint64 weight,
        bytes memory call
    ) external;

    /** Transact through XCM using fee based on its erc20 address through signed origins
    * Selector 0139d697
    * @dev No token is burnt before sending the message. The caller must ensure the destination
    * is able to undertand the DescendOrigin message, and create a unique account from which
    * dispatch the call
    * @param dest The destination chain (as multilocation) where to send the message
    * @param fee_location_address The ERC20 address of the token we want to use to pay for fees
    * only callable if such an asset has been BRIDGED to our chain
    * @param weight The weight we want to buy in the destination chain for the call to be made
    * @param call The call to be executed in the destination chain
    */
    function transact_through_signed(
        Multilocation memory dest,
        address fee_location_address,
        uint64 weight,
        bytes memory call
    ) external;
}