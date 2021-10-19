import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { ethers } from "ethers";
import { createBlock } from "typescript";

import { ALITH, ALITH_PRIV_KEY } from "../util/constants";
import { getCompiled } from "../util/contracts";

import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { createContract } from "../util/transactions";

const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

// A call from root (sudo) can make a transfer directly in pallet_evm
// A signed call cannot make a transfer directly in pallet_evm

describeDevMoonbeam("Pallet EVM contract call - no sudo", (context) => {
  let events, contract;
  before("Send a simple transfer with pallet evm", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    const contractData = await getCompiled("TestContractIncr");
    const iFace = new ethers.utils.Interface(contractData.contract.abi);
    ({ contract } = await createContract(context.web3, "TestContractIncr"));

    const address = contract.options.address;

    const data = iFace.encodeFunctionData(
      // action
      "incr",
      []
    );
    ({ events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.evm.call(
        ALITH,
        address,
        data,
        0n,
        12_000_000n,
        1_000_000_000n,
        undefined
      )
    ));
    events.forEach((e) => {
      console.log(e.toHuman());
    });
  });

  it("should fail without sudo", async function () {
    expect(events[3].toHuman().method).to.eq("ExtrinsicFailed");
    expect(await contract.methods.count().call()).to.eq("0");

    // expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal("0");
  });
});
describeDevMoonbeam("Pallet EVM transfer - with sudo", (context) => {
  let events, contract;
  before("Send a simple transfer with pallet evm with sudo", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    const contractData = await getCompiled("TestContractIncr");
    // console.log("abi", contractData.contract.abi);
    // const iFace = new ethers.utils.Interface(contractData.contract.abi);
    ({ contract } = await createContract(context.web3, "TestContractIncr"));
    // console.log("abi2", JSON.stringify(contract.options.jsonInterface, null, 2));
    // console.log("abi3", contract.options.jsonInterface);
    const iFace = new ethers.utils.Interface(contract.options.jsonInterface);

    const address = contract.options.address;

    const data = iFace.encodeFunctionData(
      // action
      "incr",
      []
    );
    ({ events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.evm.call(
          ALITH,
          address,
          data,
          0n,
          12_000_000n,
          1_000_000_000n,
          undefined
        )
      )
    ));
    events.forEach((e) => {
      console.log(e.toHuman());
    });
    console.log(Object.keys(contract.methods));
    // await context.createBlock();
    // await context.createBlock();
    // await context.createBlock();
  });

  it.only("should succeed with sudo", async function () {
    console.log(await contract.methods.count().call({ from: ALITH }));
    // console.log(await contract.methods.count())
    expect(await contract.methods.count().call()).to.eq("1");
    expect(events[5].toHuman().method).to.eq("ExtrinsicSuccess");
  });
});
