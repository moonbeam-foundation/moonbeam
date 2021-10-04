import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { customRequest, importAccount, init } from "../init-web3";
import { customWeb3Request } from "../utils/transactions";

const addressFilters = [
  "0xE3C7487Eb01C74b73B7184D198c7fBF46b34E5AF",
  "0x802B0B134B76765378e10F1Ef5175349751af90a",
  "0xD184B1317125b166f01e8a0d6088ce1de61D00BA",
  "0xa9177F8d98DAaB74C24715Ba0A81b73654710523",
  "0xb64Ec03ABAC06FcFf7D5FBC2b4d36Ce5f8f46F44",
].map((t) => t.toLocaleLowerCase());

const web3 = init("http://localhost:56991");

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

    block.extrinsics.forEach(async (extrinsic, index) => {
      const {
        method: { args, method, section },
      } = extrinsic;

      const isEthereum = section == "ethereum" && method == "transact";

      // Transfer do not include input data
      const isSeaDexTransfer =
        isEthereum &&
        (args[0] as any).action.isCall &&
        addressFilters.includes((args[0] as any).action.asCall.toString());

      if (!isSeaDexTransfer) {
        return;
      }
      const events = records.filter(
        ({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index)
      );

      let from = "";
      // Search for ethereum execution
      events.forEach(({ event }) => {
        if (event.section == "ethereum" && event.method == "Executed") {
          from = event.data[0].toString();
        }
      });

      const ethTx = args[0] as any;
      console.log(
        `SeaDex call to ${ethTx.action.asCall.toString()} [${ethTx.gasLimit}]: ${ethTx.input}`
      );

      console.log({
        from,
        to: ethTx.action.asCall.toString(),
        gasPrice: `0x${Number(ethTx.gasPrice?.toString()).toString(16)}`,
        gas: `0x${Number(ethTx.gasLimit?.toString()).toString(16)}`,
        value: `0x${Number(ethTx.value?.toString()).toString(16)}`,
        data: ethTx.input?.toString(),
        nonce: `0x${(Number(ethTx.nonce) + 1).toString(16)}`,
      });

      const estimate = await customWeb3Request(web3, "eth_estimateGas", [
        {
          from,
          to: ethTx.action.asCall.toString(),
          gasPrice: `0x${Number(ethTx.gasPrice?.toString()).toString(16)}`,
          gas: `0x${Number(ethTx.gasLimit?.toString()).toString(16)}`,
          value: `0x${Number(ethTx.value?.toString()).toString(16)}`,
          data: ethTx.input?.toString(),
          nonce: `0x${(Number(ethTx.nonce) + 1).toString(16)}`,
        },
      ]);
      console.log(
        estimate.result ? BigInt(estimate.result).toString() : (estimate?.error as any)?.message
      );
    });
  });
};

main();
