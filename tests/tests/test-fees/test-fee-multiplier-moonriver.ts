import "@moonbeam-network/api-augment/moonbase";
import { expect } from "chai";
import { BN, bnToHex } from "@polkadot/util";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { baltathar, BALTATHAR_ADDRESS, generateKeyringPair } from "../../util/accounts";
import { alith } from "../../util/accounts";
import { createContract, createContractExecution } from "../../util/transactions";
import {
  RawXcmMessage,
  XcmFragment,
  descendOriginFromAddress,
  injectHrmpMessageAndSeal,
} from "../../util/xcm";
import { expectOk } from "../../util/expect";
import { KeyringPair } from "@substrate/txwrapper-core";
import { TARGET_FILL_AMOUNT } from "../../util/constants";
import { execTechnicalCommitteeProposal, executeProposalWithCouncil } from "../../util/governance";
import { blake2AsHex } from "@polkadot/util-crypto";

// Note on the values from 'transactionPayment.nextFeeMultiplier': this storage item is actually a
// FixedU128, which is basically a u128 with an implicit denominator of 10^18. However, this
// denominator is omitted when it is queried through the API, leaving some very large numbers.
//
// To make sense of them, basically remove 18 zeroes (divide by 10^18). This will give you the
// number used internally by transaction-payment for fee calculations.

describeDevMoonbeam("Max Fee Multiplier (Moonriver)", (context) => {
  beforeEach("set to max multiplier (moonriver)", async function () {
    this.timeout(20000);
    const MULTIPLIER_STORAGE_KEY = context.polkadotApi.query.transactionPayment.nextFeeMultiplier
      .key(0)
      .toString();

    const U128_MAX = new BN("340282366920938463463374607431768211455");

    // set transaction-payment's multiplier to something above max in storage. on the next round,
    // it should enforce its upper bound and reset it.
    let proposal = context.polkadotApi.tx.system.setStorage([
      [MULTIPLIER_STORAGE_KEY, bnToHex(U128_MAX, { isLe: true, bitLength: 128 })],
    ]);
    let encodedProposal = proposal.method.toHex();
    let encodedHash = blake2AsHex(encodedProposal);
    await executeProposalWithCouncil(context.polkadotApi, encodedHash);
  });

  it("should enforce upper bound (moonriver)", async function () {
    this.timeout(20000);
    // we set it to u128_max, but the max should have been enforced in on_finalize()
    const multiplier = (
      await context.polkadotApi.query.transactionPayment.nextFeeMultiplier()
    ).toBigInt();
    expect(multiplier).to.equal(100_000_000_000_000_000_000_000n);

    const result = await context.ethers.send("eth_gasPrice", []);
    const gasPrice = BigInt(result);
    expect(gasPrice).to.eq(125_000_000_000_000n);
  });

},
"Legacy",
"moonriver");
