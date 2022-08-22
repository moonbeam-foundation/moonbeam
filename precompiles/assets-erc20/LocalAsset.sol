// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @author The Moonbeam Team
/// @title Extension of the ERC20 interface that allows an owner
/// @dev Contract to pilot the ERC20 contract.
interface LocalAsset {
    /// @dev Mint tokens to an address
    /// @custom:selector 40c10f19
    /// @param to address The address to which you want to mint tokens
    /// @param value uint256 the amount of tokens to be minted
    function mint(address to, uint256 value) external returns (bool);

    /// @dev Burn tokens from an address
    /// @custom:selector 9dc29fac
    /// @param from address The address from which you want to burn tokens
    /// @param value uint256 the amount of tokens to be burnt
    function burn(address from, uint256 value) external returns (bool);

    /// @dev Freeze an account, preventing it from operating with the asset
    /// @custom:selector 8d1fdf2f
    /// @param account address The address that you want to freeze
    function freeze(address account) external returns (bool);

    /// @dev Unfreeze an account, letting it from operating againt with the asset
    /// @custom:selector 5ea20216
    /// @param account address The address that you want to unfreeze
    function thaw(address account) external returns (bool);

    /// @dev Freeze the entire asset operations
    /// @custom:selector d4937f51
    function freezeAsset() external returns (bool);

    /// @dev Unfreeze the entire asset operations
    /// @custom:selector 51ec2ad7
    function thawAsset() external returns (bool);

    /// @dev Transfer the ownership of an asset to a new account
    /// @custom:selector f2fde38b
    /// @param owner address The address of the new owner
    function transferOwnership(address owner) external returns (bool);

    /// @dev Specify the issuer, admin and freezer of an asset
    /// @custom:selector c7d93c59
    /// @param issuer address The address capable of issuing tokens
    /// @param admin address The address capable of burning tokens and unfreezing accounts/assets
    /// @param freezer address The address capable of freezing accounts/asset
    function setTeam(
        address issuer,
        address admin,
        address freezer
    ) external returns (bool);

    /// @dev Specify the name, symbol and decimals of your asset
    /// @custom:selector 37d2c2f4
    /// @param name string The name of the asset
    /// @param symbol string The symbol of the asset
    /// @param decimals uint8 The number of decimals of your asset
    function setMetadata(
        string calldata name,
        string calldata symbol,
        uint8 decimals
    ) external returns (bool);

    /// @dev Clear the name, symbol and decimals of your asset
    /// @custom:selector efb6d432
    function clearMetadata() external returns (bool);
}
