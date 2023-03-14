import { describeSuite, beforeAll, expect } from "@moonsong-labs/moonwall-cli";
import { ALITH_PRIVATE_KEY, alith } from "@moonsong-labs/moonwall-util";
import { ApiPromise } from "@polkadot/api";
import { parseEther, ethers, Transaction, Wallet, parseUnits } from "ethers";
import "@polkadot/api-augment";
// import "@moonbeam-network/api-augment" // This is disabled until typegen fixed

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

    // This test case isn't working yet, but you get the idea
    it({
      id: "T04",
      title: "Can send send a ETH transaction via substrate",
      modifier: "skip",
      test: async () => {
        const balanceBefore = (await api.query.system.account(DUMMY_ACCOUNT)).data.free.toBigInt();

        const ethApi = new ethers.WebSocketProvider("wss://wss.api.moonbeam.network");
        const signer = new Wallet(ALITH_PRIVATE_KEY, ethApi);
        const tx = new Transaction();

        tx.to = DUMMY_ACCOUNT;
        tx.value = parseEther("2").toString();
        tx.chainId = (await signer.provider.getNetwork()).chainId;
        tx.nonce = await signer.getNonce();
        tx.maxPriorityFeePerGas = parseUnits("1.5", "gwei");
        tx.maxFeePerGas = parseUnits("5", "gwei");
        tx.gasLimit = 300000;

        const signedTx = await signer.signTransaction(tx);
        const signed = Transaction.from(signedTx).signature;
        let transaction = {
          EIP1559: {
            chainId: Transaction.from(signedTx).chainId,
            nonce: Transaction.from(signedTx).nonce,
            maxPriorityFeePerGas: Transaction.from(signedTx).maxPriorityFeePerGas,
            maxFeePerGas: Transaction.from(signedTx).maxFeePerGas,
            gasLimit: Transaction.from(signedTx).gasLimit,
            action: {
              call: DUMMY_ACCOUNT,
            },
            value: Transaction.from(signedTx).value,
            input: "0x",
            accessList: [],
            oddYParity: false,
            r: signed.r,
            s: signed.s,
          },
        };

        await api.tx.ethereum.transact(transaction).signAndSend(alith);
        await context.createBlock();

        const balanceAfter = (await api.query.system.account(DUMMY_ACCOUNT)).data.free.toBigInt();
        expect(balanceBefore < balanceAfter).to.be.true;
      },
    });
  },
});
