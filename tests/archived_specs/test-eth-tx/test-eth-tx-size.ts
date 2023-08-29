import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeam } from "../../../util/setup-dev-tests";

import { EXTRINSIC_GAS_LIMIT } from "../../../util/constants";
import { customWeb3Request } from "../../../util/providers";
import { ethers } from "ethers";
import { alith, ALITH_PRIVATE_KEY } from "../../../util/accounts";

describeDevMoonbeam("Ethereum Transaction - Large Transaction", (context) => {
  // function to generate a junk transaction with a specified data size
  const generateLargeTxn = async (size: bigint) => {
    const byte = "FF";
    const data = "0x" + byte.repeat(Number(size));

    let signer = new ethers.Wallet(ALITH_PRIVATE_KEY, context.ethers);

    return await signer.signTransaction({
      from: alith.address,
      to: null,
      value: "0x0",
      gasLimit: EXTRINSIC_GAS_LIMIT,
      gasPrice: 10_000_000_000,
      data: data,
      nonce: await context.web3.eth.getTransactionCount(alith.address),
    });
  };

  // TODO: I'm not sure where this 2000 came from...
  const max_size = (EXTRINSIC_GAS_LIMIT - 21000n) / 16n - 2000n;

  it("should accept txns up to known size", async function () {
    expect(max_size).to.equal(809187n); // our max Ethereum TXN size in bytes

    // max_size - shanghai init cost - create cost
    let max_size_shanghai = max_size - 6474n;

    const tx = await generateLargeTxn(max_size_shanghai);
    const { result } = await context.createBlock(tx);
    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.status).to.be.false; // this txn is nonsense, but the RPC should accept it
  });

  it("should reject txns which are too large to pay for", async function () {
    const tx = await generateLargeTxn(max_size + 1n);
    const txResults = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);

    // RPC should outright reject this txn -- this is important because it prevents it from being
    // gossipped, thus preventing potential for spam
    expect(txResults).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      error: { message: "intrinsic gas too low", code: -32603 },
    });
  });
});
