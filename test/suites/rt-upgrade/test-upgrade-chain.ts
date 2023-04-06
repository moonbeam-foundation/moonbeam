import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { parseEther } from "ethers";
import "@moonbeam-network/api-augment";

describeSuite({
  id: "CHU",
  title: "Chopsticks Upgrade",
  foundationMethods: "chopsticks",
  testCases: ({ it, context }) => {
    let api: ApiPromise;
    const DUMMY_ACCOUNT = "0x11d88f59425cbc1867883fcf93614bf70e87E854";

    beforeAll(() => {
      api = context.polkadotJs();
    });

    it({
      id: "T1",
      title: "Calling chain constants data",
      test: async () => {
        const specVersion = api.consts.system.version.specVersion.toNumber();
        expect(specVersion).to.be.greaterThan(0);
      },
    });

    it({
      id: "T2",
      title: "Do an upgrade test",
      timeout: 30000,
      test: async function () {
        const rtBefore = api.consts.system.version.specVersion.toNumber();
        await context.upgradeRuntime(context);
        const rtafter = api.consts.system.version.specVersion.toNumber();
        expect(rtBefore).toBeLessThan(rtafter);
      },
    });

    it({
      id: "T3",
      title: "Can create new blocks",
      test: async () => {
        const currentHeight = (await api.rpc.chain.getBlock()).block.header.number.toNumber();
        await context.createBlock({ count: 2 });
        const newHeight = (await api.rpc.chain.getBlock()).block.header.number.toNumber();
        expect(newHeight - currentHeight).to.be.equal(2);
      },
    });

    it({
      id: "T4",
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
