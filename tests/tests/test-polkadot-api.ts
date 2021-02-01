import { expect } from "chai";
import { Keyring } from "@polkadot/keyring";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam } from "./util";

describeWithMoonbeam("Moonbeam Polkadot API", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

  step("api can retrieve last header", async function () {
    const lastHeader = await context.polkadotApi.rpc.chain.getHeader();
    expect(Number(lastHeader.number) >= 0).to.be.true;
  });

  step("api can retrieve last block", async function () {
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    expect(signedBlock.block.header.number.toNumber() >= 0).to.be.true;
  });

  const TEST_ACCOUNT_2 = "0x1111111111111111111111111111111111111112";

  step("transfer from polkadotjs should appear in ethereum", async function () {
    this.timeout(30000);

    const keyring = new Keyring({ type: "ethereum" });
    const testAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    try {
      let hash = await context.polkadotApi.tx.balances
        .transfer(TEST_ACCOUNT_2, 123)
        .signAndSend(testAccount);
    } catch (e) {
      expect(false, "error during polkadot api transfer" + e);
    }
    // TODO: do some testing with the hash
    await createAndFinalizeBlock(context.polkadotApi);
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT_2)).to.equal("123");
  });

  step("read extrinsic information", async function () {
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    expect(signedBlock.block.header.number.toNumber() >= 0).to.be.true;

    // Expecting 4 extrinsics so far:
    // timestamp, author, the parachain validation data and the balances transfer.
    expect(signedBlock.block.extrinsics).to.be.of.length(4);

    signedBlock.block.extrinsics.forEach((ex, index) => {
      const {
        isSigned,
        method: { args, method, section },
      } = ex;
      const message = `${section}.${method}(${args.map((a) => a.toString()).join(", ")})`;
      switch (index) {
        case 0:
          expect(message).to.eq(`timestamp.set(6000)`);
          break;
        case 1:
          expect(message).to.eq(
            `parachainUpgrade.setValidationData({"validationData":{"persisted":{"parentHead":"0x","blockNumber":0,"relayStorageRoot":"0xd806e3afb4d36275caf5ce33158f64e860d90e47f39207caf581a54648b8828d","hrmpMqcHeads":[],"dmqMqcHead":"0x0000000000000000000000000000000000000000000000000000000000000000","maxPovSize":0},"transient":{"maxCodeSize":0,"maxHeadDataSize":0,"balance":0,"codeUpgradeAllowed":null,"dmqLength":0}},"relayChainState":["0x7f0106de3d8a54d27e44a9d5ce189618f22db4b49d95320d9021994c850f25b8e38590000020000000100008000000000400000001000005000000050000000600000006000000"]})`
          );
          break;
        case 2:
          expect(message).to.eq(
            `authorInherent.setAuthor(0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b)`
          );
          break;
        case 3:
          expect(ex.signer.toString().toLocaleLowerCase()).to.eq(GENESIS_ACCOUNT);
          expect(message).to.eq(
            `balances.transfer(0x1111111111111111111111111111111111111112, 123)`
          );
          break;
        default:
          throw new Error(`Unexpected extrinsic: ${message}`);
      }
    });
  });
});
