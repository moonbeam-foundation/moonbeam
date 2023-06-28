import "@moonbeam-network/api-augment";

import { u8aToHex } from "@polkadot/util";
import { expect } from "chai";
import Debug from "debug";
import { ethers } from "ethers";
import { getAddress } from "ethers/lib/utils";

import { alith } from "../../util/accounts";
import { jumpBlocks } from "../../util/block";
import { GLMR } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { expectOk, expectSubstrateEvent, expectSubstrateEvents } from "../../util/expect";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

const debug = Debug("test:precompile-referenda");

const REFERENDA_CONTRACT = getCompiled("precompiles/referenda/Referenda");
const REFERENDA_INTERFACE = new ethers.utils.Interface(REFERENDA_CONTRACT.contract.abi);
const PREIMAGE_CONTRACT = getCompiled("precompiles/preimage/Preimage");
const PREIMAGE_INTERFACE = new ethers.utils.Interface(PREIMAGE_CONTRACT.contract.abi);
const CONVICTION_VOTING_CONTRACT = getCompiled("precompiles/conviction-voting/ConvictionVoting");
const CONVICTION_VOTING_INTERFACE = new ethers.utils.Interface(
  CONVICTION_VOTING_CONTRACT.contract.abi
);

describeDevMoonbeam("Precompiles - Referenda Auto Upgrade Demo", (context) => {
  it("should be accessible from a smart contract", async function () {
    this.timeout(180000);
    const setStorageCallIndex = u8aToHex(context.polkadotApi.tx.system.setStorage.callIndex);
    const trackName = "root";
    const tracksInfo = await context.polkadotApi.consts.referenda.tracks;
    const trackInfo = tracksInfo.find((track) => track[1].name.toString() == trackName);
    expect(trackInfo).to.not.be.empty;

    let nonce = (await context.polkadotApi.rpc.system.accountNextIndex(alith.address)).toNumber();
    const contractV1 = await createContract(
      context,
      "ReferendaAutoUpgradeDemoV1",
      {
        nonce: nonce++,
      },
      [trackName, setStorageCallIndex]
    );
    const contractV2 = await createContract(
      context,
      "ReferendaAutoUpgradeDemoV2",
      {
        nonce: nonce++,
      },
      [trackName, setStorageCallIndex]
    );
    await context.createBlock([contractV1.rawTx, contractV2.rawTx]);

    const contractJson = getCompiled("ReferendaAutoUpgradeDemoV1");
    const contractAbi = new ethers.utils.Interface(contractJson.contract.abi);

    const ethersContract = new ethers.Contract(
      contractV1.contractAddress,
      contractAbi,
      context.ethers
    );

    expect(
      (await ethersContract.version()).toBigInt(),
      "Version should first be initialized to 1"
    ).to.equals(1n);

    const v1Code = await context.polkadotApi.query.evm.accountCodes(contractV1.contractAddress);
    const v1CodeKey = context.polkadotApi.query.evm.accountCodes.key(contractV1.contractAddress);
    const v2CodeKey = context.polkadotApi.query.evm.accountCodes.key(contractV2.contractAddress);
    const v2CodeStorage = (await context.polkadotApi.rpc.state.getStorage(v2CodeKey)) as any;

    expect(await context.polkadotApi.query.evm.accountCodes(contractV1.contractAddress)).to.not.eq(
      v1Code
    );

    // Gives the contract 500M Tokens to allow to quickly pass the referenda
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.balances.setBalance(
          contractV1.contractAddress,
          500_000_000n * GLMR,
          0
        )
      )
    );

    const data = await context.createBlock(
      createContractExecution(context, {
        contract: contractV1.contract,
        contractCall: contractV1.contract.methods.autoUpgrade(v2CodeStorage.toHex(), v1CodeKey),
      })
    );
    const {
      data: [referendumIndex],
    } = expectSubstrateEvent(data, "referenda", "Submitted");
    expectSubstrateEvent(data, "referenda", "DecisionDepositPlaced");

    // We all of the EVM Logs, but only some of their inputs, not all of them
    const evmEvents = expectSubstrateEvents(data, "evm", "Log");
    const expectedEvents = [
      { interface: PREIMAGE_INTERFACE, name: "PreimageNoted" },
      { interface: REFERENDA_INTERFACE, name: "SubmittedAfter", inputs: { trackId: 0 } },
      {
        interface: REFERENDA_INTERFACE,
        name: "DecisionDepositPlaced",
        inputs: { index: referendumIndex.toNumber() },
      },
      {
        interface: CONVICTION_VOTING_INTERFACE,
        name: "Voted",
        inputs: {
          pollIndex: referendumIndex.toNumber(),
          voter: getAddress(contractV1.contractAddress),
          aye: true,
          conviction: 1,
        },
      },
    ];
    expectedEvents.forEach((expectedEvent, index) => {
      const evmLog = expectedEvent.interface.parseLog({
        topics: evmEvents[index].data[0].topics.map((t) => t.toHex()),
        data: evmEvents[index].data[0].data.toHex(),
      });

      expect(evmLog.name, "Wrong event").to.equal(expectedEvent.name);

      if (expectedEvent.inputs) {
        Object.keys(expectedEvent.inputs).forEach((inputName) => {
          expect(
            expectedEvent.inputs[inputName],
            `${expectedEvent.name}.${inputName} not matching`
          ).to.equal(evmLog.args[inputName]);
        });
      }
    });

    let referendumInfo = await context.polkadotApi.query.referenda.referendumInfoFor(
      referendumIndex
    );

    expect(referendumInfo.isSome, "Referenda should contain the proposal").to.be.true;
    expect(referendumInfo.unwrap().isOngoing, "Referenda should be ongoing").to.be.true;
    expect(
      referendumInfo.unwrap().asOngoing.deciding.isNone,
      "Referenda should still be in preparation"
    ).to.be.true;

    debug(`Waiting preparation time: ${trackInfo[1].preparePeriod.toNumber()}`);
    await jumpBlocks(context, trackInfo[1].preparePeriod.toNumber());
    referendumInfo = await context.polkadotApi.query.referenda.referendumInfoFor(referendumIndex);
    expect(referendumInfo.unwrap().asOngoing.deciding.isSome, "Referenda should now be in deciding")
      .to.be.true;

    debug(`Waiting confirmation time: ${trackInfo[1].minEnactmentPeriod.toNumber()}`);
    await jumpBlocks(context, trackInfo[1].confirmPeriod.toNumber());
    referendumInfo = await context.polkadotApi.query.referenda.referendumInfoFor(referendumIndex);
    expect(referendumInfo.unwrap().isApproved, "Referenda should now be approved").to.be.true;

    debug(`Waiting enactment time: ${trackInfo[1].minEnactmentPeriod.toNumber()}`);
    await jumpBlocks(context, trackInfo[1].confirmPeriod.toNumber());

    expect(
      (await ethersContract.version()).toBigInt(),
      "Version should haven update to 2"
    ).to.equals(2n);
  });

  it("should be work for valid tracks", async function () {
    this.timeout(180000);
    const validTracks = [
      "root",
      "whitelisted_caller",
      "general_admin",
      "referendum_canceller",
      "referendum_killer",
    ];
    for (const trackName of validTracks) {
      const setStorageCallIndex = u8aToHex(context.polkadotApi.tx.system.setStorage.callIndex);

      let nonce = (await context.polkadotApi.rpc.system.accountNextIndex(alith.address)).toNumber();
      const contract = await createContract(
        context,
        "ReferendaAutoUpgradeDemoV1",
        {
          nonce: nonce++,
        },
        [trackName, setStorageCallIndex]
      );
      const { result } = await expectOk(context.createBlock(contract.rawTx));
      expectEVMResult(result.events, "Succeed");
      expect(
        (await context.polkadotApi.query.evm.accountCodes(contract.contractAddress)).toHex(),
        "Contract should have been deployed"
      ).to.be.length.above(2);
    }
  });

  it("should be fail for invalid tracks", async function () {
    this.timeout(180000);
    const validTracks = ["toor", "", 0, "admin", -1, "0x01", "0xFFFF", "0xFFFFFFFF"];
    for (const trackName of validTracks) {
      const setStorageCallIndex = u8aToHex(context.polkadotApi.tx.system.setStorage.callIndex);

      let nonce = (await context.polkadotApi.rpc.system.accountNextIndex(alith.address)).toNumber();
      const contract = await createContract(
        context,
        "ReferendaAutoUpgradeDemoV1",
        {
          nonce: nonce++,
          gas: 5_000_000, // To avoid the gas estimation failing
        },
        [trackName, setStorageCallIndex]
      );
      const { result } = await context.createBlock(contract.rawTx);
      expectEVMResult(result.events, "Revert"); // No Revert reason to validate

      expect(
        (await context.polkadotApi.query.evm.accountCodes(contract.contractAddress)).toHex(),
        "Contract should not have been deployed"
      ).to.be.length.at.most(2);
    }
  });
});
