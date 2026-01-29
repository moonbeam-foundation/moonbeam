import "@moonbeam-network/api-augment";
import {
  BALTATHAR_ADDRESS,
  GLMR,
  createRawTransfer,
  describeSuite,
  expect,
  sendRawTransaction,
} from "moonwall";
import { setTimeout } from "node:timers/promises";

describeSuite({
  id: "D023504",
  title: "Subscription -  Pending transactions",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should return a valid subscriptionId",
      test: async function () {
        let response: any;
        const sub = await context.web3().eth.subscribe("newPendingTransactions");

        sub.once("data", (data) => {
          response = data;
        });

        const rawTx = await createRawTransfer(context, BALTATHAR_ADDRESS, GLMR);
        const hash = await sendRawTransaction(context, rawTx);
        await setTimeout(200);

        expect(response).not.toBeUndefined();
        expect(response).toBe(hash);
      },
    });
  },
});
