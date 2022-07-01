 pragma solidity ^0.8.0;

/// @title ERC20 interface
interface LocalAssetExtendedErc20 {

    /// @dev Returns the name of the token.
    /// Selector: 06fdde03
    function name() external view returns (string memory);

    /// @dev Returns the symbol of the token.
    /// Selector: 95d89b41
    function symbol() external view returns (string memory);

    /// @dev Returns the decimals places of the token.
    /// Selector: 313ce567
    function decimals() external view returns (uint8);
    
    /// @dev Total number of tokens in existence
    /// Selector: 18160ddd
    function totalSupply() external view returns (uint256);

    /// @dev Gets the balance of the specified address.
    /// Selector: 70a08231
    /// @param who The address to query the balance of.
    /// @return An uint256 representing the amount owned by the
    function balanceOf(address who) external view returns (uint256);

    /// @dev Function to check the amount of tokens that an owner allowed to a spender.
    /// Selector: dd62ed3e
    /// @param owner address The address which owns the funds.
    /// @param spender address The address which will spend the funds.
    /// @return A uint256 specifying the amount of tokens still available for
    function allowance(address owner, address spender)
        external view returns (uint256);

    /// @dev Transfer token for a specified address
    /// Selector: a9059cbb
    /// @param to The address to transfer to.
    /// @param value The amount to
    function transfer(address to, uint256 value) external returns (bool);

    /// @dev Approve the passed address to spend the specified amount of tokens on behalf
    /// of msg.sender.
    /// Beware that changing an allowance with this method brings the risk that someone may
    /// use both the old
    /// and the new allowance by unfortunate transaction ordering. One possible solution to
    /// mitigate this race condition is to first reduce the spender's allowance to 0 and set
    /// the desired value afterwards:
    /// https://github.com/ethereum/EIPs/issues/20#issuecomment-263524729
    /// Selector: 095ea7b3
    /// @param spender The address which will spend the funds.
    /// @param value The amount of tokens to
    function approve(address spender, uint256 value)
        external returns (bool);

    /// @dev Transfer tokens from one address to another
    /// Selector: 23b872dd
    /// @param from address The address which you want to send tokens from
    /// @param to address The address which you want to transfer to
    /// @param value uint256 the amount of tokens to be transferred
    function transferFrom(address from, address to, uint256 value)
        external returns (bool);

    /// @dev Mint tokens to an address
    /// Selector: 40c10f19
    /// @param to address The address to which you want to mint tokens
    /// @param value uint256 the amount of tokens to be minted
    function mint(address to, uint256 value)
        external returns (bool);

    /// @dev Burn tokens from an address
    /// Selector: 9dc29fac
    /// @param from address The address from which you want to burn tokens
    /// @param value uint256 the amount of tokens to be burnt
    function burn(address from, uint256 value)
        external returns (bool);

    /// @dev Freeze an account, preventing it from operating with the asset
    /// Selector: 8d1fdf2f
    /// @param account address The address that you want to freeze
    function freeze(address account)
        external returns (bool);

    /// @dev Unfreeze an account, letting it from operating againt with the asset
    /// Selector: 5ea20216
    /// @param account address The address that you want to unfreeze
    function thaw(address account)
        external returns (bool);

    /// @dev Freeze the entire asset operations
    /// Selector: d4937f51
    function freezeAsset()
        external returns (bool);

    /// @dev Unfreeze the entire asset operations
    /// Selector: 51ec2ad7
    function thawAsset()
        external returns (bool);

    /// @dev Transfer the ownership of an asset to a new account
    /// Selector: f2fde38b
    /// @param owner address The address of the new owner
    function transferOwnership(address owner)
        external returns (bool);
    
    /// @dev Specify the issuer, admin and freezer of an asset
    /// Selector: c7d93c59
    /// @param issuer address The address capable of issuing tokens
    /// @param admin address The address capable of burning tokens and unfreezing accounts/assets
    /// @param freezer address The address capable of freezing accounts/asset
    function setTeam(address issuer, address admin, address freezer)
        external returns (bool);

    /// @dev Specify the name, symbol and decimals of your asset
    /// Selector: 4af77f93
    /// @param name string The name of the asset
    /// @param symbol string The symbol of the asset
    /// @param decimals uint8 The number of decimals of your asset
    function setMetadata(string calldata name, string calldata symbol, uint8 decimals)
        external returns (bool);

    /// @dev Clear the name, symbol and decimals of your asset
    /// Selector: efb6d432
    function clearMetadata()
        external returns (bool);
    /// @dev Event emited when a transfer has been performed.
    /// Selector: ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
    /// @param from address The address sending the tokens
    /// @param to address The address receiving the tokens.
    /// @param value uint256 The amount of
    event Transfer(
        address indexed from,
        address indexed to,
        uint256 value
    );

    /// @dev Event emited when an approval has been registered.
    /// Selector: 8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
    /// @param owner address Owner of the tokens.
    /// @param spender address Allowed spender.
    /// @param value uint256 Amount of
    event Approval(
        address indexed owner,
        address indexed spender,
        uint256 value
    );
}