// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/// @author The Moonbeam Team
/// @title The interface through which solidity contracts will interact with the Proxy pallet
/// @custom:address 0x000000000000000000000000000000000000080a
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
    /// @custom:selector aa11bbcc
    /// @param real the account that the proxy will make a call on behalf of
    /// @param call the call to be made by the real account
    function proxy(address real, bytes[] calldata call) external;

    /// @dev Dispatch the given call from an account that the sender is authorised for through
    /// addProxy. Removes any corresponding announcement(s)
    /// Exact proxy type is used and checked for this call
    /// @custom:selector aa11bbcc
    /// @param real the account that the proxy will make a call on behalf of
    /// @param forceProxyType the exact ProxyType to be used and checked for this call
    /// @param call the call to be made by the real account
    function proxyForceType(
        address real,
        uint32 forceProxyType,
        bytes[] calldata call
    ) external;

    /// @dev Register a proxy account for the sender that is able to make calls on its behalf
    /// @custom:selector aa11bbcc
    /// @param delegate the account that the caller would like to make a proxy
    /// @param proxyType the permissions allowed for this proxy account
    /// @param delay the announcement period required of the initial proxy, will generally be zero
    function addProxy(
        address delegate,
        uint32 proxyType,
        uint32 delay
    ) external;

    /// @dev Register a proxy account for the sender that is able to make calls on its behalf
    /// @custom:selector aa11bbcc
    /// @param delegate The account that the caller would like to remove as a proxy
    /// @param proxyType The permissions currently enabled for the removed proxy account
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    function removeProxy(
        address delegate,
        uint32 proxyType,
        uint32 delay
    ) external;

    /// @dev Unregister all proxy accounts for the sender
    /// @custom:selector aa11bbcc
    function removeProxies() external;

    /// @dev Spawn a fresh new account that is guaranteed to be otherwise inaccessible
    /// @custom:selector aa11bbcc
    /// @param proxyType The type of the proxy that the sender will be registered as
    /// @param index A disambiguation index, in case this is called multiple times in the same
    ///	transaction, will generally be zero
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    function createAnonymous(
        uint32 proxyType,
        uint16 index,
        uint32 delay
    ) external;

    /// @dev Removes a previously spawned anonymous proxy
    /// @custom:selector aa11bbcc
    /// @param spawner the account that originally called anonymous to create this account
    /// @param proxyType the proxy type originally passed to anonymous
    /// @param index the disambiguation index originally passed to anonymous. Probably 0
    /// @param height the height of the chain when the call to anonymous was processed
    /// @param extIndex the extrinsic index in which the call to anonymous was processed
    function killAnonymous(
        address spawner,
        uint32 proxyType,
        uint16 index,
        uint32 height,
        uint32 extIndex
    ) external;

    /// @dev Publish the hash of a proxy-call that will be made in the future
    /// @param real the account that the proxy will make a call on behalf of
    /// @param callHash the hash of the call to be made by the real account
    ///	transaction, will generally be zero
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    /// @custom:selector aa11bbcc
    function announce(address real, bytes[] calldata callHash) external;

    /// @dev Remove a given announcement
    /// @custom:selector aa11bbcc
    /// @param real the account that the proxy will make a call on behalf of
    /// @param callHash the hash of the call to be made by the real account
    function removeAnnouncement(address real, bytes[] calldata callHash)
        external;

    /// @dev Remove the given announcement of a delegate
    /// @param delegate account that previously announced the call
    /// @param callHash the hash of the call to be made
    /// @custom:selector aa11bbcc
    function rejectAnnouncement(address delegate, bytes[] calldata callHash)
        external;

    /// @dev Dispatch the given call from an account that the sender is authorised for through
    /// addProxy. Removes any corresponding announcement(s)
    /// @custom:selector aa11bbcc
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
    /// @custom:selector aa11bbcc
    /// @param delegate the account that previously announced the call
    /// @param real the account that the proxy will make a call on behalf of
    /// @param forceProxyType the exact ProxyType to be used and checked for this call
    /// @param call the call to be made by the real account
    function proxyForceTypeAnnounced(
        address delegate,
        address real,
        uint32 forceProxyType,
        bytes[] calldata call
    ) external;
}
