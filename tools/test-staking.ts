import { ApiPromise, WsProvider } from "@polkadot/api";
import { start } from "polkadot-launch";
import { typesBundle } from "../moonbeam-types-bundle";

async function test() {
  //await start("config_moonbeam.json");
  console.log("done");
  const WS_PORT = 36946;
  const wsProviderUrl = `ws://localhost:${WS_PORT}`;

  const wsProvider = new WsProvider(wsProviderUrl);
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });
  //const account = await polkadotApi.query.system.account(ACC);
  const nominators = await polkadotApi.query.stake.validators();
  console.log(nominators, nominators.toHuman(), nominators.toJSON());
}
test();
