// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @author The Moonbeam Team
/// @title The interface through which solidity contracts will interact with the Proxy pallet
/// @custom:address 0x000000000000000000000000000000000000080b
interface Proxy {
    /// @dev Defines the proxy permission types that may be combined via `|` operator
    /// The values start at `0` and are represented as `uint32`
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

    /// @dev Dispatch the given call from an account that the sender is authorised for through
    /// addProxy. Removes any corresponding announcement(s)
    /// @custom:selector 93cb5160
    /// @param real the account that the proxy will make a call on behalf of
    /// @param call the call to be made by the real account
    function proxy(address real, bytes[] calldata call) external;

    /// @dev Dispatch the given call from an account that the sender is authorised for through
    /// addProxy. Removes any corresponding announcement(s)
    /// Exact proxy type is used and checked for this call
    /// @custom:selector aec65df0
    /// @param real the account that the proxy will make a call on behalf of
    /// @param forceProxyType the exact ProxyType to be used and checked for this call
    /// @param call the call to be made by the real account
    function proxyForceType(
        address real,
        ProxyType forceProxyType,
        bytes[] calldata call
    ) external;

    /// @dev Register a proxy account for the sender that is able to make calls on its behalf
    /// @custom:selector ac69400b
    /// @param delegate the account that the caller would like to make a proxy
    /// @param proxyType the permissions allowed for this proxy account
    /// @param delay the announcement period required of the initial proxy, will generally be zero
    function addProxy(
        address delegate,
        ProxyType proxyType,
        uint32 delay
    ) external;

    /// @dev Register a proxy account for the sender that is able to make calls on its behalf
    /// @custom:selector 78a804c5
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

    /// @dev Publish the hash of a proxy-call that will be made in the future
    /// @custom:selector 32cf4272
    /// @param real the account that the proxy will make a call on behalf of
    /// @param callHash the hash of the call to be made by the real account
    ///	transaction, will generally be zero
    function announce(address real, bytes32 callHash) external;

    /// @dev Remove a given announcement
    /// @custom:selector 4400aae3
    /// @param real the account that the proxy will make a call on behalf of
    /// @param callHash the hash of the call to be made by the real account
    function removeAnnouncement(address real, bytes32 callHash) external;

    /// @dev Remove the given announcement of a delegate
    /// @param delegate account that previously announced the call
    /// @param callHash the hash of the call to be made
    /// @custom:selector e508ff89
    function rejectAnnouncement(address delegate, bytes32 callHash) external;

    /// @dev Dispatch the given call from an account that the sender is authorised for through
    /// addProxy. Removes any corresponding announcement(s)
    /// @custom:selector 8a53f3f5
    /// @param delegate the account that previously announced the call
    /// @param real the account that the proxy will make a call on behalf of
    /// @param call the call to be made by the real account
    function proxyAnnounced(
        address delegate,
        address real,
        bytes[] calldata call
    ) external;

    /// @dev Dispatch the given call from an account that the sender is authorised for through
    /// addProxy. Removes any corresponding announcement(s)
    /// Exact proxy type is used and checked for this call
    /// @custom:selector af97d7af
    /// @param delegate the account that previously announced the call
    /// @param real the account that the proxy will make a call on behalf of
    /// @param forceProxyType the exact ProxyType to be used and checked for this call
    /// @param call the call to be made by the real account
    function proxyForceTypeAnnounced(
        address delegate,
        address real,
        ProxyType forceProxyType,
        bytes[] calldata call
    ) external;
}
