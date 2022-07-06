import "@moonbeam-network/api-augment";

import { expect } from "chai";

import {
  alith,
  ALITH_GENESIS_FREE_BALANCE,
  ALITH_GENESIS_RESERVE_BALANCE,
  ALITH_GENESIS_TRANSFERABLE_BALANCE,
} from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Balance genesis", (context) => {
  it("should be accessible through web3", async function () {
    expect(await context.web3.eth.getBalance(alith.address, 0)).to.equal(
      ALITH_GENESIS_TRANSFERABLE_BALANCE.toString()
    );
  });

  it("should be accessible through polkadotJs", async function () {
    const genesisHash = await context.polkadotApi.rpc.chain.getBlockHash(0);
    const account = await (
      await context.polkadotApi.at(genesisHash)
    ).query.system.account(alith.address);
    expect(account.data.free.toString()).to.equal(ALITH_GENESIS_FREE_BALANCE.toString());
    expect(account.data.reserved.toString()).to.equal(ALITH_GENESIS_RESERVE_BALANCE.toString());
  });
});
