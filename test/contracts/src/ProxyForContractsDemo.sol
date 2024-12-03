// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

interface IProxy{
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
    function addProxy(
        address delegate,
        ProxyType proxyType,
        uint32 delay
    ) external;
    function isProxy(
        address real,
        address delegate,
        ProxyType proxyType,
        uint32 delay
    ) external view returns (bool exists);
    function proxy(
        address real,
        address callTo,
        bytes memory callData
    ) external payable;
}

// This contract adds Alith (0xf24....) as proxy to itself when it gets deployed.
// The purpose is to trigger a proxy creation BEFORE the contract code is stored.
// It bypasses the restriction of "no proxy for smart contract" but cannot be exploited
// because runtimes block the execution of a proxy if the proxied account contains code
// see https://github.com/moonbeam-foundation/moonbeam/pull/2533/files#diff-95857a497fb7c3739b385c704d94c0f41293c05703b539cb139093f78e0a4cfbR1122
contract ProxyForContractsDemo {

    address immutable PROXY_ADDRESS = 0x000000000000000000000000000000000000080b;
    // for debugging purpose
    //
    constructor() payable{
        // payable because you need some funds to be resereved
        // add Alice as delegate for this newly created contract
        PROXY_ADDRESS.call(abi.encodeWithSelector(IProxy.addProxy.selector, 0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac, IProxy.ProxyType.Any, 0));
    }

    // shortcut function to check if Alice is a delegate of this contract
    function isYouMyProxy() external view returns(bool){
        return IProxy(PROXY_ADDRESS).isProxy(address(this), 0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac, IProxy.ProxyType.Any, 0);
    }

    fallback() external payable{}
    receive() external payable{}

}