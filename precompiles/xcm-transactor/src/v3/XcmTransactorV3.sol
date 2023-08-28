// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @dev The XcmTransactorV3 contract's address.
address constant XCM_TRANSACTOR_V3_ADDRESS = 0x0000000000000000000000000000000000000817;

/// @dev The XcmTransactorV3 contract's instance.
XcmTransactorV3 constant XCM_TRANSACTOR_V3_CONTRACT = XcmTransactorV3(
    XCM_TRANSACTOR_V3_ADDRESS
);

/// @author The Moonbeam Team
/// @title Xcm Transactor Interface
/// The interface through which solidity contracts will interact with xcm transactor pallet
/// @custom:address 0x0000000000000000000000000000000000000817
interface XcmTransactorV3 {
    // A multilocation is defined by its number of parents and the encoded junctions (interior)
    struct Multilocation {
        uint8 parents;
        bytes[] interior;
    }

    // Support for Weights V2
    struct Weight {
        uint64 refTime;
        uint64 proofSize;
    }

    /// Get index of an account in xcm transactor
    /// @custom:selector 3fdc4f36
    /// @param index The index of which we want to retrieve the account
    /// @return owner The owner of the derivative index
    ///
    function indexToAccount(uint16 index) external view returns (address owner);

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
            Weight memory transactExtraWeight,
            Weight memory transactExtraWeightSigned,
            Weight memory maxWeight
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
    /// @custom:selector bdacc26b
    /// @dev The token transfer burns/transfers the corresponding amount before sending
    /// @param transactor The transactor to be used
    /// @param index The index to be used
    /// @param feeAsset The asset in which we want to pay fees.
    /// It has to be a reserve of the destination chain
    /// @param transactRequiredWeightAtMost The weight we want to buy in the destination chain
    /// @param innerCall The inner call to be executed in the destination chain
    /// @param feeAmount Amount to be used as fee.
    /// @param overallWeight Overall weight to be used for the xcm message.
    /// @param refund Indicates if RefundSurplus instruction will be appended
    function transactThroughDerivativeMultilocation(
        uint8 transactor,
        uint16 index,
        Multilocation memory feeAsset,
        Weight memory transactRequiredWeightAtMost,
        bytes memory innerCall,
        uint256 feeAmount,
        Weight memory overallWeight,
        bool refund
    ) external;

    /// Transact through XCM using fee based on its currency_id
    /// @custom:selector ca8c82d8
    /// @dev The token transfer burns/transfers the corresponding amount before sending
    /// @param transactor The transactor to be used
    /// @param index The index to be used
    /// @param currencyId Address of the currencyId of the asset to be used for fees
    /// It has to be a reserve of the destination chain
    /// @param transactRequiredWeightAtMost The weight we want to buy in the destination chain
    /// @param innerCall The inner call to be executed in the destination chain
    /// @param feeAmount Amount to be used as fee.
    /// @param overallWeight Overall weight to be used for the xcm message.
    /// @param refund Indicates if RefundSurplus instruction will be appended
    function transactThroughDerivative(
        uint8 transactor,
        uint16 index,
        address currencyId,
        Weight memory transactRequiredWeightAtMost,
        bytes memory innerCall,
        uint256 feeAmount,
        Weight memory overallWeight,
        bool refund
    ) external;

    /// Transact through XCM using fee based on its multilocation through signed origins
    /// @custom:selector 27b1d492
    /// @dev No token is burnt before sending the message. The caller must ensure the destination
    /// is able to undertand the DescendOrigin message, and create a unique account from which
    /// dispatch the call
    /// @param dest The destination chain (as multilocation) where to send the message
    /// @param feeLocation The asset multilocation that indentifies the fee payment currency
    /// It has to be a reserve of the destination chain
    /// @param transactRequiredWeightAtMost The weight we want to buy in the destination chain for the call to be made
    /// @param call The call to be executed in the destination chain
    /// @param feeAmount Amount to be used as fee.
    /// @param overallWeight Overall weight to be used for the xcm message.
    /// @param refund Indicates if RefundSurplus instruction will be appended
    function transactThroughSignedMultilocation(
        Multilocation memory dest,
        Multilocation memory feeLocation,
        Weight memory transactRequiredWeightAtMost,
        bytes memory call,
        uint256 feeAmount,
        Weight memory overallWeight,
        bool refund
    ) external;

    /// Transact through XCM using fee based on its erc20 address through signed origins
    /// @custom:selector b18270cf
    /// @dev No token is burnt before sending the message. The caller must ensure the destination
    /// is able to undertand the DescendOrigin message, and create a unique account from which
    /// dispatch the call
    /// @param dest The destination chain (as multilocation) where to send the message
    /// @param feeLocationAddress The ERC20 address of the token we want to use to pay for fees
    /// only callable if such an asset has been BRIDGED to our chain
    /// @param transactRequiredWeightAtMost The weight we want to buy in the destination chain for the call to be made
    /// @param call The call to be executed in the destination chain
    /// @param feeAmount Amount to be used as fee.
    /// @param overallWeight Overall weight to be used for the xcm message.
    /// @param refund Indicates if RefundSurplus instruction will be appended
    function transactThroughSigned(
        Multilocation memory dest,
        address feeLocationAddress,
        Weight memory transactRequiredWeightAtMost,
        bytes memory call,
        uint256 feeAmount,
        Weight memory overallWeight,
        bool refund
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
