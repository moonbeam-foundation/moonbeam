import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { EXTRINSIC_GAS_LIMIT } from "../util/constants";
import { createTransfer } from "../util/transactions";
import { customWeb3Request } from "../util/providers";

describeDevMoonbeam("Resubmit transations", (context) => {
  it("should allow resubmitting with higher gas", async function () {

      const testAccount = "0x1111111111111111111111111111111111111111";
      const optionsLowGas = { nonce: 0, gasPrice: 0 };
      const optionsHighGas = { nonce: 0, gasPrice: 1 };

      const transactions = [
        await createTransfer(context.web3, testAccount, 1, optionsLowGas),
        await createTransfer(context.web3, testAccount, 2, optionsHighGas),
      ],
      await context.createBlock({ transactions });

      expect(await context.web3.eth.getBalance(testAccount, 1)).to.equal(
        (2).toString()
      );
  });

  it("should ignore resubmitting with lower gas", async function () {

      const testAccount = "0x1111111111111111111111111111111111111112";
      const optionsLowGas = { nonce: 1, gasPrice: 0 };
      const optionsHighGas = { nonce: 1, gasPrice: 1 };

      const transactions = [
        await createTransfer(context.web3, testAccount, 3, optionsHighGas),
        await createTransfer(context.web3, testAccount, 1, optionsLowGas),
      ],
      await context.createBlock({ transactions });

      expect(await context.web3.eth.getBalance(testAccount, 2)).to.equal(
        (3).toString()
      );
  });
});

