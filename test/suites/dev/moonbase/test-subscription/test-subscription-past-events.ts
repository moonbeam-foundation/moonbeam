import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS, ALITH_CONTRACT_ADDRESSES, createEthersTransaction } from "@moonwall/util";
import { encodeDeployData } from "viem";
import { web3SubscribeHistoricalLogs } from "../../../../helpers";

describeSuite({
  id: "D013503",
  title: "Subscription - Past Events",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      const { abi, bytecode } = fetchCompiledContract("EventEmitter");
      let nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });
      const rawTx1 = await createEthersTransaction(context, {
        data: encodeDeployData({
          abi,
          bytecode,
        }),
        nonce: nonce++,
      });

      const rawTx2 = await createEthersTransaction(context, {
        data: encodeDeployData({
          abi,
          bytecode,
        }),
        nonce: nonce++,
      });

      const rawTx3 = await createEthersTransaction(context, {
        data: encodeDeployData({
          abi,
          bytecode,
        }),
        nonce: nonce++,
      });

      const rawTx4 = await createEthersTransaction(context, {
        data: encodeDeployData({
          abi,
          bytecode,
        }),
        nonce: nonce++,
      });

      await context.createBlock([rawTx1, rawTx2, rawTx3, rawTx4]);
    });

    it({
      id: "T01",
      title: "should be retrieved by topic",
      test: async function () {
        const filter = {
          fromBlock: 0,
          topics: ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
        };

        const eventLogs = await web3SubscribeHistoricalLogs(context.web3(), 200, filter);
        expect(eventLogs).to.not.be.empty;
      },
    });

    it({
      id: "T02",
      title: "should be retrieved by address",
      test: async function () {
        const filter = {
          fromBlock: 0,
          address: "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
        };

        const eventLogs = await web3SubscribeHistoricalLogs(context.web3(), 200, filter);
        expect(eventLogs).to.not.be.empty;
      },
    });

    it({
      id: "T03",
      title: "should be retrieved by address + topic",
      test: async function () {
        const filter = {
          fromBlock: "0x0",
          topics: ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
          address: "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
        };

        const eventLogs = await web3SubscribeHistoricalLogs(context.web3(), 200, filter);
        expect(eventLogs).to.not.be.empty;
      },
    });

    it({
      id: "T04",
      title: "should be retrieved by multiple addresses",
      test: async function () {
        const filter = {
          fromBlock: "0x0",
          topics: ["0x0040d54d5e5b097202376b55bcbaaedd2ee468ce4496f1d30030c4e5308bf94d"],
          address: [
            ALITH_CONTRACT_ADDRESSES[4],
            ALITH_CONTRACT_ADDRESSES[3],
            ALITH_CONTRACT_ADDRESSES[2],
            ALITH_CONTRACT_ADDRESSES[1],
            ALITH_CONTRACT_ADDRESSES[0],
          ],
        };

        const eventLogs = await web3SubscribeHistoricalLogs(context.web3(), 200, filter);
        expect(eventLogs).to.not.be.empty;
      },
    });
  },
});
