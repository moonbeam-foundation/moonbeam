import "@moonbeam-network/api-augment"
import { ApiPromise, WsProvider } from "@polkadot/api";
import { typesBundlePre900 } from "moonbeam-types-bundle";

const api = await ApiPromise.create({
  provider: new WsProvider("wss://wss.api.moonbase.moonbeam.network"),
  typesBundle: typesBundlePre900,
});

console.log(api.consts.system.version.specName.toString());

console.log(api.consts.system.version.specVersion.toNumber());

const headBlock = (await api.rpc.chain.getHeader()).hash.toHex();

console.log((await api.rpc.moon.isBlockFinalized(headBlock)).toHuman());

console.log((await api.query.system.events()).toHuman())

console.log(
  (await api.query.parachainStaking.round()
  ).toHuman())



await api.disconnect();
