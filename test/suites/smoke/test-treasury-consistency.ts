import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "S25",
  title: "Verify treasury consistency",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise">;
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      atBlockNumber = (await paraApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));
    });

    it({
      id: "C100",
      title: "should have value > 0",
      test: async function () {
        // Load data
        const treasuryPalletId = paraApi.consts.treasury.palletId;
        const treasuryAccount = await apiAt.query.system.account(
          `0x6d6f646C${treasuryPalletId.toString().slice(2)}0000000000000000`
        );

        expect(treasuryAccount.data.free.toBigInt() > 0n).to.be.true;
        expect(treasuryAccount.data.reserved.toBigInt()).to.be.equal(0n);

        log(`Verified treasury free/reserved balance`);
      },
    });
  },
});
