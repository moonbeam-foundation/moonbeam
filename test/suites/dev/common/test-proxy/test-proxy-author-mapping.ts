import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_SESSION_ADDRESS,
  baltathar,
} from "@moonwall/util";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D010401",
  title: "Proxy : Author Mapping - simple association",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should succeed in adding an association",
      test: async function () {
        const { result } = await context.createBlock(
          context.polkadotJs().tx.proxy.addProxy(BALTATHAR_ADDRESS, "AuthorMapping" as any, 0)
        );
        expect(result!.events[2].event.method).to.be.eq("ProxyAdded");
        expect(result!.events[2].event.data[2].toString()).to.be.eq("AuthorMapping"); //ProxyType
        expect(result!.events[8].event.method).to.be.eq("ExtrinsicSuccess");
        const { result: result2 } = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.proxy(
              ALITH_ADDRESS,
              null,
              context.polkadotJs().tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
            )
            .signAsync(baltathar)
        );

        expect(result2!.events[3].event.method).to.be.eq("ProxyExecuted");
        expect(result2!.events[3].event.data[0].toString()).to.be.eq("Ok");
        expect(result2!.events[7].event.method).to.be.eq("ExtrinsicSuccess");

        expect(
          (await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS))?.account,
          "No association with Alith"
        ).to.eq(ALITH_ADDRESS);
      },
    });
  },
});
