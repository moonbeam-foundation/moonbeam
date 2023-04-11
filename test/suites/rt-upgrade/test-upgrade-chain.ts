import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { parseEther } from "ethers";
import "@moonbeam-network/api-augment";

const env = process.env.GH_WORKFLOW_MATRIX_CHAIN
  ? process.env.GH_WORKFLOW_MATRIX_CHAIN
  : "moonbeam";

describeSuite({
  id: "CIRT",
  title: "Chopsticks Upgrade",
  foundationMethods: "chopsticks",
  testCases: ({ it, context }) => {
    let api: ApiPromise;
    const DUMMY_ACCOUNT = "0x11d88f59425cbc1867883fcf93614bf70e87E854";

    beforeAll(async () => {
      api = context.polkadotJs();

      const rtBefore = api.consts.system.version.specVersion.toNumber();
      await context.upgradeRuntime(context);
      const rtafter = api.consts.system.version.specVersion.toNumber();
      expect(rtBefore, "RT upgrade has not increased specVersion").toBeLessThan(rtafter);

      const specName = api.consts.system.version.specName.toString();
      expect(specName).to.contain(env);
    });

    it({
      id: "T1",
      title: "Can create new blocks",
      test: async () => {
        const currentHeight = (await api.rpc.chain.getBlock()).block.header.number.toNumber();
        await context.createBlock({ count: 2 });
        const newHeight = (await api.rpc.chain.getBlock()).block.header.number.toNumber();
        expect(newHeight - currentHeight).to.be.equal(2);
      },
    });

    it({
      id: "T2",
      title: "Can send balance transfers",
      test: async () => {
        const balanceBefore = (await api.query.system.account(DUMMY_ACCOUNT)).data.free.toBigInt();
        await api.tx.balances.transfer(DUMMY_ACCOUNT, parseEther("1")).signAndSend(alith);
        await context.createBlock();
        const balanceAfter = (await api.query.system.account(DUMMY_ACCOUNT)).data.free.toBigInt();
        expect(balanceBefore < balanceAfter).to.be.true;
      },
    });
  },
});
