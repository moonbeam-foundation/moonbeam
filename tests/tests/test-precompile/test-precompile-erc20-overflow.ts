import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import { generateKeyringPair } from "../../util/accounts";
import { PRECOMPILE_BATCH_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransaction } from "../../util/transactions";

describeDevMoonbeam("Precompile ERC20 - Transfering through precompile", (context) => {
  const randomAccount = generateKeyringPair();
  // TODO: Remove once v0.9.23 with frontier
  it.skip("should not allow overflowing the value", async function () {
    const batchInterface = new ethers.utils.Interface(
      getCompiled("precompiles/batch/Batch").contract.abi
    );

    // each tx have a different gas limit to ensure it doesn't impact gas used

    const { result } = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_BATCH_ADDRESS,
        data: batchInterface.encodeFunctionData("batchAll", [
          [randomAccount.address],
          [`${(2n ** 128n + 5_000_000_000_000_000_000n).toString()}`],
          [],
          [],
        ]),
      })
    );

    expectEVMResult(result.events, "Error", "OutOfFund");
    const account = await context.polkadotApi.query.system.account(randomAccount.address);
    expect(account.data.free.toBigInt()).to.equal(0n);
    expect(account.data.reserved.toBigInt()).to.equal(0n);
    expect(await context.web3.eth.getBalance(randomAccount.address)).to.equal(0n.toString());
  });
});
