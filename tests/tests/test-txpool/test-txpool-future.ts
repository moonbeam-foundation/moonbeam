import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith } from "../../util/accounts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeam("TxPool - Future Ethereum transaction", (context) => {
  let txHash: string;
  before("Setup: Create transaction", async () => {
    const { rawTx } = await createContract(context, "MultiplyBy7", {
      gas: 1048576,
      nonce: 1, // future nonce
    });
    txHash = (await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).result;
  });

  it("should appear in the txpool inspection", async function () {
    let inspect = await customWeb3Request(context.web3, "txpool_inspect", []);
    // web3 rpc returns lowercase
    let data = inspect.result.queued[alith.address.toLowerCase()][context.web3.utils.toHex(1)];
    expect(data).to.not.be.undefined;
    expect(data).to.be.equal(
      "0x0000000000000000000000000000000000000000: 0 wei + 1048576 gas x 1000000000 wei"
    );
  });

  it("should appear in the txpool content", async function () {
    let content = await customWeb3Request(context.web3, "txpool_content", []);
    // web3 rpc returns lowercase
    const data = content.result.queued[alith.address.toLowerCase()][context.web3.utils.toHex(1)];
    expect(data).to.include({
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
      blockNumber: null,
      from: alith.address.toLowerCase(),
      gas: "0x100000",
      gasPrice: "0x3b9aca00",
      hash: txHash,
      nonce: context.web3.utils.toHex(1),
      to: "0x0000000000000000000000000000000000000000",
      value: "0x0",
    });
  });
});
