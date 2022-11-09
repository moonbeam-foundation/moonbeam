import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { BN, bnToHex } from "@polkadot/util";
import {
  TREASURY_ACCOUNT,
  MIN_GLMR_STAKING,
  PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
} from "../../util/constants";
import { describeDevMoonbeamAllEthTxTypes, describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransfer, sendPrecompileTx } from "../../util/transactions";
import {
  baltathar,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_PRIVATE_KEY,
} from "../../util/accounts";
import { u128 } from "@polkadot/types";
import { alith } from "../../util/accounts";

// storage key for "transactionPayment.nextFeeMultplier"
const MULTIPLIER_STORAGE_KEY = "0x3f1467a096bcd71a5b6a0c8155e208103f2edf3bdf381debe331ab7446addfdc";

describeDevMoonbeam("Fee Multiplier", (context) => {
  it("should have spendable max", async () => {


    const MULTIPLIER_STORAGE_KEY
      = context.polkadotApi.query.transactionPayment.nextFeeMultiplier.key(0).toString()

    const initialValue = (await context.polkadotApi.query.transactionPayment.nextFeeMultiplier()).toBigInt();
    console.log(`initial value: ${initialValue}`);
    expect(initialValue).to.equal(8_000_000_000_000_000_000n);
    console.log(`wtf 0`);

    const U128_MAX = new BN("340282366920938463463374607431768211455");
    console.log(`wtf 0.a`);
    const newMultiplierValue = context.polkadotApi.createType("u128", U128_MAX);

    console.log(`wtf 1`);

    // set transaction-payment's multiplier to something above max in storage. on the next round,
    // it should enforce its upper bound and reset it.
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [MULTIPLIER_STORAGE_KEY, bnToHex(newMultiplierValue)],
        ])
      )
      .signAndSend(alith);
    console.log(`wtf 2`);
    await context.createBlock();
    console.log(`wtf 3`);

    const newValue = (await context.polkadotApi.query.transactionPayment.nextFeeMultiplier()).toBigInt();
    console.log(`new value: ${newValue}`);
    expect(newValue).to.equal(100_000_000_000_000_000_000_000n);

  });
});
