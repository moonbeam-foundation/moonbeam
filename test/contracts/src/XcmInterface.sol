// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @dev The XCM contract's address.
address constant XCM_CONTRACT_ADDRESS = 0x000000000000000000000000000000000000081A;

/// @dev The XCM contract's instance.
XCM constant XCM_CONTRACT = XCM(XCM_CONTRACT_ADDRESS);

/// @author The Moonbeam Team
/// @title XCM precompile Interface
/// @dev The interface that Solidity contracts use to interact with the substrate pallet-xcm.
interface XCM {
    // A location is defined by its number of parents and the encoded junctions (interior)
    struct Location {
        uint8 parents;
        bytes[] interior;
    }

    // Support for Weights V2
    struct Weight {
        uint64 refTime;
        uint64 proofSize;
    }

    // A way to represent fungible assets in XCM using Location format
    struct AssetLocationInfo {
        Location location;
        uint256 amount;
    }

    // A way to represent fungible assets in XCM using address format
    struct AssetAddressInfo {
        address asset;
        uint256 amount;
    }

    // The values start at `0` and are represented as `uint8`
    enum TransferType {
        Teleport,
        LocalReserve,
        DestinationReserve
    }

    /// @dev Function to send assets via XCM using transfer_assets() pallet-xcm extrinsic.
    /// @custom:selector 9ea8ada7
    /// @param dest The destination chain.
    /// @param beneficiary The actual account that will receive the tokens on dest.
    /// @param assets The combination (array) of assets to send in Location format.
    /// @param feeAssetItem The index of the asset that will be used to pay for fees.
    function transferAssetsLocation(
        Location memory dest,
        Location memory beneficiary,
        AssetLocationInfo[] memory assets,
        uint32 feeAssetItem
    ) external;

    /// @dev Function to send assets via XCM to a 20 byte-like parachain 
    /// using transfer_assets() pallet-xcm extrinsic.
    /// @custom:selector a0aeb5fe
    /// @param paraId The para-id of the destination chain.
    /// @param beneficiary The actual account that will receive the tokens on paraId destination.
    /// @param assets The combination (array) of assets to send in Address format.
    /// @param feeAssetItem The index of the asset that will be used to pay for fees.
    function transferAssetsToPara20(
        uint32 paraId,
        address beneficiary,
        AssetAddressInfo[] memory assets,
        uint32 feeAssetItem
    ) external;

    /// @dev Function to send assets via XCM to a 32 byte-like parachain 
    /// using transfer_assets() pallet-xcm extrinsic.
    /// @custom:selector f23032c3
    /// @param paraId The para-id of the destination chain.
    /// @param beneficiary The actual account that will receive the tokens on paraId destination.
    /// @param assets The combination (array) of assets to send in Address format.
    /// @param feeAssetItem The index of the asset that will be used to pay for fees.
    function transferAssetsToPara32(
        uint32 paraId,
        bytes32 beneficiary,
        AssetAddressInfo[] memory assets,
        uint32 feeAssetItem
    ) external;

    /// @dev Function to send assets via XCM to the relay chain 
    /// using transfer_assets() pallet-xcm extrinsic.
    /// @custom:selector 6521cc2c
    /// @param beneficiary The actual account that will receive the tokens on the relay chain.
    /// @param assets The combination (array) of assets to send in Address format.
    /// @param feeAssetItem The index of the asset that will be used to pay for fees.
    function transferAssetsToRelay(
        bytes32 beneficiary,
        AssetAddressInfo[] memory assets,
        uint32 feeAssetItem
    ) external;

    /// @dev Function to send assets through transfer_assets_using_type_and_then() pallet-xcm
    /// extrinsic.
    /// Important: in this selector RemoteReserve type (for either assets or fees) is not allowed.
    /// If users want to send assets and fees (in Location format) with a remote reserve, 
    /// they must use the selector fc19376c.
    /// @custom:selector 8425d893
    /// @param dest The destination chain.
    /// @param assets The combination (array) of assets to send in Location format.
    /// @param assetsTransferType The TransferType corresponding to assets being sent.
    /// @param remoteFeesIdIndex The index of the asset (inside assets array) to use as fees.
    /// @param feesTransferType The TransferType corresponding to the asset used as fees.
    /// @param customXcmOnDest The XCM message to execute on destination chain.
    function transferAssetsUsingTypeAndThenLocation(
        Location memory dest,
        AssetLocationInfo[] memory assets,
        TransferType assetsTransferType,
        uint8 remoteFeesIdIndex,
        TransferType feesTransferType,
        bytes memory customXcmOnDest
    ) external;

    /// @dev Function to send assets through transfer_assets_using_type_and_then() pallet-xcm
    /// extrinsic.
    /// @custom:selector fc19376c
    /// @param dest The destination chain.
    /// @param assets The combination (array) of assets to send in Location format.
    /// @param remoteFeesIdIndex The index of the asset (inside assets array) to use as fees.
    /// @param customXcmOnDest The XCM message to execute on destination chain.
    /// @param remoteReserve The remote reserve corresponding for assets and fees. They MUST
    /// share the same reserve.
    function transferAssetsUsingTypeAndThenLocation(
        Location memory dest,
        AssetLocationInfo[] memory assets,
        uint8 remoteFeesIdIndex,
        bytes memory customXcmOnDest,
        Location memory remoteReserve
    ) external;

    /// @dev Function to send assets through transfer_assets_using_type_and_then() pallet-xcm
    /// extrinsic.
    /// Important: in this selector RemoteReserve type (for either assets or fees) is not allowed.
    /// If users want to send assets and fees (in Address format) with a remote reserve, 
    /// they must use the selector aaecfc62.
    /// @custom:selector 998093ee
    /// @param dest The destination chain.
    /// @param assets The combination (array) of assets to send in Address format.
    /// @param assetsTransferType The TransferType corresponding to assets being sent.
    /// @param remoteFeesIdIndex The index of the asset (inside assets array) to use as fees.
    /// @param feesTransferType The TransferType corresponding to the asset used as fees.
    /// @param customXcmOnDest The XCM message to execute on destination chain.
    function transferAssetsUsingTypeAndThenAddress(
        Location memory dest,
        AssetAddressInfo[] memory assets,
        TransferType assetsTransferType,
        uint8 remoteFeesIdIndex,
        TransferType feesTransferType,
        bytes memory customXcmOnDest
    ) external;

    /// @dev Function to send assets through transfer_assets_using_type_and_then() pallet-xcm
    /// extrinsic.
    /// @custom:selector aaecfc62
    /// @param dest The destination chain.
    /// @param assets The combination (array) of assets to send in Address format.
    /// @param remoteFeesIdIndex The index of the asset (inside assets array) to use as fees.
    /// @param customXcmOnDest The XCM message to execute on destination chain.
    /// @param remoteReserve The remote reserve corresponding for assets and fees. They MUST
    /// share the same reserve.
    function transferAssetsUsingTypeAndThenAddress(
        Location memory dest,
        AssetAddressInfo[] memory assets,
        uint8 remoteFeesIdIndex,
        bytes memory customXcmOnDest,
        Location memory remoteReserve
    ) external;
}