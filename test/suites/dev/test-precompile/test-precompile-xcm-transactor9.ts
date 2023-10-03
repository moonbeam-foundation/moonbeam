import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, fetchCompiledContract } from "@moonwall/cli";
import {
  createEthersTransaction,
} from "@moonwall/util";
import { fromBytes } from "viem";
import { verifyLatestBlockFees } from "../../../helpers/block.js";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { registerXcmTransactorAndContract } from "../../../helpers/xcm.js";
import { encodeFunctionData } from "viem";

const PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS = "0x0000000000000000000000000000000000000817";

describeSuite({
  id: "D2581",
  title: "Precompiles - xcm transactor V3",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to transact signed multilocation with custom weights V2 and fee",
      test: async function () {
        // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
        // And we need relay tokens for issuing a transaction to be executed in the relay
        const dest: [number, {}[]] = [1, []];
        const asset: [number, {}[]] = [1, []];
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const transactWeight = {refTime: 1000, proofSize: 1000};
        const overallWeight = {refTime: 2000, proofSize: 2000};
        const feeAmount = 1000;
        const refund = true;

        const { abi: transactorABI } = fetchCompiledContract("XcmTransactorV3");

        const rawTx = await createEthersTransaction(context, {
          to: PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS,
          gas: 500_000n,
          data: encodeFunctionData({
            abi: transactorABI,
            functionName: "transactThroughSignedMultilocation",
            args: [dest, asset, transactWeight, transact_call, feeAmount, overallWeight, refund],
          }),
        });

        const {result} = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Succeed");

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
