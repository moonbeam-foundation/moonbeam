// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The XcmTransactorV1 contract's address.
address constant XCM_TRANSACTOR_V1_ADDRESS = 0x0000000000000000000000000000000000000806;

/// @dev The XcmTransactorV1 contract's instance.
XcmTransactorV1 constant XCM_TRANSACTOR_V1_CONTRACT = XcmTransactorV1(
    XCM_TRANSACTOR_V1_ADDRESS
);

/// @author The Moonbeam Team
/// @title Xcm Transactor Interface
/// @dev The interface through which solidity contracts will interact with xcm transactor pallet
/// @custom:address 0x0000000000000000000000000000000000000806
interface XcmTransactorV1 {
    // A multilocation is defined by its number of parents and the encoded junctions (interior)
    struct Multilocation {
        uint8 parents;
        bytes[] interior;
    }

    /// Get index of an account in xcm transactor
    /// @custom:selector 3fdc4f36
    /// @param index The index of which we want to retrieve the account
    /// @return owner The owner of the derivative index
    ///
    function indexToAccount(uint16 index) external view returns (address owner);

    /// DEPRECATED, replaced by transactInfoWithSigned
    /// Get transact info of a multilocation
    /// @custom:selector d07d87c3
    /// @param multilocation The location for which we want to know the transact info
    /// @return transactExtraWeight The extra weight involved in the XCM message of using derivative
    /// @return feePerSecond The amount of fee charged for a second of execution in the dest
    /// @return maxWeight Maximum allowed weight for a single message in dest
    ///
    function transactInfo(Multilocation memory multilocation)
        external
        view
        returns (
            uint64 transactExtraWeight,
            uint256 feePerSecond,
            uint64 maxWeight
        );

    /// Get transact info of a multilocation
    /// @custom:selector b689e20c
    /// @param multilocation The location for which we want to know the transact info
    /// @return transactExtraWeight The extra weight involved in the XCM message of using derivative
    /// @return transactExtraWeightSigned The extra weight involved in the XCM message of using signed
    /// @return maxWeight Maximum allowed weight for a single message in dest
    ///
    function transactInfoWithSigned(Multilocation memory multilocation)
        external
        view
        returns (
            uint64 transactExtraWeight,
            uint64 transactExtraWeightSigned,
            uint64 maxWeight
        );

    /// Get fee per second charged in its reserve chain for an asset
    /// @custom:selector 906c9990
    /// @param multilocation The asset location for which we want to know the fee per second value
    /// @return feePerSecond The fee per second that the reserve chain charges for this asset
    ///
    function feePerSecond(Multilocation memory multilocation)
        external
        view
        returns (uint256 feePerSecond);

    /// Transact through XCM using fee based on its multilocation
    /// @custom:selector 94a63c54
    /// @dev The token transfer burns/transfers the corresponding amount before sending
    /// @param transactor The transactor to be used
    /// @param index The index to be used
    /// @param feeAsset The asset in which we want to pay fees.
    /// It has to be a reserve of the destination chain
    /// @param weight The weight we want to buy in the destination chain
    /// @param innerCall The inner call to be executed in the destination chain
    function transactThroughDerivativeMultilocation(
        uint8 transactor,
        uint16 index,
        Multilocation memory feeAsset,
        uint64 weight,
        bytes memory innerCall
    ) external;

    /// Transact through XCM using fee based on its currencyId
    /// @custom:selector 02ae072d
    /// @dev The token transfer burns/transfers the corresponding amount before sending
    /// @param transactor The transactor to be used
    /// @param index The index to be used
    /// @param currencyId Address of the currencyId of the asset to be used for fees
    /// It has to be a reserve of the destination chain
    /// @param weight The weight we want to buy in the destination chain
    /// @param innerCall The inner call to be executed in the destination chain
    function transactThroughDerivative(
        uint8 transactor,
        uint16 index,
        address currencyId,
        uint64 weight,
        bytes memory innerCall
    ) external;

    /// Transact through XCM using fee based on its multilocation through signed origins
    /// @custom:selector 71d31587
    /// @dev No token is burnt before sending the message. The caller must ensure the destination
    /// is able to undertand the DescendOrigin message, and create a unique account from which
    /// dispatch the call
    /// @param dest The destination chain (as multilocation) where to send the message
    /// @param feeLocation The asset multilocation that indentifies the fee payment currency
    /// It has to be a reserve of the destination chain
    /// @param weight The weight we want to buy in the destination chain for the call to be made
    /// @param call The call to be executed in the destination chain
    function transactThroughSignedMultilocation(
        Multilocation memory dest,
        Multilocation memory feeLocation,
        uint64 weight,
        bytes memory call
    ) external;

    /// Transact through XCM using fee based on its erc20 address through signed origins
    /// @custom:selector 42ca339d
    /// @dev No token is burnt before sending the message. The caller must ensure the destination
    /// is able to undertand the DescendOrigin message, and create a unique account from which
    /// dispatch the call
    /// @param dest The destination chain (as multilocation) where to send the message
    /// @param feeLocationAddress The ERC20 address of the token we want to use to pay for fees
    /// only callable if such an asset has been BRIDGED to our chain
    /// @param weight The weight we want to buy in the destination chain for the call to be made
    /// @param call The call to be executed in the destination chain
    function transactThroughSigned(
        Multilocation memory dest,
        address feeLocationAddress,
        uint64 weight,
        bytes memory call
    ) external;

    /// @dev Encode 'utility.as_derivative' relay call
    /// @custom:selector ff86378d
    /// @param transactor The transactor to be used
    /// @param index: The derivative index to use
    /// @param innerCall: The inner call to be executed from the derivated address
    /// @return result The bytes associated with the encoded call
    function encodeUtilityAsDerivative(uint8 transactor, uint16 index, bytes memory innerCall)
        external
        pure
        returns (bytes memory result);
}