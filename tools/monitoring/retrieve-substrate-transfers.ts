import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, WsProvider } from "@polkadot/api";

const main = async () => {
  const wsProvider = new WsProvider("wss://moonriver.api.onfinality.io/public-ws");
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });

  const startBlockNumber = 400458n; // transfer-enabled runtime upgrade block
  const endBlockNumber = (await polkadotApi.rpc.chain.getBlock()).block.header.number.toBigInt(); // or 414465

  for (
    let currentBlockNumber = startBlockNumber;
    currentBlockNumber < endBlockNumber;
    currentBlockNumber++
  ) {
    let blockHash = await polkadotApi.rpc.chain.getBlockHash(currentBlockNumber);
    let { block } = await polkadotApi.rpc.chain.getBlock(blockHash);

    const records = await polkadotApi.query.system.events.at(block.header.hash);

    block.extrinsics.forEach((extrinsic, index) => {
      const {
        method: { args, method, section },
      } = extrinsic;

      // We only want Substrate extrinsics
      if (section == "ethereum" && method == "transact") {
        return;
      }

      // Retrieve all events for this extrinsic
      const events = records.filter(
        ({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index)
      );

      // Search if it is a transfer
      events.forEach(({ event }) => {
        if (event.section == "balances" && event.method == "Transfer") {
          const from = event.data[0].toString();
          const to = event.data[1].toString();
          const balance = (event.data[2] as any).toBigInt();

          const substrateHash = extrinsic.hash.toString();

          console.log(
            `#${block.header.number}: Substrate Transfer from ${from} to ${to} of ${balance} (${substrateHash})`
          );
        }
      });
    });
  }
};

main();
