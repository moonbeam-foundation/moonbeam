import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { EXTRINSIC_GAS_LIMIT } from "../../util/constants";
import { customWeb3Request } from "../../util/providers";
import { ethers } from "ethers";
import { alith, ALITH_PRIVATE_KEY } from "../../util/accounts";

describeDevMoonbeam("Ethereum Transaction - Base Gas Fee Error", (context) => {
  it("should return proper error", async function () {
    const signer = new ethers.Wallet(ALITH_PRIVATE_KEY, context.ethers);

    const tx = await signer.signTransaction({
      from: alith.address,
      to: null,
      value: "0x0",
      gasLimit: EXTRINSIC_GAS_LIMIT,
      gasPrice: 1, // cannot be less than block base fee
      data: "0x00",
    });
    const txResults = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);

    expect(txResults).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      error: { message: "max fee per gas less than block base fee", code: -32603 },
    });
  });
});
