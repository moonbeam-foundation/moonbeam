import { expect } from "chai";
import Keyring from "@polkadot/keyring";

import {
  DEFAULT_GENESIS_STAKING,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

// const testAuthorId = "0x0af46a0ee747ab44191404543214f634ab09b237aa14d355fb0f5f92c3cd414f";
// const testAuthorId2 = "0x8c8618d1daab13e60fe2af2b17dc1a223326b8dc425dcca24c1d27661889e140";

const testAuthorId = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const testAuthorId2 = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

describeDevMoonbeam("Author Mapping - Genesis", (context) => {
  it("should match collator reserved bond reserved", async function () {
    console.log((await context.polkadotApi.query.authorMapping.mapping(testAuthorId)).toHuman());
    console.log((await context.polkadotApi.query.authorMapping.mapping(testAuthorId2)).toHuman());
    // expect(account.data.reserved.toString()).to.equal(DEFAULT_GENESIS_STAKING.toString());
    // console.log("oh",(await api.query.authorMapping.mappings(lastHeader.digest.logs[0].asConsensus[1])).toHuman())
  });
  it("should match collator reserved bond reserved", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    console.log(Object.keys(context.polkadotApi.tx));
    await context.polkadotApi.tx.authorMapping
      .addAssociation(testAuthorId)
      .signAndSend(genesisAccount);
    console.log("ok");
    console.log((await context.polkadotApi.query.authorMapping.mapping(testAuthorId)).toHuman());
  });
});
