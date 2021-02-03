//import ethers from "ethers";
import { describeWithMoonbeam } from "./util";
import { HttpProvider } from "web3-core";
const ethers = require("ethers");

describeWithMoonbeam("Moonbeam RPC (Ethers.js)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

  it.only("get nonce", async function () {
    // Providers
    //console.log("prov", context.web3.currentProvider);
    let prov = context.web3.currentProvider as HttpProvider;
    const provider = new ethers.providers.JsonRpcProvider(prov.host); //("https://rpc.testnet.moonbeam.network");
    console.log(await provider.getNetwork());
    const providertestnet = new ethers.providers.JsonRpcProvider(
      "https://rpc.testnet.moonbeam.network"
    );
    console.log(await providertestnet.getNetwork());
    const providerGanache = new ethers.providers.JsonRpcProvider(
      "https://mainnet.infura.io/v3/c14b133a94f541c580f37fe718ec4fa9"
    );
    console.log(await providerGanache.getNetwork());

    // const logNet = async () => {
    //   console.log(await provider.getNetwork());
    // };

    // await logNet();
  });
});
