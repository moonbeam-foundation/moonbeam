import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";

describeSuite({
  id: "D024126",
  title: "Mock XCM - EthereumXcm only disable by root",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should check suspend ethereum xcm only callable by root",
      test: async function () {
        let suspended = await context.polkadotJs().query.ethereumXcm.ethereumXcmSuspended();
        // should be not suspended by default
        expect(suspended.toHuman()).to.be.false;

        // We try to activate without sudo
        await context.createBlock(
          context.polkadotJs().tx.ethereumXcm.suspendEthereumXcmExecution().signAsync(alith)
        );
        suspended = await context.polkadotJs().query.ethereumXcm.ethereumXcmSuspended();
        // should not have worked, and should still not be suspended
        expect(suspended.toHuman()).to.be.false;

        // Now with sudo
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.ethereumXcm.suspendEthereumXcmExecution())
            .signAsync(alith)
        );

        suspended = await context.polkadotJs().query.ethereumXcm.ethereumXcmSuspended();
        // should have worked, and should now be suspended
        expect(suspended.toHuman()).to.be.true;
      },
    });
  },
});
