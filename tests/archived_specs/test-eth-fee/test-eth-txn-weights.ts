import "@moonbeam-network/api-augment";

import { expect } from "chai";

import {
  alith,
  baltathar,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_PRIVATE_KEY,
} from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { EXTRINSIC_GAS_LIMIT, WEIGHT_PER_GAS } from "../../util/constants";
import {
  createTransaction,
  createTransfer,
  ALITH_TRANSACTION_TEMPLATE,
} from "../../util/transactions";

// This tests an issue where pallet Ethereum in Frontier does not properly account for weight after
// transaction application. Specifically, it accounts for weight before a transaction by multiplying
// GasToWeight by gas_price, but does not adjust this afterwards. This leads to accounting for too
// much weight in a block.
describeDevMoonbeam("Ethereum Weight Accounting", (context) => {
  it("should account for weight used", async function () {
    this.timeout(10000);
    const { block, result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        maxFeePerGas: 10_000_000_000,
        maxPriorityFeePerGas: 0,
        to: baltathar.address,
        nonce: 0,
        data: null,
      })
    );

    const EXPECTED_GAS_USED = 21_000n;
    const EXPECTED_WEIGHT = EXPECTED_GAS_USED * WEIGHT_PER_GAS;

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(BigInt(receipt.gasUsed)).to.equal(EXPECTED_GAS_USED);

    // query the block's weight, whose normal portion should reflect only this txn
    const apiAt = await context.polkadotApi.at(block.hash);
    // TODO: Remove casting when updated to use SpWeightsWeightV2Weight
    let blockWeightsUsed = await apiAt.query.system.blockWeight();
    let normalWeight = blockWeightsUsed.normal.refTime.toBigInt();
    expect(normalWeight).to.equal(EXPECTED_WEIGHT);

    // look for the event for our eth txn
    let wholeBlock = await context.polkadotApi.rpc.chain.getBlock(block.hash);
    let index = wholeBlock.block.extrinsics.findIndex(
      (ext) => ext.method.method == "transact" && ext.method.section == "ethereum"
    );
    const extSuccessEvent = result.events
      .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
      .find(({ event }) => context.polkadotApi.events.system.ExtrinsicSuccess.is(event));

    expect(extSuccessEvent).to.not.be.eq(null);
    let eventWeight = (extSuccessEvent.event.data as any).dispatchInfo.weight.refTime.toBigInt();
    expect(eventWeight).to.eq(EXPECTED_WEIGHT);
  });

  it("should correctly refund weight from excess gas_limit supplied", async function () {
    const gasAmount = (EXTRINSIC_GAS_LIMIT * 8n) / 10n;
    const tx_1 = await createTransfer(context, baltathar.address, 1, {
      gas: gasAmount.toString(),
      nonce: 1,
    });
    const tx_2 = await createTransfer(context, charleth.address, 1, {
      gas: gasAmount.toString(),
      privateKey: BALTATHAR_PRIVATE_KEY,
      nonce: 0,
    });
    const tx_3 = await createTransfer(context, alith.address, 1, {
      gas: gasAmount.toString(),
      privateKey: CHARLETH_PRIVATE_KEY,
      nonce: 0,
    });

    const fails = (await context.createBlock([tx_1, tx_2, tx_3])).result.filter(
      (a) => !a.successful
    );
    expect(fails, `Transactions ${fails.map((a) => a.hash).join(", ")} have failed to be included`)
      .to.be.empty;
  });
});
