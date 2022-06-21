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
    const runtimeVersion = await context.polkadotApi.runtimeVersion.specVersion.toNumber();
    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    collatorsPools = await apiAt.query.moonbeamOrbiters.collatorsPool.entries();
    registeredOrbiters =
      runtimeVersion >= 1605 ? await apiAt.query.moonbeamOrbiters.registeredOrbiter.entries() : [];
    counterForCollatorsPool = await apiAt.query.moonbeamOrbiters.counterForCollatorsPool();
  });

  it("should have reserved tokens", async function () {
    const reserves = await apiAt.query.balances.reserves.entries();
    const orbiterReserves = reserves
      .map((reserveSet) =>
        reserveSet[1].find((r) => r.id.toUtf8() == "orbi")
          ? `0x${reserveSet[0].toHex().slice(-40)}`
          : null
      )
      .filter((r) => !!r);

    const orbiterRegisteredAccounts = registeredOrbiters.map((o) => `0x${o[0].toHex().slice(-40)}`);

    for (const reservedAccount of orbiterReserves) {
      expect(
        orbiterRegisteredAccounts,
        `Account ${reservedAccount} has "orbi" reserve but is not orbiter.`
      ).to.include(reservedAccount);
    }

    for (const orbiterAccount of orbiterRegisteredAccounts) {
      expect(
        orbiterReserves,
        `Account ${orbiterAccount} is orbiter but doesn't have "orbi" reserve.`
      ).to.include(orbiterAccount);
    }
    debug(`Verified ${orbiterRegisteredAccounts.length} orbiter reserves`);
  });

  it("should be registered if in a pool", async function () {
    for (const orbiterPool of collatorsPools) {
      const collator = `0x${orbiterPool[0].toHex().slice(-40)}`;
      const pool = orbiterPool[1].unwrap();
      const orbiterRegisteredAccounts = registeredOrbiters.map(
        (o) => `0x${o[0].toHex().slice(-40)}`
      );
      if (pool.maybeCurrentOrbiter.isSome) {
        const selectedOrbiter = pool.maybeCurrentOrbiter.unwrap().accountId.toHex();
        const isRemoved = pool.maybeCurrentOrbiter.unwrap().removed.isTrue;
        const poolOrbiters = pool.orbiters.map((o) => o.toHex());

        if (isRemoved) {
          expect(
            poolOrbiters,
            `Selected orbiter ${selectedOrbiter} is removed but ` +
              `still in the pool ${collator} orbiters`
          ).to.not.include(selectedOrbiter);
        } else {
          expect(
            poolOrbiters,
            `Selected orbiter ${selectedOrbiter} is not in the pool ${collator} orbiters`
          ).to.include(selectedOrbiter);
        }

        expect(
          orbiterRegisteredAccounts,
          `Account ${selectedOrbiter} is in a pool but not registered`
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
