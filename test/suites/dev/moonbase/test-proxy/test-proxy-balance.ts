import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_SESSION_ADDRESS,
  CHARLETH_ADDRESS,
  baltathar,
} from "@moonwall/util";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D013002",
  title: "Proxy: Balances",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should accept known proxy",
      test: async () => {
        const beforeCharlieBalance = await context.viem().getBalance({ address: CHARLETH_ADDRESS });
        const { result } = await context.createBlock(
          context.polkadotJs().tx.proxy.addProxy(baltathar.address, "Balances" as any, 0)
        );
        expect(result!.events[2].event.method).to.be.eq("ProxyAdded");
        expect(result!.events[2].event.data[2].toString()).to.be.eq("Balances"); //ProxyType
        expect(result!.events[8].event.method).to.be.eq("ExtrinsicSuccess");

        const { result: result2 } = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.proxy(
              ALITH_ADDRESS,
              null,
              context.polkadotJs().tx.balances.transferAllowDeath(CHARLETH_ADDRESS, 100)
            )
            .signAsync(baltathar)
        );

        expect(result2!.events[2].event.method).to.be.eq("ProxyExecuted");
        expect(result2!.events[2].event.data[0].toString()).to.be.eq("Ok");
        expect(result2!.events[6].event.method).to.be.eq("ExtrinsicSuccess");
        const afterCharlieBalance = await context.viem().getBalance({ address: CHARLETH_ADDRESS });
        expect(afterCharlieBalance - beforeCharlieBalance).to.be.eq(100n);
      },
    });

    it({
      id: "T02",
      title: "shouldn't accept other proxy types",
      test: async () => {
        await context.createBlock(
          context.polkadotJs().tx.proxy.addProxy(baltathar.address, "Balances", 0)
        );

        const beforeAlithBalance = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.proxy(
              ALITH_ADDRESS,
              null,
              context.polkadotJs().tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
            )
            .signAsync(baltathar)
        );

        expect(result!.events[1].event.method).to.be.eq("ProxyExecuted");
        expect(result!.events[1].event.data[0].toString()).to.be.eq(
          `{"err":{"module":{"index":0,"error":"0x05000000"}}}`
        );
        expect(result!.events[5].event.method).to.be.eq("ExtrinsicSuccess");

        expect(
          await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS),
          "Association should have failed"
        ).toBeUndefined();
        const afterAlithBalance = await context.viem().getBalance({ address: ALITH_ADDRESS });
        expect(afterAlithBalance - beforeAlithBalance).to.be.eq(0n);
      },
    });
  },
});
