import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { expectSubstrateEvent } from "../../../helpers/expect.js";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { decodeEventLog } from "viem";

// Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
describeSuite({
  id: "D2552",
  title: "Precompiles - Referenda precompile",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let proposalIndex: number;
    const { abi: referendaAbi } = fetchCompiledContract("Referenda");

    beforeEach(async function () {
      let nonce = (
        await context.polkadotJs().rpc.system.accountNextIndex(alith.address)
      ).toNumber();
      const call = context.polkadotJs().tx.identity.setIdentity({ display: { raw: "Me" } });
      const block = await context.createBlock([
        context
          .polkadotJs()
          .tx.preimage.notePreimage(call.toHex())
          .signAsync(alith, { nonce: nonce++ }),
        context
          .polkadotJs()
          .tx.referenda.submit(
            { system: "root" },
            { Lookup: { Hash: call.hash.toHex(), len: call.length } },
            { After: 1 }
          )
          .signAsync(alith, { nonce: nonce++ }),
      ]);
      proposalIndex = expectSubstrateEvent(
        block as any,
        "referenda",
        "Submitted"
      ).data[0].toNumber();
    });

    it({
      id: "T01",
      title: "should allow to provide decision deposit",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Referenda",
          functionName: "placeDecisionDeposit",
          args: [proposalIndex],
          rawTxOnly: true,
        });

        const block = await context.createBlock(rawTxn, {
          expectEvents: [context.polkadotJs().events.referenda.DecisionDepositPlaced],
          signer: alith,
        });

        expectEVMResult(block!.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");

        const evmLog: any = decodeEventLog({
          abi: referendaAbi,
          topics: data[0].topics.map((t) => t.toHex()) as [`0x${string}`],
          data: data[0].data.toHex(),
        });

        expect(evmLog.eventName, "Wrong event").to.equal("DecisionDepositPlaced");
        expect(evmLog.args.index!, "Wrong event").to.equal(proposalIndex);
      },
    });

    it({
      id: "T02",
      title: "should fail to place deposit on the wrong proposal",
      test: async function () {
        const invalidProposals = [999, 99, (2 ^ 32) - 1, 2 ^ 32];
        for (const proposalIndex of invalidProposals) {
          const rawTxn = await context.writePrecompile!({
            precompileName: "Referenda",
            functionName: "placeDecisionDeposit",
            args: [proposalIndex],
            rawTxOnly: true,
            gas: 5_000_000n,
          });

          const block = await context.createBlock(rawTxn);

          expectEVMResult(block.result!.events, "Revert");
          expect(
            async () =>
              await context.writePrecompile!({
                precompileName: "Referenda",
                functionName: "placeDecisionDeposit",
                args: [proposalIndex],
                gas: "estimate",
              })
          ).rejects.toThrowError("NotOngoing");
        }
      },
    });

    it({
      id: "T03",
      title: "should fail to place deposit twice",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Referenda",
          functionName: "placeDecisionDeposit",
          args: [proposalIndex],
          rawTxOnly: true,
        });

        await context.createBlock(rawTxn, {
          signer: alith,
          expectEvents: [context.polkadotJs().events.referenda.DecisionDepositPlaced],
        });

        const { result } = await context.createBlock(
          context.writePrecompile!({
            precompileName: "Referenda",
            functionName: "placeDecisionDeposit",
            args: [proposalIndex],
            gas: 1_000_000n,
            rawTxOnly: true,
          })
        );

        expect(
          async () =>
            await context.writePrecompile!({
              precompileName: "Referenda",
              functionName: "placeDecisionDeposit",
              args: [proposalIndex],
              gas: "estimate",
            })
        ).rejects.toThrowError("HasDeposit");
        expectEVMResult(result!.events, "Revert");
      },
    });
  },
});
