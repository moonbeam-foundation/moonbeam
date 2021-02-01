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
          expect(message.substring(0, 34)).to.eq(`parachainUpgrade.setValidationData`);
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
