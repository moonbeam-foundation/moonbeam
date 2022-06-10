import "@moonbeam-network/api-augment";
import { expect } from "chai";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { blake2AsHex } from "@polkadot/util-crypto";
import { alith, ALITH_GENESIS_BALANCE, generateKeyingPair } from "../../util/accounts";

describeDevMoonbeam("Reducible Balance", (context) => {
  const randomAccount = generateKeyingPair();
  it("should show the reducible balanced when some amount is locked", async function () {
    // Balance should be untouched
    expect(await context.web3.eth.getBalance(alith.address)).to.equal(
      ALITH_GENESIS_BALANCE.toString()
    );

    // Grab existential deposit
    let existentialDeposit = (await context.polkadotApi.consts.balances.existentialDeposit) as any;

    // Let's lock some funds by doing a public referendum proposal
    let lock_amount = (await context.polkadotApi.consts.democracy.minimumDeposit) as any;
    const proposal = context.polkadotApi.tx.balances.setBalance(randomAccount.address, 100, 100);

    // We encode the proposal
    let encodedProposal = (proposal as SubmittableExtrinsic)?.method.toHex() || "";
    let encodedHash = blake2AsHex(encodedProposal);

    // Submit the pre-image
    await context.createBlock(context.polkadotApi.tx.democracy.notePreimage(encodedProposal));

    // Record balance
    let beforeBalance = await context.web3.eth.getBalance(alith.address);

    // Fees
    const fee = (
      await context.polkadotApi.tx.democracy.propose(encodedHash, lock_amount).paymentInfo(alith)
    ).partialFee as any;

    // Propose
    await context.createBlock(context.polkadotApi.tx.democracy.propose(encodedHash, lock_amount));

    expect(await context.web3.eth.getBalance(alith.address)).to.equal(
      (
        BigInt(beforeBalance) -
        BigInt(lock_amount) +
        BigInt(existentialDeposit) -
        BigInt(fee)
      ).toString()
    );
  });
});
