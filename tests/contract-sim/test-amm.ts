import "@moonbeam-network/api-augment";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { prodParasPolkadot } from "@polkadot/apps-config/endpoints";
import { ITuple } from "@polkadot/types/types";

import { expect } from "chai";
import { alith } from "../util/accounts";

import { verifyLatestBlockFees } from "../util/block";
import { getCompiled } from "../util/contracts";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";

describeDevMoonbeamAllEthTxTypes("Simulating an AMM", (context) => {
  before(async function () {
    // Read AccountCode at address
    const providers = Object.values(prodParasPolkadot.find(a=>a.paraId === 2004).providers)
    const api = await ApiPromise.create({provider: new WsProvider(providers)})

    const address= "0x68A384D826D3678f78BB9FB1533c7E9577dACc0E"
    const accCode = await api.query.evm.accountCodes(address)
    const accStorage = await api.query.evm.accountStorages.entries(address)
    // the keys
    console.log("they keyss")
    const key = accStorage[0][0].toString()
    const value = accStorage[0][1].toString()
    // console.log(accStorage[0])
    console.log(key)


    //the storage
    console.log("they storage")
    console.log(accStorage[0][1].toHuman())
    console.log(value)
    // Read AccountCode at address
    /// SetStorage for router
    const tuple: [string, string] = [key, value]
    
    const tx = context.polkadotApi.tx.system.setStorage([tuple]).sign(alith,{nonce:-1})
    context.polkadotApi.tx.sudo(tx)
    /// settorages for all of contract storages
    /// SetStorage for weth
    /// settorages for all of contract storages
    /// SetStorage for factory
    /// settorages for all of contract storages
    /// SetStorage for factory
    /// settorages for all of contract storages

    api.disconnect()
  });

  after(async function(){

  })
  it("should return the transaction hash", async () => {
    const { rawTx } = await createContract(context, "MultiplyBy7");
    const { result } = await context.createBlock(rawTx);

    expect(result.hash, "0x286fc7f456a452abb22bc37974fe281164e53ce6381583c8febaa89c92f31c0b");
  });
});
