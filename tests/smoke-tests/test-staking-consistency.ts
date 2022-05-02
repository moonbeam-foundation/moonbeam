import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import type { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import { expect } from "chai";
import { printTokens } from "../util/logging";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:staking");

const wssUrl = process.env.WSS_URL || null;

describeSmokeSuite(`Verify staking consistency`, { wssUrl }, (context) => {
  const accounts: { [account: string]: FrameSystemAccountInfo } = {};

  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let specVersion: number = 0;
  let maxTopDelegationsPerCandidate: number = 0;

  before("Setup apiAt", async function () {
    // It takes time to load all the accounts.
    this.timeout(120000);

    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    specVersion = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion.toNumber();
    maxTopDelegationsPerCandidate =
      apiAt.consts.parachainStaking.maxTopDelegationsPerCandidate.toNumber();
  });

  it("candidate totalCounted matches top X delegations", async function () {
    this.timeout(120000);
    // Load data
    const [candidateInfo, delegatorState] = await Promise.all([
      apiAt.query.parachainStaking.candidateInfo.entries(),
      apiAt.query.parachainStaking.delegatorState.entries(),
    ]);

    for (const candidate of candidateInfo) {
      const accountId = `0x${candidate[0].toHex().slice(-40)}`;
      const delegations = delegatorState
        .map((state) =>
          state[1]
            .unwrap()
            .delegations.filter((delegation) => delegation.owner.toHex() == accountId)
        )
        .flat();

      const expectedTotalCounted =
        delegations
          .map((d) => d.amount.toBigInt())
          .sort((a, b) => (a < b ? 1 : a > b ? -1 : 0))
          .filter((_, i) => i < maxTopDelegationsPerCandidate)
          .reduce((p, amount) => p + amount, 0n) + candidate[1].unwrap().bond.toBigInt();
      debug(
        accountId,
        printTokens(context.polkadotApi, candidate[1].unwrap().totalCounted.toBigInt()),
        printTokens(context.polkadotApi, expectedTotalCounted)
      );
      expect(candidate[1].unwrap().totalCounted.toBigInt()).to.equal(expectedTotalCounted);
    }

    debug(
      `Verified ${Object.keys(candidateInfo).length} candidates and ${
        delegatorState.length
      } delegators`
    );
  });
});
