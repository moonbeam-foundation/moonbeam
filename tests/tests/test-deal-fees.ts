import { expect } from "chai";
import Keyring from "@polkadot/keyring";
import { ALITH_PRIV_KEY, TREASURY_ACCOUNT } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";

describeDevMoonbeam("blabla", (context) => {
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
  it("20% of transfer fees should deposit in treasury", async () => {
    //expect(await context.web3.eth.getBalance(TREASURY_ACCOUNT, 0)).to.equal(0n.toString());
    //console.log(JSON.stringify(await context.polkadotApi.query.balances.account(TREASURY_ACCOUNT)));
    const blockResult = await context.createBlock({
      transactions: [await createTransfer(context.web3, TEST_ACCOUNT, 512)],
    }); 
    const allRecords = await context.polkadotApi.query.system.events.at(blockResult.block.hash);
    console.log(
      JSON.stringify(
        allRecords.filter(({ phase }) => phase.isApplyExtrinsic).map(({ event }) => event)
      )
    );
    //console.log(JSON.stringify(await context.polkadotApi.query.balances.account(TREASURY_ACCOUNT)));
    /*expect(await context.web3.eth.getBalance(TREASURY_ACCOUNT, 1)).to.equal(0n.toString());

    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    let { events } = await createBlockWithExtrinsic(
      context,
      genesisAccount,
      context.polkadotApi.tx.balances.transfer(TEST_ACCOUNT, 512)
    );
    console.log(JSON.stringify(events));
    console.log(JSON.stringify(await context.polkadotApi.query.balances.account("0x6d6F646c70632f74727372790000000000000000")));
    let { events: events2 } = await createBlockWithExtrinsic(
      context,
      genesisAccount,
      context.polkadotApi.tx.balances.transfer(TEST_ACCOUNT, 512)
    );
    console.log(JSON.stringify(events2));*/
  });
});
