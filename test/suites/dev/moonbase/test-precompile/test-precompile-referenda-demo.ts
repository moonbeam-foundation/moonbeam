import "@moonbeam-network/api-augment";
import {
  deployCreateCompiledContract,
  describeSuite,
  expect,
  fetchCompiledContract,
} from "@moonwall/cli";
import { GLMR } from "@moonwall/util";
import { u8aToHex } from "@polkadot/util";
import { decodeEventLog, getAddress } from "viem";
import {
  forceReducedReferendaExecution,
  expectSubstrateEvent,
  expectSubstrateEvents,
} from "../../../../helpers";

describeSuite({
  id: "D012971",
  title: "Precompiles - Referenda Auto Upgrade Demo",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    it({
      id: "T01",
      title: "should be accessible from a smart contract",
      test: async function () {
        const setStorageCallIndex = u8aToHex(context.polkadotJs().tx.system.setStorage.callIndex);
        const trackName = "root";
        const tracksInfo = context.polkadotJs().consts.referenda.tracks;
        const trackInfo = tracksInfo.find((track) => track[1].name.toString() == trackName);
        expect(trackInfo).to.not.be.empty;

        const { contractAddress: refUpgradeDemoV1Address, abi: refUpgradeDemoV1Abi } =
          await context.deployContract!("ReferendaAutoUpgradeDemoV1", {
            args: [trackName, setStorageCallIndex],
          });
        const { contractAddress: refUpgradeDemoV2Address, abi: refUpgradeDemoV2Abi } =
          await context.deployContract!("ReferendaAutoUpgradeDemoV2", {
            args: [trackName, setStorageCallIndex],
          });

        // We verify the contract is version 1.
        // After running the proposal it will auto-upgrade to version 2.
        expect(
          await context.readContract!({
            contractAddress: refUpgradeDemoV1Address,
            contractName: "ReferendaAutoUpgradeDemoV1",
            functionName: "version",
          })
        ).toBe(1n);

        const v1Code = await context.polkadotJs().query.evm.accountCodes(refUpgradeDemoV1Address);
        const v1CodeKey = context.polkadotJs().query.evm.accountCodes.key(refUpgradeDemoV1Address);
        const v2CodeKey = context.polkadotJs().query.evm.accountCodes.key(refUpgradeDemoV2Address);
        const v2CodeStorage = (await context.polkadotJs().rpc.state.getStorage(v2CodeKey)) as any;

        expect(
          await context.polkadotJs().query.evm.accountCodes(refUpgradeDemoV1Address)
        ).to.not.eq(v1Code);

        // Gives the contract 500M Tokens to allow to quickly pass the referenda
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.balances.forceSetBalance(refUpgradeDemoV1Address, 500_000_000n * GLMR)
            )
        );

        const rawTxn = await context.writeContract!({
          contractAddress: refUpgradeDemoV1Address,
          contractName: "ReferendaAutoUpgradeDemoV1",
          functionName: "autoUpgrade",
          args: [v2CodeStorage.toHex(), v1CodeKey],
          rawTxOnly: true,
        });

        const result = (await context.createBlock(rawTxn)) as any;

        const {
          data: [referendumIndex],
        } = expectSubstrateEvent(result, "referenda", "Submitted");
        expectSubstrateEvent(result, "referenda", "DecisionDepositPlaced");

        const { abi: preimageAbi } = fetchCompiledContract("Preimage");
        const { abi: referendaAbi } = fetchCompiledContract("Referenda");
        const { abi: convictionAbi } = fetchCompiledContract("ConvictionVoting");

        // We all of the EVM Logs, but only some of their inputs, not all of them
        const evmEvents = expectSubstrateEvents(result, "evm", "Log");
        const expectedEvents = [
          { interface: preimageAbi, name: "PreimageNoted" },
          { interface: referendaAbi, name: "SubmittedAfter", inputs: { trackId: 0 } },
          {
            interface: referendaAbi,
            name: "DecisionDepositPlaced",
            inputs: { index: referendumIndex.toNumber() },
          },
          {
            interface: convictionAbi,
            name: "Voted",
            inputs: {
              pollIndex: referendumIndex.toNumber(),
              voter: getAddress(refUpgradeDemoV1Address),
              aye: true,
              conviction: 1,
            },
          },
        ];
        expectedEvents.forEach((expectedEvent: any, index) => {
          const evmLog: any = decodeEventLog({
            abi: expectedEvent.interface,
            topics: evmEvents[index].data[0].topics.map((t) => t.toHex()) as [`0x${string}`],
            data: evmEvents[index].data[0].data.toHex(),
          });

          if (expectedEvent.inputs) {
            Object.keys(expectedEvent.inputs).forEach((inputName) => {
              expect(
                expectedEvent.inputs[inputName],
                `${expectedEvent.name}.${inputName} not matching`
              ).to.equal(evmLog.args[inputName]);
            });
          }
        });

        const referendumInfo = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(referendumIndex);

        expect(referendumInfo.isSome, "Referenda should contain the proposal").to.be.true;
        expect(referendumInfo.unwrap().isOngoing, "Referenda should be ongoing").to.be.true;
        expect(
          referendumInfo.unwrap().asOngoing.deciding.isNone,
          "Referenda should still be in preparation"
        ).to.be.true;

        // Keeping this for knowledge on fast tracking by producing many blocks
        // This is a bit slow (30s)

        // log(`Waiting preparation time: ${trackInfo![1].preparePeriod.toNumber()}`);
        // await jumpBlocks(context, trackInfo![1].preparePeriod.toNumber());
        // referendumInfo = await context
        //   .polkadotJs()
        //   .query.referenda.referendumInfoFor(referendumIndex);
        // expect(
        //   referendumInfo.unwrap().asOngoing.deciding.isSome,
        //   "Referenda should now be in deciding"
        // ).to.be.true;

        // log(`Waiting confirmation time: ${trackInfo![1].minEnactmentPeriod.toNumber()}`);
        // await jumpBlocks(context, trackInfo![1].confirmPeriod.toNumber());
        // referendumInfo = await context
        //   .polkadotJs()
        //   .query.referenda.referendumInfoFor(referendumIndex);
        //expect(referendumInfo.unwrap().isApproved, "Referenda should now be approved").to.be.true;

        // log(`Waiting enactment time: ${trackInfo![1].minEnactmentPeriod.toNumber()}`);
        // await jumpBlocks(context, trackInfo![1].confirmPeriod.toNumber());

        // Use this forced reduced referenda execution which modify the storage data of the
        // referenda and the agenda
        await forceReducedReferendaExecution(context, referendumIndex.toNumber(), {
          forceTally: false,
        });

        expect(
          await context.readContract!({
            contractAddress: refUpgradeDemoV1Address,
            contractName: "ReferendaAutoUpgradeDemoV1",
            functionName: "version",
          }),
          "Version should haven update to 2"
        ).toBe(2n);
      },
    });

    it({
      id: "T02",
      title: "should work for valid tracks",
      test: async function () {
        const validTracks = [
          "root",
          "whitelisted_caller",
          "general_admin",
          "referendum_canceller",
          "referendum_killer",
        ];
        const failures: any[] = [];
        for (const trackName of validTracks) {
          const setStorageCallIndex = u8aToHex(context.polkadotJs().tx.system.setStorage.callIndex);

          const { contractAddress } = await context.deployContract!("ReferendaAutoUpgradeDemoV1", {
            args: [trackName, setStorageCallIndex],
          });

          if (
            (await context.polkadotJs().query.evm.accountCodes(contractAddress)).toHex().length <= 2
          ) {
            failures.push(trackName);
            log(`Contract not deployed for track ${trackName}`);
          }
        }

        expect(failures.length).toBe(0);
      },
    });

    it({
      id: "T03",
      title: "should be fail for invalid tracks",
      test: async function () {
        const validTracks = ["toor", "", 0, "admin", -1, "0x01", "0xFFFF", "0xFFFFFFFF"];
        const failures: any[] = [];
        for (const trackName of validTracks) {
          const setStorageCallIndex = u8aToHex(context.polkadotJs().tx.system.setStorage.callIndex);

          const { contractAddress, status } = await deployCreateCompiledContract(
            context,
            "ReferendaAutoUpgradeDemoV1",
            {
              args: [trackName, setStorageCallIndex],
              gas: 5_000_000n,
            }
          );

          if (
            status === "success" ||
            (await context.polkadotJs().query.evm.accountCodes(contractAddress)).toHex().length > 2
          ) {
            failures.push(trackName);
            log(`Contract deployed for track ${trackName}, but it should not have been`);
          }
        }

        expect(failures.length).toBe(0);
      },
    });
  },
});
