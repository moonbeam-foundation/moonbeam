import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { bool, Option, u32 } from "@polkadot/types-codec";
import type { PalletMoonbeamOrbitersCollatorPoolInfo } from "@polkadot/types/lookup";
import type { AccountId20 } from "@polkadot/types/interfaces";

import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { StorageKey } from "@polkadot/types";
const debug = require("debug")("smoke:orbiter");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Verify orbiters`, { wssUrl, relayWssUrl }, (context) => {
  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let collatorsPools: [
    StorageKey<[AccountId20]>,
    Option<PalletMoonbeamOrbitersCollatorPoolInfo>
  ][] = null;
  let registeredOrbiters: [StorageKey<[AccountId20]>, Option<bool>][] = null;
  let counterForCollatorsPool: u32 = null;

  before("Setup api & retrieve data", async function () {
    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    collatorsPools = await apiAt.query.moonbeamOrbiters.collatorsPool.entries();
    registeredOrbiters = await apiAt.query.moonbeamOrbiters.registeredOrbiter.entries();
    counterForCollatorsPool = await apiAt.query.moonbeamOrbiters.counterForCollatorsPool();
  });

  it("should have reserved tokens", async function () {
    const reserves = await apiAt.query.balances.reserves.entries();
    const orbiterReserves = reserves
      .map((reserveSet) =>
        reserveSet[1].find((r) => r.id.toUtf8() == "orbi") ? reserveSet[0].toHex().slice(-40) : null
      )
      .filter((r) => !!r);

    const orbiterAccounts = registeredOrbiters.map((o) => o[0].toHex().slice(-40));

    for (const reservedAccount of orbiterReserves) {
      expect(
        orbiterAccounts,
        `Account ${reservedAccount} has "orbi" reserve but is not orbiter.`
      ).to.include(reservedAccount);
    }

    for (const orbiterAccount of orbiterAccounts) {
      expect(
        orbiterReserves,
        `Account ${orbiterAccount} is orbiter but doesn't have "orbi" reserve.`
      ).to.include(orbiterAccount);
    }
    debug(`Verified ${orbiterAccounts.length} orbiter reserves`);
  });

  it("should be registered if in a pool", async function () {
    for (const orbiterPool of collatorsPools) {
      const collator = orbiterPool[0].toHex().slice(-40);
      const pool = orbiterPool[1].unwrap();
      if (pool.maybeCurrentOrbiter.isSome) {
        const selectedOrbiter = pool.maybeCurrentOrbiter.unwrap().accountId.toHex();
        const poolOrbiters = pool.orbiters.map((o) => o.toHex());

        expect(
          poolOrbiters,
          `Selected orbiter ${selectedOrbiter} is not in the pool ${collator} orbiters`
        ).to.include(selectedOrbiter);
      }
    }

    debug(`Verified ${collatorsPools.length} orbiter pools`);
  });

  it("should not have more pool than the max allowed", async function () {
    expect(collatorsPools.length, `Orbiter pool is too big`).to.be.at.most(
      counterForCollatorsPool.toNumber()
    );

    debug(`Verified orbiter pools size`);
  });
});
