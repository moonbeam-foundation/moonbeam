import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar } from "../../util/accounts";
import { DEFAULT_GENESIS_BALANCE } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

// A call from root (sudo) can make a transfer directly in pallet_evm
// A signed call cannot make a transfer directly in pallet_evm

describeDevMoonbeam("Pallet EVM - call", (context) => {
  it("should fail without sudo", async function () {
    expect(
      await context
        .createBlock(
          context.polkadotApi.tx.evm.call(
            alith.address,
            baltathar.address,
            "0x0",
            100_000_000_000_000_000_000n,
            12_000_000n,
            1_000_000_000n,
            "0",
            undefined,
            []
          )
        )
        .catch((e) => e.toString())
    ).to.equal("RpcError: 1010: Invalid Transaction: Transaction call is not expected");

    expect(await context.web3.eth.getBalance(baltathar.address)).to.equal(
      DEFAULT_GENESIS_BALANCE.toString()
    );
  });
});

describeDevMoonbeam("Pallet EVM - call", (context) => {
  it("should succeed with sudo", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.evm.call(
          alith.address,
          baltathar.address,
          "0x0",
          100_000_000_000_000_000_000n,
          12_000_000n,
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
    expect(await context.web3.eth.getBalance(baltathar.address)).to.equal(
      (DEFAULT_GENESIS_BALANCE + 100_000_000_000_000_000_000n).toString()
    );
  });
});
