// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @author The Moonbeam Team
/// @title ERC20 interface Asset Roles
/// @dev Extension of the ERC20 interface that allows users to get the account capable of fulfilling different asset roles
/// @custom:address 0xFFFFFFFE + hex(assetId)
interface Roles {
    /// @dev Function to check the owner of the asset
    /// @custom:selector 8da5cb5b
    /// @return the address of the owner.
    function owner()
        external
        view
        returns (address);

    /// @dev Function to check the freezer of the asset
    /// @dev Freezer: the account that can freeze an asset
    /// @custom:selector 92716054
    /// @return the address of the freezer.
    function freezer()
        external
        view
        returns (address);

    /// @dev Function to check the issuer of the asset
    /// @dev Issuer: the account that can issue tokens for an asset
    /// @custom:selector 1d143848
    /// @return the address of the issuer.
    function issuer()
        external
        view
        returns (address);

    /// @dev Function to check the admin of the asset
    /// @dev Admin: the account that can unfreeze and force transfer
    /// @custom:selector f851a440
    /// @return the address of the admin.
    function admin()
        external
        view
        returns (address);
}
