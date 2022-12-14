import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar, BALTATHAR_SESSION_ADDRESS } from "../../util/accounts";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

export async function getMappingInfo(
  context: DevTestContext,
  authorId: string
): Promise<{ account: string; deposit: BigInt }> {
  const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(authorId);
  if (mapping.isSome) {
    return {
      account: mapping.unwrap().account.toString(),
      deposit: mapping.unwrap().deposit.toBigInt(),
    };
  }
  return null;
}

describeDevMoonbeam("Proxy : Author Mapping - simple association", (context) => {
  it("should succeed in adding an association", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.proxy.addProxy(baltathar.address, "AuthorMapping" as any, 0)
    );
    expect(events[2].event.method).to.be.eq("ProxyAdded");
    expect(events[2].event.data[2].toString()).to.be.eq("AuthorMapping"); //ProxyType
    expect(events[8].event.method).to.be.eq("ExtrinsicSuccess");
    const {
      result: { events: events2 },
    } = await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(
          alith.address,
          null,
          context.polkadotApi.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
        )
        .signAsync(baltathar)
    );

    expect(events2[3].event.method).to.be.eq("ProxyExecuted");
    expect(events2[3].event.data[0].toString()).to.be.eq("Ok");
    expect(events2[7].event.method).to.be.eq("ExtrinsicSuccess");

    // // check association
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(alith.address);
  });
});
