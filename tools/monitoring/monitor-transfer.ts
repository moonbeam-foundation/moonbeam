import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, WsProvider } from "@polkadot/api";

const main = async () => {
  const wsProvider = new WsProvider("wss://wss.moonriver.moonbeam.network");
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });
  polkadotApi.query.system.events((events: any) => {
    // Loop through the Vec<EventRecord>
    events.forEach((record) => {
      console.log(record.phase.toString());
      if (record.event.section == "balances" && record.event.method == "Transfer") {
        const from = record.event.data[0].toString();
        const to = record.event.data[1].toString();
        const balance = record.event.data[2].toBigInt();

        console.log(`Transfer from ${from} to ${to} of ${balance}`);
      }
    });
  });
};

main();
