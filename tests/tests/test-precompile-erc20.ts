import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";
import { GENESIS_ACCOUNT, ALITH } from "../util/constants";

describeDevMoonbeam("Precompiles - ERC20", (context) => {
  it("ERC20 Native currency - getBalance", async function () {
    let selector = `70a08231`;
    let address = ALITH.slice(2).padStart(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: context.web3.utils.numberToHex(1_000_000_000),
        to: "0x0000000000000000000000000000000000000801",
        data: `0x${selector}${address}`,
      },
    ]);

    const genesisHash = await context.polkadotApi.rpc.chain.getBlockHash(0);
    const account = await context.polkadotApi.query.system.account.at(genesisHash, ALITH);
    let amount = "0x" + account.data.free.toHex().slice(2).padStart(64, "0");

    expect(tx_call.result).equals(amount);
  });
});
