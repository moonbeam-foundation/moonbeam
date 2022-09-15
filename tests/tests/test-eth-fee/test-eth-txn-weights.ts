import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, ALITH_PRIVATE_KEY, baltathar } from "../../util/accounts";
import { getCompiled } from "../../util/contracts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { EXTRINSIC_GAS_LIMIT, EXTRINSIC_BASE_WEIGHT, WEIGHT_PER_GAS } from "../../util/constants";
import { createTransaction, ALITH_TRANSACTION_TEMPLATE } from "../../util/transactions";

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
        gas: EXTRINSIC_GAS_LIMIT,
        maxFeePerGas: 1_000_000_000,
        maxPriorityFeePerGas: 0,
        to: baltathar.address,
        data: null,
      })
    );

    const EXPECTED_GAS_USED = 21_000n;
    const EXPECTED_WEIGHT = EXPECTED_GAS_USED * WEIGHT_PER_GAS + BigInt(EXTRINSIC_BASE_WEIGHT);

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(BigInt(receipt.gasUsed)).to.equal(EXPECTED_GAS_USED);

    // query the block's weight, whose normal portion should reflect only this txn
    const apiAt = await context.polkadotApi.at(block.hash);

    let blockWeightsUsed = await apiAt.query.system.blockWeight();
    let normalWeight = blockWeightsUsed.normal;

    expect(normalWeight.toBigInt()).to.equal(EXPECTED_WEIGHT);
  });
});
