import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import { alith, ALITH_PRIVATE_KEY, generateKeyringPair } from "../../util/accounts";
import { GLMR } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

// A call from root (sudo) can make a transfer directly in pallet_evm
// A signed call cannot make a transfer directly in pallet_evm

describeDevMoonbeam("Pallet EVM - Transfering", (context) => {
  const randomAccount = generateKeyringPair();
  it("should not overflow", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.evm.call(
          alith.address,
          randomAccount.address,
          "0x0",
          `0x${(5n * GLMR + 2n ** 128n).toString(16)}`,
          "0x5209",
          1_000_000_000n,
          "0",
          undefined,
          []
        )
      )
    );

    expect(
      events.find(
        ({ event: { section, method } }) => section == "system" && method == "ExtrinsicSuccess"
      )
    ).to.exist;

    const account = await context.polkadotApi.query.system.account(randomAccount.address);
    expect(account.data.free.toBigInt()).to.equal(0n);
    expect(account.data.reserved.toBigInt()).to.equal(0n);
  });
});
