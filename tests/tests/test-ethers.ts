import { createAndFinalizeBlock, describeWithMoonbeam } from "./util";
import { expect } from "chai";
import { ethers } from "ethers";
import { GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";
import { getCompiled } from "./util/contracts";

describeWithMoonbeam("Moonbeam RPC (Ethers.js)", `simple-specs.json`, (context) => {
  it("get network ids", async function () {
    expect((await context.ethers.getNetwork()).chainId).to.equal(1281);
    const providerTestnet = new ethers.providers.JsonRpcProvider(
      "https://rpc.testnet.moonbeam.network"
    );
    expect((await providerTestnet.getNetwork()).chainId).to.equal(1287);
  });

  it("deploy contract and interact with it", async function () {
    let signer = new ethers.Wallet(GENESIS_ACCOUNT_PRIVATE_KEY, context.ethers);

    // deploy contract
    const factory = new ethers.ContractFactory(
      (await getCompiled("TestContract")).contract.abi as ethers.ContractInterface,
      (await getCompiled("TestContract")).byteCode,
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
      (await getCompiled("TestContract")).contract.abi as ethers.ContractInterface,
      signer
    );
    expect((await contractFromAddress.multiply(3)).toString()).to.equal("21");
  });
});
