import { expect } from "chai";
import { Contract } from "web3-eth-contract";

import { GENESIS_ACCOUNT } from "../util/constants";
import { createContract, createContractExecution } from "../util/transactions";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";

describeDevMoonbeam("TxPool - Future Ethereum transaction", (context) => {
  let txHash;
  before("Setup: Create transaction", async () => {
    const { rawTx } = await createContract(context.web3, "TestContract", {
      gas: 1048576,
      nonce: 1, // future nonce
    });
    txHash = (await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).result;
  });

  it("should appear in the txpool inspection", async function () {
    let inspect = await customWeb3Request(context.web3, "txpool_inspect", []);
    let data = inspect.result.queued[GENESIS_ACCOUNT][context.web3.utils.toHex(1)];
    expect(data).to.not.be.undefined;
    expect(data).to.be.equal(
      "0x0000000000000000000000000000000000000000: 0 wei + 1048576 gas x 1000000000 wei"
    );
  });

  it("should appear in the txpool content", async function () {
    let content = await customWeb3Request(context.web3, "txpool_content", []);

    const data = content.result.queued[GENESIS_ACCOUNT][context.web3.utils.toHex(1)];
    expect(data).to.include({
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
      blockNumber: null,
      from: GENESIS_ACCOUNT,
      gas: "0x100000",
      gasPrice: "0x3b9aca00",
      hash: txHash,
      nonce: context.web3.utils.toHex(1),
      to: "0x0000000000000000000000000000000000000000",
      value: "0x0",
    });
  });
});
