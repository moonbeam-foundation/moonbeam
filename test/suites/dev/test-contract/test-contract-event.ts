import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { encodeDeployData, encodeFunctionData } from "viem";
import { TransactionTypes } from "../../../helpers/viem.js";
import { createEthersTxn } from "../../../helpers/ethers.js";
import { getCompiled } from "../../../helpers/contracts.js";

describeSuite({
  id: "D0604",
  title: "Contract event",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: "should contain event",
        test: async function () {
          const { byteCode, contract } = getCompiled("EventEmitter");

          const { rawSigned } = await createEthersTxn(context, {
            data: encodeDeployData({ abi: contract.abi, bytecode: byteCode, args: [] }),
            txnType,
            gasLimit: 10_000_000,
          });

          const { result } = await context.createBlock(rawSigned);

          expect(result?.successful, "Unsuccessful deploy").toBe(true);
          const receipt = await context
            .viemClient("public")
            .getTransactionReceipt({ hash: result?.hash as `0x${string}` });

          expect(receipt.logs.length).toBe(1);
          expect(
            "0x" + receipt.logs[0].topics[1]!.substring(26, receipt.logs[0].topics[1]!.length + 1)
          ).toBe(ALITH_ADDRESS.toLowerCase());
        },
      });
    }
  },
});
