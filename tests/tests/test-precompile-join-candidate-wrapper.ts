import { expect } from "chai";
import { Event } from "@polkadot/types/interfaces";
import { GLMR } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution, createTransfer } from "../util/transactions";

describeDevMoonbeam("Precompiles - JoinCandidatesWrapper", (context) => {
  it("should be accessible from a smart contract", async function () {
    //TODO we need to somehow get some code at the address 256.

    const { rawTx, contract, contractAddress } = await createContract(
      context.web3,
      "JoinCandidatesWrapper",
      {},
      ["0x0000000000000000000000000000000000000100"] // parameter for constructor. Where is the interface located
    );
    const res = await context.createBlock({ transactions: [rawTx] });
    // console.log("RES", res);
    // console.log("contractAddress", contractAddress);

    // Transfer 10k GLMR to the contract
    // could go in the block above. Harmless as it is
    await context.createBlock({
      transactions: [await createTransfer(context.web3, contractAddress, 10_000n * GLMR)],
    });

    // Call the join method
    await context.createBlock({
      transactions: [
        await createContractExecution(context.web3, {
          contract,
          contractCall: contract.methods.join(),
        }),
      ],
    });

    // Check the candidate pool because the contract just enetered. It might not (probably won't) be
    // in the active set yet.
    const collators = await context.polkadotApi.query.parachainStaking.candidatePool();
    console.log("COLLATORS", collators.toHuman());
    expect((collators[1] as Buffer).toString("hex").toLowerCase()).equal(contractAddress);

    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    const allRecords = await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    );

    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
      // filter the specific events based on the phase and then the
      // index of our extrinsic in the block
      const events: Event[] = allRecords
        .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
        .map(({ event }) => event);
      console.log(`${section}.${method}:: ${events.join(", ") || "no events"}`);
    });
    expect(true);
  });
});
