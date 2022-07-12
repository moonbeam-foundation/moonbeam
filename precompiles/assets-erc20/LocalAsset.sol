pragma solidity ^0.8.0;

/**
 * @title Extension of the ERC20 interface that allows an owner
 * contract to pilot the ERC20 contract.
 */
interface LocalAsset {
    /**
     * @dev Mint tokens to an address
     * Selector: 40c10f19
     * @param to address The address to which you want to mint tokens
     * @param value uint256 the amount of tokens to be minted
     */
    function mint(address to, uint256 value)
        external returns (bool);

    /**
     * @dev Burn tokens from an address
     * Selector: 9dc29fac
     * @param from address The address from which you want to burn tokens
     * @param value uint256 the amount of tokens to be burnt
     */
    function burn(address from, uint256 value)
        external returns (bool);

    /**
     * @dev Freeze an account, preventing it from operating with the asset
     * Selector: 8d1fdf2f
     * @param account address The address that you want to freeze
     */
    function freeze(address account)
        external returns (bool);

    /**
     * @dev Unfreeze an account, letting it from operating againt with the asset
     * Selector: 5ea20216
     * @param account address The address that you want to unfreeze
     */
    function thaw(address account)
        external returns (bool);

    /**
     * @dev Freeze the entire asset operations
     * Selector: 6b8751c1
     */
    function freeze_asset()
        external returns (bool);

    /**
     * @dev Unfreeze the entire asset operations
     * Selector: 1cddec19
     */
    function thaw_asset()
        external returns (bool);

    /**
     * @dev Transfer the ownership of an asset to a new account
     * Selector: f0350c04
     * @param owner address The address of the new owner
     */
    function transfer_ownership(address owner)
        external returns (bool);
    
    /**
     * @dev Specify the issuer, admin and freezer of an asset
     * Selector: f8bf8e95
     * @param issuer address The address capable of issuing tokens
     * @param admin address The address capable of burning tokens and unfreezing accounts/assets
     * @param freezer address The address capable of freezing accounts/asset
     */
    function set_team(address issuer, address admin, address freezer)
        external returns (bool);

    /**
     * @dev Specify the name, symbol and decimals of your asset
     * Selector: ee5dc1e4
     * @param name string The name of the asset
     * @param symbol string The symbol of the asset
     * @param decimals uint8 The number of decimals of your asset
     */
    function set_metadata(string calldata name, string calldata symbol, uint8 decimals)
        external returns (bool);

    /**
     * @dev Clear the name, symbol and decimals of your asset
     * Selector: d3ba4b9e
     */
    function clear_metadata()
        external returns (bool);
}