import { use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";

import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

import { ALITH, GENESIS_ACCOUNT } from "../../util/constants";

chaiUse(chaiAsPromised);

describeDevMoonbeamAllEthTxTypes("Estimate Gas - Staking precompile", (context) => {
  it("should estimate sufficient gas for revoke", async function () {
    const estimate = await context.web3.eth.estimateGas({
      from: GENESIS_ACCOUNT,
      data: `0xe42366a6000000000000000000000000${ALITH.slice(
        2
      ).toLowerCase()}000000000000000000000000${ALITH.slice(2).toLowerCase()}`,
    });

    console.log(JSON.stringify(estimate, null, 2));
    // TODO: add expectations
  });
});
