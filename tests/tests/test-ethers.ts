import { createAndFinalizeBlock, describeWithMoonbeam } from "./util";
import { HttpProvider } from "web3-core";
import { expect } from "chai";
import { ethers } from "ethers";
import { getCompiled } from "./util/contracts";

describeWithMoonbeam("Moonbeam RPC (Ethers.js)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

  it("get network id", async function () {
    expect((await context.ethers.getNetwork()).chainId).to.equal(1281);
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
