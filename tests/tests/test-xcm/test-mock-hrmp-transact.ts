import "@moonbeam-network/api-augment";

import { KeyringPair } from "@polkadot/keyring/types";
import { BN } from "@polkadot/util";
import { expect } from "chai";

import { generateKeyringPair } from "../../util/accounts";
import { descendOriginFromAddress, injectHrmpMessageAndSeal, RawXcmMessage } from "../../util/xcm";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { expectOk } from "../../util/expect";

describeDevMoonbeam("Mock XCM - receive horizontal transact", (context) => {
  let transferredBalance;
  let sendingAddress;
  let random: KeyringPair;

  before("Should receive transact action with DescendOrigin", async function () {
    const { originAddress, descendOriginAddress } = descendOriginFromAddress(context);
    sendingAddress = originAddress;
    random = generateKeyringPair();
    transferredBalance = 10_000_000_000_000_000_000n;

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(descendOriginAddress, transferredBalance)
      )
    );
    const balance = (
      (await context.polkadotApi.query.system.account(descendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it("Should receive transact and should be able to execute ", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const transferCall = context.polkadotApi.tx.balances.transfer(
      random.address,
      transferredBalance / 10n
    );
    const transferCallEncoded = transferCall?.method.toHex();
    // We are going to test that we can receive a transact operation from parachain 1
    // using descendOrigin first
    const xcmMessage = {
      V2: [
        {
          DescendOrigin: {
            X1: {
              AccountKey20: {
                network: "Any",
                key: sendingAddress,
              },
            },
          },
        },
        {
          WithdrawAsset: [
            {
              id: {
                Concrete: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex },
                  },
                },
              },
              fun: { Fungible: transferredBalance / 2n },
            },
          ],
        },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex },
                  },
                },
              },
              fun: { Fungible: transferredBalance / 2n },
            },
            weightLimit: { Limited: new BN(4000000000) },
          },
        },
        {
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(1000000000),
            call: {
              encoded: transferCallEncoded,
            },
          },
        },
      ],
    };

    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, 1, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Make sure the state has ALITH's foreign parachain tokens
    const testAccountBalance = (
      await context.polkadotApi.query.system.account(random.address)
    ).data.free.toBigInt();

    expect(testAccountBalance).to.eq(transferredBalance / 10n);
  });
});
