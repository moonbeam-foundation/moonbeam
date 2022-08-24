// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.3;

/// @author The Moonbeam Team
/// @title Pallet Proxy Interface
/// @title The interface through which solidity contracts will interact with the Proxy pallet
/// @custom:address 0x000000000000000000000000000000000000080b
interface Proxy {
    /// @dev Defines the proxy permission types.
    /// The values start at `0` (most permissive) and are represented as `uint8`
    enum ProxyType {
        Any,
        NonTransfer,
        Governance,
        Staking,
        CancelProxy,
        Balances,
        AuthorMapping,
        IdentityJudgement
    }

    /// @dev Register a proxy account for the sender that is able to make calls on its behalf
    /// @custom:selector 74a34dd3
    /// @param delegate The account that the caller would like to make a proxy
    /// @param proxyType The permissions allowed for this proxy account
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    function addProxy(
        address delegate,
        ProxyType proxyType,
        uint32 delay
    ) external;

    /// @dev Register a proxy account for the sender that is able to make calls on its behalf
    /// @custom:selector fef3f708
    /// @param delegate The account that the caller would like to remove as a proxy
    /// @param proxyType The permissions currently enabled for the removed proxy account
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    function removeProxy(
        address delegate,
        ProxyType proxyType,
        uint32 delay
    ) external;

    /// @dev Unregister all proxy accounts for the sender
    /// @custom:selector 14a5b5fa
    function removeProxies() external;

    /// @dev Checks if the caller has an account proxied with a given proxy type
    /// @custom:selector e26d38ed
    /// @param real The real account that maybe has a proxy
    /// @param delegate The account that the caller has maybe proxied
    /// @param proxyType The permissions allowed for the proxy
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    /// @return exists True if a proxy exists, False otherwise
    function isProxy(
        address real,
        address delegate,
        ProxyType proxyType,
        uint32 delay
    ) external view returns (bool exists);
}
