import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import { decodeEventLog } from "viem";
import {
  Referenda,
  cancelProposal,
  expectEVMResult,
  expectSubstrateEvent,
} from "../../../../helpers";

// Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
describeSuite({
  id: "D022857",
  title: "Precompiles - Referenda precompile",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let proposalIndex: number;
    const { abi: referendaAbi } = fetchCompiledContract("Referenda");
    let referenda: Referenda;

    beforeEach(async function () {
      let nonce = (
        await context.polkadotJs().rpc.system.accountNextIndex(ALITH_ADDRESS)
      ).toNumber();
      const call = context.polkadotJs().tx.identity.setIdentity({ display: { raw: "Me" } });
      const block = await context.createBlock([
        context
          .polkadotJs()
          .tx.preimage.notePreimage(call.method.toHex())
          .signAsync(alith, { nonce: nonce++ }),
        context
          .polkadotJs()
          .tx.referenda.submit(
            { system: "root" },
            { Lookup: { Hash: call.hash.toHex(), len: call.method.encodedLength } },
            { After: 1 }
          )
          .signAsync(alith, { nonce: nonce++ }),
      ]);
      proposalIndex = expectSubstrateEvent(
        block as any,
        "referenda",
        "Submitted"
      ).data[0].toNumber();
      referenda = new Referenda(context);
    });

    it({
      id: "T01",
      title: "should allow to provide decision deposit",
      test: async function () {
        const block = await referenda
          .withSigner(alith)
          .withExpectEvents([context.polkadotJs().events.referenda.DecisionDepositPlaced])
          .placeDecisionDeposit(proposalIndex)
          .block();

        expectEVMResult(block!.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");

        const evmLog: any = decodeEventLog({
          abi: referendaAbi,
          topics: data[0].topics.map((t) => t.toHex()) as [`0x${string}`],
          data: data[0].data.toHex(),
        });

        expect(evmLog.eventName, "Wrong event").to.equal("DecisionDepositPlaced");
        expect(evmLog.args.index!, "Wrong event").to.equal(proposalIndex);
        expect(evmLog.args.caller!, "Wrong event").to.equal(ALITH_ADDRESS);
      },
    });

    it({
      id: "T02",
      title: "should fail to place deposit on the wrong proposal",
      test: async function () {
        const invalidProposals = [999, 99, (2 ^ 32) - 1, 2 ^ 32];
        for (const proposalIndex of invalidProposals) {
          const block = await referenda
            .withGas(5_000_000n)
            .placeDecisionDeposit(proposalIndex)
            .block();

          expectEVMResult(block.result!.events, "Revert");
          expect(
            async () => await referenda.reset().placeDecisionDeposit(proposalIndex).tx()
          ).rejects.toThrowError("NotOngoing");
        }
      },
    });

    it({
      id: "T03",
      title: "should fail to place deposit twice",
      test: async function () {
        await referenda
          .withSigner(alith)
          .withExpectEvents([context.polkadotJs().events.referenda.DecisionDepositPlaced])
          .placeDecisionDeposit(proposalIndex)
          .block();

        const { result } = await referenda
          .reset()
          .withGas(2_000_000n)
          .placeDecisionDeposit(proposalIndex)
          .block();

        expect(
          async () => await referenda.reset().placeDecisionDeposit(proposalIndex).tx()
        ).rejects.toThrowError("HasDeposit");
        expectEVMResult(result!.events, "Revert");
      },
    });

    it({
      id: "T04",
      title: "should allow to submit at a certain block ",
      test: async function () {
        const trackId = 0;
        const call = context.polkadotJs().tx.identity.setIdentity({ display: { raw: "Me" } });
        const blockNumber = await context.polkadotJs().query.system.number();
        const block = await referenda
          .withSigner(alith)
          .withExpectEvents([context.polkadotJs().events.referenda.Submitted])
          .submitAt(
            trackId,
            call.hash.toHex(),
            call.method.encodedLength,
            blockNumber.toNumber() + 1
          )
          .block();

        expectEVMResult(block!.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");

        const evmLog: any = decodeEventLog({
          abi: referendaAbi,
          topics: data[0].topics.map((t) => t.toHex()) as [`0x${string}`],
          data: data[0].data.toHex(),
        });

        expect(evmLog.eventName, "Wrong event").to.equal("SubmittedAt");
        expect(evmLog.args.trackId, "Wrong event").to.equal(trackId);
        expect(evmLog.args.hash, "Wrong event").to.equal(call.hash.toHex());
      },
    });

    it({
      id: "T05",
      title: "should allow to submit after a certain block",
      test: async function () {
        const trackId = 0;
        const call = context.polkadotJs().tx.identity.setIdentity({ display: { raw: "Me" } });
        const blockNumber = await context.polkadotJs().query.system.number();
        const block = await referenda
          .withSigner(alith)
          .withExpectEvents([context.polkadotJs().events.referenda.Submitted])
          .submitAfter(
            trackId,
            call.hash.toHex(),
            call.method.encodedLength,
            blockNumber.toNumber() + 1
          )
          .block();

        expectEVMResult(block!.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");

        const evmLog: any = decodeEventLog({
          abi: referendaAbi,
          topics: data[0].topics.map((t) => t.toHex()) as [`0x${string}`],
          data: data[0].data.toHex(),
        });

        expect(evmLog.eventName, "Wrong event").to.equal("SubmittedAfter");
        expect(evmLog.args.trackId, "Wrong event").to.equal(trackId);
        expect(evmLog.args.hash, "Wrong event").to.equal(call.hash.toHex());
      },
    });

    it({
      id: "T06",
      title: "should allow to refund decision deposit",
      test: async function () {
        // Place deposit
        await referenda
          .withSigner(alith)
          .withExpectEvents([context.polkadotJs().events.referenda.DecisionDepositPlaced])
          .placeDecisionDeposit(proposalIndex)
          .block();

        await cancelProposal(context, proposalIndex);

        // Refund deposit
        const block = await referenda
          .reset()
          .withSigner(alith)
          .withExpectEvents([context.polkadotJs().events.referenda.DecisionDepositRefunded])
          .refundDecisionDeposit(proposalIndex)
          .block();

        expectEVMResult(block!.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");

        const evmLog: any = decodeEventLog({
          abi: referendaAbi,
          topics: data[0].topics.map((t) => t.toHex()) as [`0x${string}`],
          data: data[0].data.toHex(),
        });

        expect(evmLog.eventName, "Wrong event").to.equal("DecisionDepositRefunded");
        expect(evmLog.args.index!, "Wrong event").to.equal(proposalIndex);
        expect(evmLog.args.caller!, "Wrong event").to.equal(ALITH_ADDRESS);
      },
    });

    it({
      id: "T07",
      title: "should fail to refund unplaced decision deposit",
      test: async function () {
        await cancelProposal(context, proposalIndex);

        // Refund deposit
        const block = await referenda
          .withGas(5_000_000n)
          .refundDecisionDeposit(proposalIndex)
          .block();

        expectEVMResult(block!.result!.events, "Revert");
      },
    });

    it({
      id: "T08",
      title: "should fail to refund decision deposit when the referenda is not closed",
      test: async function () {
        // Place deposit
        await referenda
          .withSigner(alith)
          .withExpectEvents([context.polkadotJs().events.referenda.DecisionDepositPlaced])
          .placeDecisionDeposit(proposalIndex)
          .block();

        // Refund deposit
        const block = await referenda
          .reset()
          .withGas(5_000_000n)
          .refundDecisionDeposit(proposalIndex)
          .block();

        // Check that the transaction failed
        expectEVMResult(block!.result!.events, "Revert");
      },
    });

    it({
      id: "T09",
      title: "should fail to refund decision deposit twice",
      test: async function () {
        // Place deposit
        await referenda
          .withSigner(alith)
          .withExpectEvents([context.polkadotJs().events.referenda.DecisionDepositPlaced])
          .placeDecisionDeposit(proposalIndex)
          .block();

        await cancelProposal(context, proposalIndex);

        // Refund deposit
        await referenda
          .reset()
          .withSigner(alith)
          .withExpectEvents([context.polkadotJs().events.referenda.DecisionDepositRefunded])
          .refundDecisionDeposit(proposalIndex)
          .block();

        // Refund deposit again
        const block = await referenda
          .reset()
          .withGas(5_000_000n)
          .refundDecisionDeposit(proposalIndex)
          .block();

        // Check that the transaction failed
        expectEVMResult(block!.result!.events, "Revert");
      },
    });
  },
});
