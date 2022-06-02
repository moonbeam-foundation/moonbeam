import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { alith, baltathar } from "../../util/accounts";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../../util/substrate-rpc";

// A call from root (sudo) can make a transfer directly in pallet_evm
// A signed call cannot make a transfer directly in pallet_evm

describeDevMoonbeam("Pallet EVM transfer - no sudo", (context) => {
  let events;
  before("Send a simple transfer with pallet evm", async () => {
    ({ events } = await createBlockWithExtrinsic(
      context,
      alith,
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
    ));
  });

  it("should fail without sudo", async function () {
    expect(events[5].toHuman().method).to.eq("ExtrinsicFailed");
    expect(await context.web3.eth.getBalance(baltathar.address)).to.equal("0");
  });
});
describeDevMoonbeam("Pallet EVM transfer - with sudo", (context) => {
  let events;
  before("Send a simple transfer with pallet evm with sudo", async () => {
    ({ events } = await createBlockWithExtrinsic(
      context,
      alith,
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
    ));
  });

  it("should succeed with sudo", async function () {
    expect(events[13].toHuman().method).to.eq("ExtrinsicSuccess");
    expect(await context.web3.eth.getBalance(baltathar.address)).to.equal(
      100_000_000_000_000_000_000n.toString()
    );
  });
});
