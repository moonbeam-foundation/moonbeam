import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  TEST_ACCOUNT,
} from "../../util/constants";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { blake2AsHex } from "@polkadot/util-crypto";

describeDevMoonbeam("Reducible Balance", (context) => {
  it("should show the reducible balanced when some amount is locked", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");

    // Balance should be untouched
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(
      GENESIS_ACCOUNT_BALANCE.toString()
    );

    // Grab existential deposit
    let existentialDeposit = (await context.polkadotApi.consts.balances.existentialDeposit) as any;

    // Let's lock some funds by doing a public referendum proposal
    let lock_amount = (await context.polkadotApi.consts.democracy.minimumDeposit) as any;
    const proposal = context.polkadotApi.tx.balances.setBalance(TEST_ACCOUNT, 100, 100);

    // We encode the proposal
    let encodedProposal = (proposal as SubmittableExtrinsic)?.method.toHex() || "";
    let encodedHash = blake2AsHex(encodedProposal);

    // Submit the pre-image
    await context.polkadotApi.tx.democracy
      .notePreimage(encodedProposal)
      .signAndSend(genesisAccount);

    await context.createBlock();

    // Record balance
    let beforeBalance = await context.web3.eth.getBalance(GENESIS_ACCOUNT);

    // Fees
    const fee = (
      await context.polkadotApi.tx.democracy
        .propose(encodedHash, lock_amount)
        .paymentInfo(genesisAccount)
    ).partialFee as any;

    // Propose
    await context.polkadotApi.tx.democracy
      .propose(encodedHash, lock_amount)
      .signAndSend(genesisAccount);

    await context.createBlock();

    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.equal(
      (
        BigInt(beforeBalance) -
        BigInt(lock_amount) +
        BigInt(existentialDeposit) -
        BigInt(fee)
      ).toString()
    );
  });
});
