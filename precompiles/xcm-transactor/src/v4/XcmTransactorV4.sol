// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The XcmTransactorV4 contract's address.
address constant XCM_TRANSACTOR_V4_ADDRESS = 0x000000000000000000000000000000000000081B;

/// @dev The XcmTransactorV4 contract's instance.
XcmTransactorV4 constant XCM_TRANSACTOR_V4_CONTRACT = XcmTransactorV4(
    XCM_TRANSACTOR_V4_ADDRESS
);

/// @author The Moonbeam Team
/// @title Xcm Transactor V4 Interface - AssetHub Support
/// @notice Interface for cross-chain transactions with AssetHub support
/// @dev V4 adds support for AssetHub as a transaction destination (transactor = 1)
/// @custom:address 0x000000000000000000000000000000000000081B
interface XcmTransactorV4 {
    /// @dev Supported destination chains
    /// @notice Relay = 0 (Polkadot/Kusama/Westend Relay Chain)
    /// @notice AssetHub = 1 (AssetHub system parachain)
    enum Transactor {
        Relay,
        AssetHub
    }

    /// @dev A multilocation is defined by its number of parents and the encoded junctions (interior)
    struct Multilocation {
        uint8 parents;
        bytes[] interior;
    }

    /// @dev Weight V2 structure
    struct Weight {
        uint64 refTime;
        uint64 proofSize;
    }

    /// @notice Get index of an account in xcm transactor
    /// @custom:selector 3fdc4f36
    /// @param index The index of which we want to retrieve the account
    /// @return owner The owner of the derivative index
    function indexToAccount(uint16 index) external view returns (address owner);

    /// @notice Get transact info of a multilocation
    /// @custom:selector b689e20c
    /// @param multilocation The location for which we want to know the transact info
    /// @return transactExtraWeight The extra weight involved in the XCM message of using derivative
    /// @return transactExtraWeightSigned The extra weight involved in the XCM message of using signed
    /// @return maxWeight Maximum allowed weight for a single message in dest
    function transactInfoWithSigned(Multilocation memory multilocation)
        external
        view
        returns (
            Weight memory transactExtraWeight,
            Weight memory transactExtraWeightSigned,
            Weight memory maxWeight
        );

    /// @notice Get fee per second charged in its reserve chain for an asset
    /// @custom:selector 906c9990
    /// @param multilocation The asset location for which we want to know the fee per second value
    /// @return feePerSecond The fee per second that the reserve chain charges for this asset
    function feePerSecond(Multilocation memory multilocation)
        external
        view
        returns (uint256 feePerSecond);

    /// @notice Transact through XCM using fee based on its multilocation
    /// @custom:selector bdacc26b
    /// @dev The token transfer burns/transfers the corresponding amount before sending
    /// @param transactor The target chain (0=Relay, 1=AssetHub)
    /// @param index The derivative account index
    /// @param feeAsset The asset in which we want to pay fees (must be a reserve of the destination chain)
    /// @param transactRequiredWeightAtMost The weight we want to buy in the destination chain
    /// @param innerCall The inner call to be executed in the destination chain
    /// @param feeAmount Amount to be used as fee
    /// @param overallWeight Overall weight to be used for the xcm message. If uint64::MAX is passed through refTime field, Unlimited variant will be used
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

    /// @notice Transact through XCM using fee based on its currency_id
    /// @custom:selector ca8c82d8
    /// @dev The token transfer burns/transfers the corresponding amount before sending
    /// @param transactor The target chain (0=Relay, 1=AssetHub)
    /// @param index The derivative account index
    /// @param currencyId Address of the currencyId of the asset to be used for fees
    /// @param transactRequiredWeightAtMost The weight we want to buy in the destination chain
    /// @param innerCall The inner call to be executed in the destination chain
    /// @param feeAmount Amount to be used as fee
    /// @param overallWeight Overall weight to be used for the xcm message
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

    /// @notice Transact through XCM using signed origin with multilocation fee asset
    /// @custom:selector 27b1d492
    /// @param dest The destination for the transaction
    /// @param feeAsset The asset in which we want to pay fees
    /// @param transactRequiredWeightAtMost The weight we want to buy in the destination chain
    /// @param innerCall The inner call to be executed in the destination chain
    /// @param feeAmount Amount to be used as fee
    /// @param overallWeight Overall weight to be used for the xcm message
    /// @param refund Indicates if RefundSurplus instruction will be appended
    function transactThroughSignedMultilocation(
        Multilocation memory dest,
        Multilocation memory feeAsset,
        Weight memory transactRequiredWeightAtMost,
        bytes memory innerCall,
        uint256 feeAmount,
        Weight memory overallWeight,
        bool refund
    ) external;

    /// @notice Transact through XCM using signed origin with currency_id fee asset
    /// @custom:selector b18270cf
    /// @param dest The destination for the transaction
    /// @param currencyId Address of the currencyId of the asset to be used for fees
    /// @param transactRequiredWeightAtMost The weight we want to buy in the destination chain
    /// @param innerCall The inner call to be executed in the destination chain
    /// @param feeAmount Amount to be used as fee
    /// @param overallWeight Overall weight to be used for the xcm message
    /// @param refund Indicates if RefundSurplus instruction will be appended
    function transactThroughSigned(
        Multilocation memory dest,
        address currencyId,
        Weight memory transactRequiredWeightAtMost,
        bytes memory innerCall,
        uint256 feeAmount,
        Weight memory overallWeight,
        bool refund
    ) external;

    /// @notice Encode a utility.asDerivative call
    /// @custom:selector ff86378d
    /// @param transactor The target chain (0=Relay, 1=AssetHub)
    /// @param index The derivative index
    /// @param innerCall The call to wrap
    /// @return result The SCALE-encoded asDerivative call
    function encodeUtilityAsDerivative(
        uint8 transactor,
        uint16 index,
        bytes memory innerCall
    ) external pure returns (bytes memory result);
}
