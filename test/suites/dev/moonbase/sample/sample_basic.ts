import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { CHARLETH_ADDRESS, BALTATHAR_ADDRESS, alith, setupLogger } from "@moonwall/util";
import { parseEther, formatEther, Signer } from "ethers";
import { BN } from "@polkadot/util";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "D014201",
  title: "Dev test suite",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let signer: Signer;
    let w3;
    let polkadotJs: ApiPromise;
    const anotherLogger = setupLogger("anotherLogger");

    beforeAll(() => {
      signer = context.ethers();
      w3 = context.web3();
      polkadotJs = context.polkadotJs();
    });

    it({
      id: "E01",
      title: "Checking that launched node can create blocks",
      test: async function () {
        const block = (await polkadotJs.rpc.chain.getBlock()).block.header.number.toNumber();
        await context.createBlock();
        const block2 = (await polkadotJs.rpc.chain.getBlock()).block.header.number.toNumber();
        log(`Original block #${block}, new block #${block2}`);
        expect(block2).to.be.greaterThan(block);
      },
    });

    it({
      id: "E02",
      title: "Checking that substrate txns possible",
      timeout: 20000,
      test: async function () {
        const balanceBefore = (await polkadotJs.query.system.account(BALTATHAR_ADDRESS)).data.free;

        await polkadotJs.tx.balances
          .transfer(BALTATHAR_ADDRESS, parseEther("2"))
          .signAndSend(alith);

        await context.createBlock();
        const balanceAfter = (await polkadotJs.query.system.account(BALTATHAR_ADDRESS)).data.free;
        log(
          `Baltathar account balance before ${formatEther(
            balanceBefore.toBigInt()
          )} GLMR, balance after ${formatEther(balanceAfter.toBigInt())} GLMR`
        );
        expect(balanceBefore.lt(balanceAfter)).to.be.true;
      },
    });

    it({
      id: "E03",
      title: "Checking that sudo can be used",
      test: async function () {
        await context.createBlock();
        const tx = polkadotJs.tx.rootTesting.fillBlock(60 * 10 ** 7);
        await polkadotJs.tx.sudo.sudo(tx).signAndSend(alith);

        await context.createBlock();
        const blockFill = await polkadotJs.query.system.blockWeight();
        expect(blockFill.normal.refTime.unwrap().gt(new BN(0))).to.be.true;
      },
    });

    it({
      id: "E04",
      title: "Can send Ethers txns",
      test: async function () {
        const balanceBefore = (await polkadotJs.query.system.account(BALTATHAR_ADDRESS)).data.free;

        await signer.sendTransaction({
          to: BALTATHAR_ADDRESS,
          value: parseEther("1.0"),
          nonce: await signer.getNonce(),
        });
        await context.createBlock();
        anotherLogger("Example use of another logger");
        const balanceAfter = (await polkadotJs.query.system.account(BALTATHAR_ADDRESS)).data.free;
        expect(balanceBefore.lt(balanceAfter)).to.be.true;
      },
    });

    it({
      id: "T05",
      title: "Testing out Create block and listen for event",
      timeout: 30000,
      test: async function () {
        const expectEvents = [
          polkadotJs.events.system.ExtrinsicSuccess,
          polkadotJs.events.balances.Transfer,
          // polkadotJs.events.authorFilter.EligibleUpdated,
        ];

        await context.createBlock(
          polkadotJs.tx.balances.transferAllowDeath(CHARLETH_ADDRESS, parseEther("3")),
          { expectEvents, logger: log }
        );
      },
    });

    it({
      id: "T06",
      title: "Testing out Create block and analyse failures",
      timeout: 30000,
      test: async function () {
        const { result } = await context.createBlock(
          polkadotJs.tx.balances.forceTransfer(
            BALTATHAR_ADDRESS,
            CHARLETH_ADDRESS,
            parseEther("3")
          ),
          { allowFailures: true, logger: log }
        );

        expect(
          result.events.find((evt) => polkadotJs.events.system.ExtrinsicFailed.is(evt.event)),
          "No Event found in block"
        ).toBeTruthy();
      },
    });
  },
});
