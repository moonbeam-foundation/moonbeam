// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @author The Moonbeam Team
 * @title The interface through which solidity contracts will interact with the Proxy pallet
 * We follow this same interface including four-byte function selectors, in the precompile that
 * wraps the pallet
 * Address :    0x000000000000000000000000000000000000080a
 */

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
    /// @param real the account that the proxy will make a call on behalf of
    /// @param forceProxyType the exact proxy type to be used and checked for this call, encoded 
    /// as bytes[] where first byte is either 0x00 for not using the parameter, or `0x01` followed
    /// by big-endian encoded `uint32` ProxyType
    /// @param call the call to be made by the real account
    /// @return A boolean confirming whether the address is a nominator
    /// Selector: a
    function proxy(
        address real, 
        bytes[] forceProxyType,
        bytes[] call
    ) external;

    /// @dev Register a proxy account for the sender that is able to make calls on its behalf
    /// @param delegate the account that the caller would like to make a proxy
    /// @param proxyType the permissions allowed for this proxy account
    /// @param delay the announcement period required of the initial proxy, will generally be zero
    /// Selector: a
    function addProxy(
        address delegate,
        uint32 proxyType,
        uint32 delay
    ) external;

    /// @dev Register a proxy account for the sender that is able to make calls on its behalf
    /// @param delegate The account that the caller would like to remove as a proxy
    /// @param proxyType The permissions currently enabled for the removed proxy account
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    /// Selector: a
    function removeProxy(
        address delegate,
        uint32 proxyType,
        uint32 delay
    ) external;

    /// @dev Unregister all proxy accounts for the sender
    /// Selector: a
    function removeProxies() external;

    /// @dev Spawn a fresh new account that is guaranteed to be otherwise inaccessible
    /// @param proxyType The type of the proxy that the sender will be registered as
    /// @param index A disambiguation index, in case this is called multiple times in the same
	///	transaction, will generally be zero
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    /// Selector: a
    function anonymous(
        uint32 proxyType,
        uint16 index,
        uint32 delay
    ) external;

    /// @dev Spawn a fresh new account that is guaranteed to be otherwise inaccessible
    /// @param proxyType The type of the proxy that the sender will be registered as
    /// @param index A disambiguation index, in case this is called multiple times in the same
	///	transaction, will generally be zero
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    /// Selector: a
    function killAnonymous(
        address spwaner,
        uint32 proxyType,
        uint16 index,
        uint32 height,
        uint32 extIndex
    ) external;

    /// @dev Spawn a fresh new account that is guaranteed to be otherwise inaccessible
    /// @param proxyType The type of the proxy that the sender will be registered as
    /// @param index A disambiguation index, in case this is called multiple times in the same
	///	transaction, will generally be zero
    /// @param delay The announcement period required of the initial proxy, will generally be zero
    /// Selector: a
    function announce(
        address spwaner,
        uint32 proxyType,
        uint16 index,
        uint32 height,
        uint32 extIndex
    ) external;

    /// @dev Remove a given announcement
    /// @param real the account that the proxy will make a call on behalf of
    /// @param callHash the hash of the call to be made by the real account
    /// Selector: a
    function removeAnnouncement(
        address real,
        bytes[] callHash,
    ) external;

    /// @dev Remove the given announcement of a delegate
    /// @param delegate account that previously announced the call
    /// @param callHash the hash of the call to be made
    /// Selector: a
    function rejectAnnouncement(
        address delegate,
        bytes[] callHash,
    ) external;

    /// @dev Dispatch the given call from an account that the sender is authorised for through
    /// addProxy. Removes any corresponding announcement(s)
    /// @param delegate the account that previously announced the call
    /// @param real the account that the proxy will make a call on behalf of
    /// @param forceProxyType the exact proxy type to be used and checked for this call, encoded 
    /// as bytes[] where first byte is either 0x00 for not using the parameter, or `0x01` followed
    /// by big-endian encoded `uint32` ProxyType
    /// @param call the call to be made by the real account
    /// @return A boolean confirming whether the address is a nominator
    /// Selector: a
    function proxyAnnounced(
        address delegate, 
        address real, 
        bytes[] forceProxyType,
        bytes[] call
    ) external;
}
