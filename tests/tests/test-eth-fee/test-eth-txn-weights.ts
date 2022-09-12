import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import { alith, ALITH_PRIVATE_KEY } from "../../util/accounts";
import { getCompiled } from "../../util/contracts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes, DevTestContext } from "../../util/setup-dev-tests";
import { EXTRINSIC_GAS_LIMIT } from "../../util/constants";

// This tests an issue where pallet Ethereum in Frontier does not properly account for weight after
// transaction application. Specifically, it accounts for weight before a transaction by multiplying
// GasToWeight by gas_price, but does not adjust this afterwards. This leads to accounting for too
// much weight in a block.
describeDevMoonbeamAllEthTxTypes("Ethereum Weight Accounting", (context) => {
  it("should account for weight used", async function () {
    this.timeout(10000);

    let signer = new ethers.Wallet(ALITH_PRIVATE_KEY, context.ethers);

    let tx = await signer.signTransaction({
      from: alith.address,
      to: null,
      value: "0x0",
      gasLimit: EXTRINSIC_GAS_LIMIT,
      gasPrice: 1_000_000_000,
      data: null,
      nonce: await context.web3.eth.getTransactionCount(alith.address),
    });

    // TODO: tweak: this includes some priority fee. it would be more clear without this.
    const expected_weight = 1_575_000_000;

    const { block } = await context.createBlock(tx);

    const apiAt = await context.polkadotApi.at(block.hash);

    let blockWeightsUsed = await apiAt.query.system.blockWeight();
    let normalWeight = blockWeightsUsed.normal;

    expect(normalWeight.toNumber()).to.equal(expected_weight);
  });
});
