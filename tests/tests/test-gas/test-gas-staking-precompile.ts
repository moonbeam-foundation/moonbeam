import { expect, use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";

import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

import { ALITH, GENESIS_ACCOUNT } from "../../util/constants";

chaiUse(chaiAsPromised);

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Staking precompile", (context) => {
  it("should estimate sufficient gas for revoke", async function () {
    const estimate = await context.web3.eth.estimateGas({
      from: GENESIS_ACCOUNT,
      data: `0xe42366a6${ALITH.slice(2).toLowerCase().padStart(64, "0")}${ALITH.slice(2)
        .toLowerCase()
        .padStart(64, "0")}`,
    });

    expect(estimate).to.be.at.least(55641);
    expect(estimate).to.be.at.most(55641 * 1.1); // Allow 10% extra
  });
});
