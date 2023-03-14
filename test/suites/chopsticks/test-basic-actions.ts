import { describeSuite, beforeAll, expect, alithSigner } from "@moonsong-labs/moonwall-cli";
import { ALITH_ADDRESS, ALITH_PRIVATE_KEY, alith } from "@moonsong-labs/moonwall-util";
import { ApiPromise } from "@polkadot/api";
import { parseEther } from "ethers";
import w3 from "web3";
import "@polkadot/api-augment";

describeSuite({
  id: "CMB01",
  title: "Chopsticks test suite",
  foundationMethods: "chopsticks",
  testCases: ({ it, context }) => {
    let api: ApiPromise;
    const DUMMY_ACCOUNT = "0x11d88f59425cbc1867883fcf93614bf70e87E854";

    beforeAll(() => {
      api = context.getMoonbeam();
    });

    it({
        id: "T01",
        title: "Calling chain constants data",
        modifier: "only",
        test: async () => {
          const specName = api.consts.system.version.specName.toString();
          expect(specName).to.contain("moonbeam");
        },
      });

    it({
      id: "T01",
      title: "Calling chain constants data",
      test: async () => {
        const specName = api.consts.system.version.specName.toString();
        expect(specName).to.contain("moonbeam");
      },
    });

    it({
      id: "T02",
      title: "Can create new blocks",
      test: async () => {
        const currentHeight = (await api.rpc.chain.getBlock()).block.header.number.toNumber();
        await context.createBlock({ count: 2 });
        const newHeight = (await api.rpc.chain.getBlock()).block.header.number.toNumber();
        expect(newHeight - currentHeight).to.be.equal(2);
      },
    });

    it({
      id: "T03",
      title: "Can send balance transfers",
      test: async () => {
        const balanceBefore = (await api.query.system.account(DUMMY_ACCOUNT)).data.free.toBigInt();
        await api.tx.balances.transfer(DUMMY_ACCOUNT, parseEther("1")).signAndSend(alith);
        await context.createBlock();
        const balanceAfter = (await api.query.system.account(DUMMY_ACCOUNT)).data.free.toBigInt();
        expect(balanceBefore < balanceAfter).to.be.true;
      },
    });

    it({
      id: "T04",
      title: "Can send send a ETH transaction via substrate",
      test: async () => {
        const balanceBefore = (await api.query.system.account(DUMMY_ACCOUNT)).data.free.toBigInt();

        const web3 = new w3();
        const txn = web3.eth.sendTransaction({
          from: ALITH_ADDRESS,
          to: DUMMY_ACCOUNT,
          value: parseEther("2"),
        })
        // const signed = web3.eth.accounts.sign(txn, ALITH_PRIVATE_KEY);
        // await api.tx.balances.transfer(DUMMY_ACCOUNT,parseEther("1")).signAndSend(alith)
        // await context.createBlock()

        const balanceAfter = (await api.query.system.account(DUMMY_ACCOUNT)).data.free.toBigInt();
        expect(balanceBefore < balanceAfter).to.be.true;
      },
    });
  },
});
