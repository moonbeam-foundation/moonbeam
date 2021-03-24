import { createAndFinalizeBlock, describeWithMoonbeam } from "./util";
import { HttpProvider } from "web3-core";
import { expect } from "chai";
import { ethers } from "ethers";
import { TEST_CONTRACT_ABI, TEST_CONTRACT_BYTECODE } from "./constants";

describeWithMoonbeam("Moonbeam RPC (Ethers.js)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

  it("get network ids", async function () {
    // Providers
    let prov = context.web3.currentProvider as HttpProvider;
    const provider = new ethers.providers.JsonRpcProvider(prov.host);
    expect((await provider.getNetwork()).chainId).to.equal(1281);
    const providerTestnet = new ethers.providers.JsonRpcProvider(
      "https://rpc.testnet.moonbeam.network"
    );
    expect((await providerTestnet.getNetwork()).chainId).to.equal(1287);
  });
  it("deploy contract and interact with it", async function () {
    // Providers
    let prov = context.web3.currentProvider as HttpProvider;
    const provider = new ethers.providers.JsonRpcProvider(prov.host);

    let signer = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, provider);

    // deploy contract
    const factory = new ethers.ContractFactory(
      [TEST_CONTRACT_ABI] as ethers.ContractInterface,
      TEST_CONTRACT_BYTECODE,
      signer
    );
    let contract = await new Promise<ethers.Contract>(async (resolve) => {
      const contract = factory.deploy();
      await createAndFinalizeBlock(context.polkadotApi);
      resolve(await contract);
    });
    expect(contract.address);

    // call method
    let res = await new Promise<string>(async (resolve) => {
      const re = contract.multiply(3);
      await createAndFinalizeBlock(context.polkadotApi);
      resolve(await re);
    });
    expect(res.toString()).to.equal("21");

    // Instantiate contract from address
    const contractFromAddress = new ethers.Contract(
      contract.address,
      [TEST_CONTRACT_ABI] as ethers.ContractInterface,
      signer
    );
    expect((await contractFromAddress.multiply(3)).toString()).to.equal("21");
  });
});
