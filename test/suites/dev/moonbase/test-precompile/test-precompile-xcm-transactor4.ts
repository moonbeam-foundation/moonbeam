import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite } from "@moonwall/cli";
import { fromBytes } from "viem";
import {
  verifyLatestBlockFees,
  expectEVMResult,
  registerXcmTransactorAndContract,
} from "../../../../helpers";

describeSuite({
  id: "D012895",
  title: "Precompiles - xcm transactor",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to issue transfer signed xcm transactor with multilocation",
      test: async function () {
        // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
        // And we need relay tokens for issuing a transaction to be executed in the relay
        const dest: [number, any[]] = [1, []];
        const asset: [number, any[]] = [1, []];
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const weight = 1000;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmTransactorV1",
          functionName: "transactThroughSignedMultilocation",
          args: [dest, asset, weight, transact_call],
          gas: 500_000n,
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
