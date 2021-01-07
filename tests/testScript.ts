import { ApiPromise, WsProvider } from "@polkadot/api";
async function main() {
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    types: polkadotJsTypes,
    rpc: polkadotJsRpc,
  });
  const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
  console.log("signedBlock", signedBlock.block.header.number.toString());
}
main();
