import "@moonbeam-network/api-augment";
import Debug from "debug";
import { u8aToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import { getAddress } from "ethers/lib/utils";
import { alith } from "../../util/accounts";
import { jumpBlocks } from "../../util/block";
import { GLMR, PRECOMPILE_REFERENDA_ADDRESS } from "../../util/constants";

import { getCompiled } from "../../util/contracts";

import { expectOk, expectSubstrateEvent, expectSubstrateEvents } from "../../util/expect";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";
import { expectEVMResult, extractRevertReason } from "../../util/eth-transactions";
const debug = Debug("test:precompile-referenda");

const REFERENDA_CONTRACT = getCompiled("precompiles/referenda/Referenda");
const REFERENDA_INTERFACE = new ethers.utils.Interface(REFERENDA_CONTRACT.contract.abi);

// Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
describeDevMoonbeam("Precompiles - Referenda precompile", (context) => {
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
      REFERENDA_CONTRACT.contract.abi,
      PRECOMPILE_REFERENDA_ADDRESS
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
    const evmLog = REFERENDA_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name, "Wrong event").to.equal("DecisionDepositPlaced");
    expect(evmLog.args.index, "Wrong event").to.equal(proposalIndex);
  });

  it("should fail to place deposit on the wrong proposal", async function () {
    const invalidProposals = [999, 99, (2 ^ 32) - 1, 2 ^ 32];
    for (const proposalIndex of invalidProposals) {
      const referendaContract = new context.web3.eth.Contract(
        REFERENDA_CONTRACT.contract.abi,
        PRECOMPILE_REFERENDA_ADDRESS
      );

      const block = await context.createBlock(
        createContractExecution(context, {
          contract: referendaContract,
          contractCall: referendaContract.methods.placeDecisionDeposit(proposalIndex),
        })
      );
      expectEVMResult(block.result.events, "Revert");
      const revertReason = await extractRevertReason(block.result.hash, context.ethers);
      // Full Error expected:
      // Dispatched call failed with error: Module(ModuleError { index: 42, error: [0, 0, 0, 0],
      //     message: Some("NotOngoing") })
      expect(revertReason).to.contain("NotOngoing");
    }
  });

  it("should fail to place deposit twice", async function () {
    const referendaContract = new context.web3.eth.Contract(
      REFERENDA_CONTRACT.contract.abi,
      PRECOMPILE_REFERENDA_ADDRESS
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
});
