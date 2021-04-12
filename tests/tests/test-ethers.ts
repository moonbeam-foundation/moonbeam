import { createAndFinalizeBlock, describeWithMoonbeam } from "./util";
import { HttpProvider } from "web3-core";
import { expect } from "chai";
import { ethers } from "ethers";
import {
  TEST_CONTRACT_ABI_ETHERS,
  TEST_CONTRACT_BYTECODE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
} from "./constants";

describeWithMoonbeam("Moonbeam RPC (Ethers.js)", `simple-specs.json`, (context) => {
  let provider;
  before(() => {
    // Providers
    let prov = context.web3.currentProvider as HttpProvider;
    provider = new ethers.providers.JsonRpcProvider(prov.host);
  });

  it("get network ids", async function () {
    expect((await provider.getNetwork()).chainId).to.equal(1281);
    const providerTestnet = new ethers.providers.JsonRpcProvider(
      "https://rpc.testnet.moonbeam.network"
    );
    expect((await providerTestnet.getNetwork()).chainId).to.equal(1287);
  });
  it("deploy contract and interact with it", async function () {
    this.timeout(15000);
    let signer = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, provider);

    // deploy contract
    const factory = new ethers.ContractFactory(
      [TEST_CONTRACT_ABI_ETHERS] as ethers.ContractInterface,
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
      [TEST_CONTRACT_ABI_ETHERS] as ethers.ContractInterface,
      signer
    );
    expect((await contractFromAddress.multiply(3)).toString()).to.equal("21");
  });
});
