import "@moonbeam-network/api-augment";

import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { expectEVMResult } from "../../../../helpers";

const CONTRACT_NAME = "P256Verify";

describeSuite({
  id: "D012803",
  title: "Precompiles - p256verify",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let contractAddress: `0x${string}`;

    beforeAll(async function () {
      const { contractAddress: _contractAddress } = await context.deployContract!(CONTRACT_NAME);
      contractAddress = _contractAddress;
    });

    it({
      id: "T01",
      title: "should be accessible from a smart contract",
      test: async function () {
        const rawTx = await context.writeContract!({
          contractName: CONTRACT_NAME,
          contractAddress: contractAddress,
          functionName: "test",
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTx);

        expect(result.successful, "Succeed");
        expectEVMResult(result!.events, "Succeed");
      },
    });
  },
});
