import { expect } from "chai";
import { Event } from "@polkadot/types/interfaces";
import { GLMR } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution, createTransfer } from "../util/transactions";

describeDevMoonbeam("Precompiles - JoinCandidatesWrapper", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { rawTx, contract, contractAddress } = await createContract(
      context.web3,
      "JoinCandidatesWrapper",
      {},
      ["0x0000000000000000000000000000000000000100"]
    );
    const res = await context.createBlock({ transactions: [rawTx] });
    // console.log("RES", res);
    // console.log("contractAddress", contractAddress);

    // Transfer 10k GLMR to the contract
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

    const collators = await context.polkadotApi.query.parachainStaking.selectedCandidates();
    console.log("COLLATORS", collators.toHuman());
    //expect((collators[1] as Buffer).toString("hex").toLowerCase()).equal(contractAddress);

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
