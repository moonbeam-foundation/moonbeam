import "@moonbeam-network/api-augment";

import { expect } from "chai";
import Debug from "debug";
import { ethers } from "ethers";

import { alith } from "../../util/accounts";
import { PRECOMPILE_CONVICTION_VOTING_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { expectSubstrateEvent } from "../../util/expect";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContractExecution } from "../../util/transactions";

const debug = Debug("test:precompile-conviction-voting");

const CONVICTION_VOTING_CONTRACT = getCompiled("precompiles/conviction-voting/ConvictionVoting");
const CONVICTION_VOTING_INTERFACE = new ethers.utils.Interface(
  CONVICTION_VOTING_CONTRACT.contract.abi
);

// Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
describeDevMoonbeam("Precompiles - Conviction Voting precompile", (context) => {
  let proposalIndex: number;
  beforeEach("create a proposal", async function () {
    let nonce = (await context.polkadotApi.rpc.system.accountNextIndex(alith.address)).toNumber();
    const call = context.polkadotApi.tx.identity.setIdentity({ display: { raw: "Me" } });
    const block = await context.createBlock([
      context.polkadotApi.tx.preimage
        .notePreimage(call.toHex())
        .signAsync(alith, { nonce: nonce++ }),
      context.polkadotApi.tx.referenda
        .submit(
          { system: "root" },
          { Lookup: { Hash: call.hash.toHex(), len: call.length } },
          { After: 1 }
        )
        .signAsync(alith, { nonce: nonce++ }),
    ]);
    proposalIndex = expectSubstrateEvent(block, "referenda", "Submitted").data[0].toNumber();
  });

  it("should allow to provide decision deposit", async function () {
    const referendaContract = new context.web3.eth.Contract(
      CONVICTION_VOTING_CONTRACT.contract.abi,
      PRECOMPILE_CONVICTION_VOTING_ADDRESS
    );

    const block = await context.createBlock(
      createContractExecution(context, {
        contract: referendaContract,
        contractCall: referendaContract.methods.placeDecisionDeposit(proposalIndex),
      })
    );
    expectEVMResult(block.result.events, "Succeed");
    expectSubstrateEvent(block, "referenda", "DecisionDepositPlaced");
    const { data } = await expectSubstrateEvent(block, "evm", "Log");
    const evmLog = CONVICTION_VOTING_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name, "Wrong event").to.equal("DecisionDepositPlaced");
    expect(evmLog.args.index, "Wrong event").to.equal(proposalIndex);
  });

  it("should fail to place deposit on the wrong proposal", async function () {
    const referendaContract = new context.web3.eth.Contract(
      CONVICTION_VOTING_CONTRACT.contract.abi,
      PRECOMPILE_CONVICTION_VOTING_ADDRESS
    );

    const block = await context.createBlock(
      createContractExecution(context, {
        contract: referendaContract,
        contractCall: referendaContract.methods.placeDecisionDeposit(999999),
      })
    );
    expectEVMResult(block.result.events, "Revert");
  });

  it("should fail to place deposit twice", async function () {
    const referendaContract = new context.web3.eth.Contract(
      CONVICTION_VOTING_CONTRACT.contract.abi,
      PRECOMPILE_CONVICTION_VOTING_ADDRESS
    );

    expectSubstrateEvent(
      await context.createBlock(
        createContractExecution(context, {
          contract: referendaContract,
          contractCall: referendaContract.methods.placeDecisionDeposit(proposalIndex),
        })
      ),
      "referenda",
      "DecisionDepositPlaced"
    );

    const secondBlock = await context.createBlock(
      createContractExecution(context, {
        contract: referendaContract,
        contractCall: referendaContract.methods.placeDecisionDeposit(proposalIndex),
      })
    );
    expectEVMResult(secondBlock.result.events, "Revert");
  });

  it("should fal to vote for the wrong proposal", async function () {
    const invalidProposals = [999, -1, ""];
    for (const proposalIndex of invalidProposals) {
      const referendaContract = new context.web3.eth.Contract(
        CONVICTION_VOTING_CONTRACT.contract.abi,
        PRECOMPILE_CONVICTION_VOTING_ADDRESS
      );

      expectSubstrateEvent(
        await context.createBlock(
          createContractExecution(context, {
            contract: referendaContract,
            contractCall: referendaContract.methods.voteYes(proposalIndex),
          })
        ),
        "referenda",
        "DecisionDepositPlaced"
      );

      const secondBlock = await context.createBlock(
        createContractExecution(context, {
          contract: referendaContract,
          contractCall: referendaContract.methods.placeDecisionDeposit(proposalIndex),
        })
      );
      expectEVMResult(secondBlock.result.events, "Revert");
    }
  });
});
