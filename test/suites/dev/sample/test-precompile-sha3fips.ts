import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

describeDevMoonbeamAllEthTxTypes("Precompiles - sha3fips", (context) => {
  // Test taken from https://github.com/binance-chain/bsc/pull/118
  it("sha3fips should be valid", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: "0x0000000000000000000000000000000000000400",
          data:
            "0x0448250ebe88d77e0a12bcf530fe6a2cf1ac176945638d309b840d631940c93b78c2bd6d16f227a887" +
            "7e3f1604cd75b9c5a8ab0cac95174a8a0a0f8ea9e4c10bca",
        })
      ).result
    ).equals("0xc7647f7e251bf1bd70863c8693e93a4e77dd0c9a689073e987d51254317dc704");
  });
});
