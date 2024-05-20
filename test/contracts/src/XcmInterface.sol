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

    /// @dev Function to send assets via XCM using transfer_assets() pallet-xcm extrinsic.
    /// @custom:selector 59df8416
    /// @param dest The destination chain.
    /// @param beneficiary The actual account that will receive the tokens on dest.
    /// @param assets The combination (array) of assets to send.
    /// @param feeAssetItem The index of the asset that will be used to pay for fees.
    /// @param weight The weight to be used for the whole XCM operation.
    /// (uint64::MAX in refTime means Unlimited weight) 
    function transferAssetsLocation(
        Location memory dest,
        Location memory beneficiary,
        AssetLocationInfo[] memory assets,
        uint32 feeAssetItem,
        Weight memory weight
    ) external;

    /// @dev Function to send assets via XCM to a 20 byte-like parachain 
    /// using transfer_assets() pallet-xcm extrinsic.
    /// @custom:selector b489262e
    /// @param paraId The para-id of the destination chain.
    /// @param beneficiary The actual account that will receive the tokens on paraId destination.
    /// @param assets The combination (array) of assets to send.
    /// @param feeAssetItem The index of the asset that will be used to pay for fees.
    /// @param weight The weight to be used for the whole XCM operation.
    /// (uint64::MAX in refTime means Unlimited weight)
    function transferAssetsToPara20(
        uint32 paraId,
        address beneficiary,
        AssetAddressInfo[] memory assets,
        uint32 feeAssetItem,
        Weight memory weight
    ) external;

    /// @dev Function to send assets via XCM to a 32 byte-like parachain 
    /// using transfer_assets() pallet-xcm extrinsic.
    /// @custom:selector 4461e6f5
    /// @param paraId The para-id of the destination chain.
    /// @param beneficiary The actual account that will receive the tokens on paraId destination.
    /// @param assets The combination (array) of assets to send.
    /// @param feeAssetItem The index of the asset that will be used to pay for fees.
    /// @param weight The weight to be used for the whole XCM operation.
    /// (uint64::MAX in refTime means Unlimited weight)
    function transferAssetsToPara32(
        uint32 paraId,
        bytes32 beneficiary,
        AssetAddressInfo[] memory assets,
        uint32 feeAssetItem,
        Weight memory weight
    ) external;

    /// @dev Function to send assets via XCM to the relay chain 
    /// using transfer_assets() pallet-xcm extrinsic.
    /// @custom:selector d7c89659
    /// @param beneficiary The actual account that will receive the tokens on the relay chain.
    /// @param assets The combination (array) of assets to send.
    /// @param feeAssetItem The index of the asset that will be used to pay for fees.
    /// @param weight The weight to be used for the whole XCM operation.
    /// (uint64::MAX in refTime means Unlimited weight)
    function transferAssetsToRelay(
        bytes32 beneficiary,
        AssetAddressInfo[] memory assets,
        uint32 feeAssetItem,
        Weight memory weight
    ) external;
}