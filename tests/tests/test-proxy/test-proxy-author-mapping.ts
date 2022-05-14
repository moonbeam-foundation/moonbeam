import { expect } from "chai";
import Keyring from "@polkadot/keyring";

import { ALITH, BOB_AUTHOR_ID } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../../util/substrate-rpc";
import { ALITH_PRIVATE_KEY, BALTATHAR_PRIVATE_KEY } from "../../util/constants";

export async function getMappingInfo(
  context,
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
    const keyring = new Keyring({ type: "ethereum" });
    const alith = keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
    const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");

    const { events } = await createBlockWithExtrinsic(
      context,
      alith,
      // @ts-ignore
      context.polkadotApi.tx.proxy.addProxy(baltathar.address, "AuthorMapping", 0)
    );
    expect(events[2].method).to.be.eq("ProxyAdded");
    expect(events[2].data[2].toString()).to.be.eq("AuthorMapping"); //ProxyType
    expect(events[7].method).to.be.eq("ExtrinsicSuccess");
    const { events: events2 } = await createBlockWithExtrinsic(
      context,
      baltathar,
      context.polkadotApi.tx.proxy.proxy(
        alith.address,
        null,
        context.polkadotApi.tx.authorMapping.addAssociation(BOB_AUTHOR_ID)
      )
    );

    expect(events2[3].method).to.be.eq("ProxyExecuted");
    expect(events2[3].data[0].toString()).to.be.eq("Ok");
    expect(events2[6].method).to.be.eq("ExtrinsicSuccess");

    // // check association
    expect((await getMappingInfo(context, BOB_AUTHOR_ID)).account).to.eq(ALITH);
  });
});
