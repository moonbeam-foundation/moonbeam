import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { TREASURY_ACCOUNT } from "../../util/constants";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";
import { baltathar } from "../../util/accounts";

describeDevMoonbeamAllEthTxTypes("Fees - Transaction", (context) => {
  it("should send 20% of the fees to treasury", async () => {
    // Treasury account should be initially empty
    expect(await context.web3.eth.getBalance(TREASURY_ACCOUNT, 0)).to.equal(0n.toString());

    // We make an ethereum transaction, 20% of the fees should go to treasury.
    await context.createBlock(createTransfer(context, baltathar.address, 128));
    expect(await context.web3.eth.getBalance(TREASURY_ACCOUNT, 1)).to.equal("4200000000000");
  });
});

describeDevMoonbeamAllEthTxTypes("Fees - Transaction", (context) => {
  it("should burn 80% of the fees", async () => {
    const originalTotalIssuance = await (
      await context.polkadotApi.query.balances.totalIssuance()
    ).toBigInt();

    // We make an ethereum transaction, 20% of the fees should go to treasury.
    await context.createBlock(createTransfer(context, baltathar.address, 128));
    expect(await (await context.polkadotApi.query.balances.totalIssuance()).toBigInt()).to.equal(
      originalTotalIssuance - 16800000000000n
    );
  });
});
