import "@moonbeam-network/api-augment/moonbase";
import { ApiPromise, WsProvider } from "@polkadot/api";

const main = async () => {
  const api = await ApiPromise.create({
    initWasm: false,
    provider: new WsProvider(`ws://localhost:9944`),
  });

  const round = await api.query.parachainStaking.round();
  console.log(round.current.toNumber());
};
