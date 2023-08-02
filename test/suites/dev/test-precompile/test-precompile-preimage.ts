import "@moonbeam-network/api-augment";
import {
  describeSuite,
  expect,
  beforeAll,
  DevModeContext,
  beforeEach,
  fetchCompiledContract,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  PRECOMPILE_PREIMAGE_ADDRESS,
  alith,
  createViemTransaction,
} from "@moonwall/util";
import { expectSubstrateEvent } from "../../../helpers/expect.js";
import { Abi, decodeEventLog, encodeFunctionData } from "viem";
import { expectEVMResult, extractRevertReason } from "../../../helpers/eth-transactions.js";
import { preimage } from "@polkadot/api-derive/democracy";
import { HexString } from "web3";

async function notePreimage(context: DevModeContext, PreimageAbi: Abi, data: string) {
  const call = context.polkadotJs().tx.identity.setIdentity({ display: { raw: data } });
  const rawTx = await createViemTransaction(context, {
    to: PRECOMPILE_PREIMAGE_ADDRESS,
    data: encodeFunctionData({
      abi: PreimageAbi,
      functionName: "notePreimage",
      args: [call.toHex()],
    }),
  });
  const block = await context.createBlock(rawTx);
  return { block, call };
}

async function unnotePreimage(context: DevModeContext, PreimageAbi: Abi, data: string) {
  const call = context.polkadotJs().tx.identity.setIdentity({ display: { raw: data } });
  const rawTx = await createViemTransaction(context, {
    to: PRECOMPILE_PREIMAGE_ADDRESS,
    data: encodeFunctionData({
      abi: PreimageAbi,
      functionName: "unnotePreimage",
      args: [call.hash.toHex()],
    }),
  });
  const block = await context.createBlock(rawTx);
  return { block, call };
}

// Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
// Be careful to not reach the maximum number of proposals.
describeSuite({
  id: "D2580",
  title: "Precompiles - Preimage precompile",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    let PreimageAbi: Abi;

    beforeAll(async function () {
      const { abi } = fetchCompiledContract("Preimage");
      PreimageAbi = abi;
    });

    beforeEach(async function () {});

    it({
      id: "T01",
      title: "should allow to note Preimage",
      test: async function () {
        const { block, call } = await notePreimage(context, PreimageAbi, "Me");

        // Verifies the EVM Side
        expectEVMResult(block.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");
        const evmLog = decodeEventLog({
          abi: PreimageAbi,
          topics: data[0].topics.map((t) => t.toHex()) as any,
          data: data[0].data.toHex(),
        }) as any;
        expect(evmLog.eventName, "Wrong event").to.equal("PreimageNoted");
        expect(evmLog.args.hash).to.equal(call.hash.toHex());

        // Verifies the Substrate side
        const preImage = await context
          .polkadotJs()
          .query.preimage.preimageFor([call.hash.toHex(), 15]);
        expect(preImage.unwrap().toHex()).to.equal(call.toHex());
      },
    });

    it({
      id: "T02",
      title: "should allow to unnote a Preimage",
      test: async function () {
        await notePreimage(context, PreimageAbi, "You");
        const { block, call } = await unnotePreimage(context, PreimageAbi, "You");

        // Verifies the EVM Side
        expectEVMResult(block.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");
        const evmLog = decodeEventLog({
          abi: PreimageAbi,
          topics: data[0].topics.map((t) => t.toHex()) as any,
          data: data[0].data.toHex(),
        }) as any;
        // context.readPrecompile!({functionName})
        expect(evmLog.eventName, "Wrong event").to.equal("PreimageUnnoted");
        expect(evmLog.args.hash).to.equal(call.hash.toHex());

        // Verifies the Substrate side
        const preImage = await context
          .polkadotJs()
          .query.preimage.preimageFor([call.hash.toHex(), 1000]);
        expect(preImage.isNone).to.equal(true);
      },
    });

    it({
      id: "T03",
      title: "should fail to note the same Preimage twice",
      test: async function () {
        await notePreimage(context, PreimageAbi, "Repeated");
        expect(
          async () => await notePreimage(context, PreimageAbi, "Repeated"),
          "Transaction should be reverted but instead preimage noted"
        ).rejects.toThrowError("AlreadyNoted");
      },
    });

    it({
      id: "T04",
      title: "should fail to unnote a missing Preimage",
      test: async function () {
        expect(
          async () => await unnotePreimage(context, PreimageAbi, "Missing Preimage"),
          "Transaction should be reverted but instead preimage unnoted"
        ).rejects.toThrowError("NotNoted");
      },
    });
  },
});
