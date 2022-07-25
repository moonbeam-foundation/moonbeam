import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import { alith, ALITH_PRIVATE_KEY, generateKeyringPair } from "../../util/accounts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

// Casting of type in solidity is performing truncation:
// https://docs.soliditylang.org/en/latest/types.html#conversions-between-elementary-types
describeDevMoonbeam("Precompile Batch - Overflowing gasLimit", (context) => {
  const randomAccount = generateKeyringPair();
  // This case can no longer be tested from frontier 0.9.23 because it is no longer possible to
  // enter a gas_limit that exceeds 65% of the block.
  it.skip("should get truncated and valid", async function () {
    // We are creating a fake function to override the argument type from uint64 to uint256
    const batchInterface = new ethers.utils.Interface([
      "function batchAll(address[], uint256[], bytes[], uint64[])",
      "function hackedbatchAll(address[], uint256[], bytes[], uint256[])",
    ]);
    // each tx have a different gas limit to ensure it doesn't impact gas used
    let batchAllTx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: "0x0000000000000000000000000000000000000808",
        gas: "0x110000",
        value: "0x00",
        nonce: 0,
        data:
          `${batchInterface.encodeFunctionData("batchAll", [[], [], [], []]).slice(0, 10)}` +
          `${batchInterface
            .encodeFunctionData("hackedbatchAll", [
              [randomAccount.address],
              [`${3_000_000_000_000_000_000n.toString()}`],
              [],
              [`${(2n ** 128n + 22_000n).toString()}`],
            ])
            .slice(10)}`,
      },
      ALITH_PRIVATE_KEY
    );

    const batchAllResult = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      batchAllTx.rawTransaction,
    ]);

    await context.createBlock();

    const batchAllReceipt = await context.web3.eth.getTransactionReceipt(batchAllResult.result);
    expect(batchAllReceipt.status).to.be.true;

    const account = await context.polkadotApi.query.system.account(randomAccount.address);
    expect(account.data.free.toBigInt()).to.equal(3_000_000_000_000_000_000n);
    expect(account.data.reserved.toBigInt()).to.equal(0n);
  });
});
