import { ApiPromise, WsProvider } from "@polkadot/api";
import Web3 from "web3";
import { typesBundle } from "../moonbeam-types-bundle";
import { FAITH } from "./test-constants";
const wsProviderUrl = `wss://wss.testnet.moonbeam.network`;

export default async function test(ACC: string) {
  const web3 = new Web3(wsProviderUrl);
  let balance = await web3.eth.getBalance(ACC);
  console.log("BALANCE WEB3", balance.toString());

  const wsProvider = new WsProvider(wsProviderUrl);
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });
  const account = await polkadotApi.query.system.account(ACC);
  // console.log("BALANCE API", account.data.feeFrozen.toString());
  // console.log("BALANCE API", account.data.miscFrozen.toString());
  // console.log("BALANCE API", account.data.reserved.toString());
  console.log("BALANCE API", account.data.free.toString());
}
test(FAITH);
