import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, WsProvider } from "@polkadot/api";

const main = async () => {
  const wsProvider = new WsProvider("wss://wss.moonriver.moonbeam.network");
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });

  await polkadotApi.rpc.chain.subscribeNewHeads(async (lastHeader) => {
    const [{ block }, records] = await Promise.all([
      polkadotApi.rpc.chain.getBlock(lastHeader.hash),
      polkadotApi.query.system.events.at(lastHeader.hash),
    ]);

    block.extrinsics.forEach((extrinsic, index) => {
      const {
        method: { args, method, section },
      } = extrinsic;

      const isEthereum = "ethereum" && method == "transact";

      // Transfer do not include input data
      const isEthereumTransfer =
        isEthereum && (args[0] as any).input.length === 0 && (args[0] as any).action.isCall;

      // Retrieve all events for this extrinsic
      const events = records.filter(
        ({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index)
      );

      // This hash will only exist if the transaction was executed through ethereum.
      let ethereumHash = "";

      if (isEthereum) {
        // Search for ethereum execution
        events.forEach(({ event }) => {
          if (event.section == "ethereum" && event.method == "Executed") {
            ethereumHash = event.data[2].toString();
          }
        });
      }

      // Search if it is a transfer
      events.forEach(({ event }) => {
        if (event.section == "balances" && event.method == "Transfer") {
          const from = event.data[0].toString();
          const to = event.data[1].toString();
          const balance = (event.data[2] as any).toBigInt();

          const substrateHash = extrinsic.hash.toString();

          console.log(`Transfer from ${from} to ${to} of ${balance} (block #${lastHeader.number})`);
          console.log(`  - Triggered by extrinsic: ${substrateHash}]`);
          if (isEthereum) {
            console.log(`  - Ethereum (isTransfer: ${isEthereumTransfer}) hash: ${ethereumHash}]`);
          }
        }
      });
    });
  });
};

main();
