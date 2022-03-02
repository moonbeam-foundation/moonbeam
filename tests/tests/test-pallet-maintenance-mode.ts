import Keyring from "@polkadot/keyring";
import { expect } from "chai";
import { BN, hexToU8a } from "@polkadot/util";
import { KeyringPair } from "@polkadot/keyring/types";
import {
  ALITH,
  ALITH_PRIV_KEY,
  BALTATHAR,
  BALTATHAR_PRIV_KEY,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  GLMR,
  relayChainAddress,
} from "../util/constants";
import { execFromAllMembersOfTechCommittee } from "../util/governance";

import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic, createBlockWithExtrinsicParachain } from "../util/substrate-rpc";
import { createTransfer } from "../util/transactions";
import { VESTING_PERIOD } from "./test-crowdloan";
import { mockAssetBalance } from "./test-precompile/test-precompile-assets-erc20";
import { customWeb3Request } from "../util/providers";
import { BALANCES_ADDRESS } from "./test-precompile/test-precompile-xtokens";

const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

export const expectError = (fun): Promise<string> => {
  return new Promise(async (resolve) => {
    try {
      await fun();
    } catch (e) {
      resolve(e.toString());
    }
  });
};

// A call from root (sudo) can make a transfer directly in pallet_evm
// A signed call cannot make a transfer directly in pallet_evm

describeDevMoonbeam("Pallet Maintenance Mode - normal call shouldnt work", (context) => {
  let events;
  before("Try turning maintenance mode on", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    ({ events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
    ));
  });

  it("should fail without sudo", async function () {
    expect(events[5].toHuman().method).to.eq("ExtrinsicFailed");
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal("0");
  });
});
describeDevMoonbeam("Pallet Maintenance Mode - with sudo shouldn't work", (context) => {
  let events;
  before("Try turning maintenance mode on", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    ({ events } = await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      )
    ));
  });

  it("shouldn't succeed with sudo", async function () {
    expect(events[3].toHuman().method).to.eq("ExtrinsicSuccess");
    expect((await context.polkadotApi.query.maintenanceMode.maintenanceMode()).toHuman()).to.equal(
      false
    );
  });
});

describeDevMoonbeam("Pallet Maintenance Mode - with council should work", (context) => {
  let events;
  before("Try turning maintenance mode on", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    // go into Maintenance
    ({ events } = await execFromAllMembersOfTechCommittee(
      context,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
    ));
  });

  it("should succeed with council", async function () {
    expect(events[3].toHuman().method).to.eq("EnteredMaintenanceMode");
    expect((await context.polkadotApi.query.maintenanceMode.maintenanceMode()).toHuman()).to.equal(
      true
    );
  });
});
// Exit
describeDevMoonbeam("Pallet Maintenance Mode - exit mode", (context) => {
  let events;
  before("Try turning maintenance mode on", async () => {
    // go into Maintenance
    await execFromAllMembersOfTechCommittee(
      context,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
    );
    // exit maintenance
    ({ events } = await execFromAllMembersOfTechCommittee(
      context,
      context.polkadotApi.tx.maintenanceMode.resumeNormalOperation()
    ));
  });

  it("should succeed with council", async function () {
    expect(events[3].toHuman().method).to.eq("NormalOperationResumed");
    expect((await context.polkadotApi.query.maintenanceMode.maintenanceMode()).toHuman()).to.equal(
      false
    );
  });
});
describeDevMoonbeam(
  "Pallet Maintenance Mode - exit mode - make sure transfers are allowed again",
  (context) => {
    before("Try turning maintenance mode on", async () => {
      // go into Maintenance
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );
      // exit maintenance
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.resumeNormalOperation()
      );

      //try transfer
      await context.createBlock({
        transactions: [await createTransfer(context, TEST_ACCOUNT, 512)],
      });
    });

    it("shouldn't succeed with maintenance on", async function () {
      expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal("512");
    });
  }
);

describeDevMoonbeam("Pallet Maintenance Mode - normal exit call shouldnt work", (context) => {
  before("Try turning maintenance mode on", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // go into Maintenance
    await execFromAllMembersOfTechCommittee(
      context,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
    );
    // and try to turn it off
    await createBlockWithExtrinsic(
      context,
      alith,
      context.polkadotApi.tx.maintenanceMode.resumeNormalOperation()
    );
  });

  it("should fail without sudo", async function () {
    expect((await context.polkadotApi.query.maintenanceMode.maintenanceMode()).toHuman()).to.equal(
      true
    );
  });
});

// pallets that should be desactivated with maintenance mode

describeDevMoonbeam(
  "Pallet Maintenance Mode - no balance transfer with maintenance mode",
  (context) => {
    before("Try turning maintenance mode on", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );

      await context.createBlock({
        transactions: [await createTransfer(context, TEST_ACCOUNT, 512)],
      });
    });

    it("shouldn't succeed with maintenance on", async function () {
      expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal("0");
    });
  }
);

describeDevMoonbeam(
  "Pallet Maintenance Mode - evm transfer with maintenance mode works with sudo",
  (context) => {
    let events;
    before("Try turning maintenance mode on", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );

      ({ events } = await createBlockWithExtrinsic(
        context,
        alith,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.evm.call(
            ALITH,
            TEST_ACCOUNT,
            "0x0",
            100_000_000_000_000_000_000n,
            12_000_000n,
            1_000_000_000n,
            0n,
            undefined,
            []
          )
        )
      ));
    });

    it("should succeed with maintenance on but with sudo", async function () {
      expect(events[11].toHuman().method).to.eq("ExtrinsicSuccess");
      expect(await context.web3.eth.getBalance(TEST_ACCOUNT)).to.equal(
        100_000_000_000_000_000_000n.toString()
      );
    });
  }
);

describeDevMoonbeam(
  "Pallet Maintenance Mode - no crowdloanRewards claim with maintenance mode",
  (context) => {
    let genesisAccount;
    before("Try turning maintenance mode on", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");

      // turn maintenance on
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );

      //init
      await context.polkadotApi.tx.sudo
        .sudo(
          context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
            [relayChainAddress, GENESIS_ACCOUNT, 3_000_000n * GLMR],
          ])
        )
        .signAndSend(alith);
      await context.createBlock();

      let initBlock = (await context.polkadotApi.query.crowdloanRewards.initRelayBlock()) as any;

      // Complete initialization
      await context.polkadotApi.tx.sudo
        .sudo(
          context.polkadotApi.tx.crowdloanRewards.completeInitialization(
            initBlock.toBigInt() + VESTING_PERIOD
          )
        )
        .signAndSend(alith);
      await context.createBlock();
    });

    it("shouldn't succeed with maintenance on", async function () {
      const error = await expectError(async () => {
        await createBlockWithExtrinsic(
          context,
          genesisAccount,
          context.polkadotApi.tx.crowdloanRewards.claim()
        );
      });
      expect(error).to.eq("Error: 1010: Invalid Transaction: Transaction call is not expected");
    });
  }
);

describeDevMoonbeam(
  "Pallet Maintenance Mode - no assets transfer with maintenance mode",
  (context) => {
    let sudoAccount, assetId;
    before("Try turning maintenance mode on", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      // turn maintenance on
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );

      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = context.polkadotApi.createType("Balance", 100000000000000);
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });

      assetId = context.polkadotApi.createType(
        "u128",
        new BN("42259045809535163221576417993425387648")
      );
      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
      });

      await mockAssetBalance(context, assetBalance, assetDetails, sudoAccount, assetId, ALITH);
    });

    it("shouldn't succeed with maintenance on", async function () {
      const error = await expectError(async () => {
        await createBlockWithExtrinsic(
          context,
          sudoAccount,
          context.polkadotApi.tx.assets.transfer(assetId, BALTATHAR, 1000)
        );
      });
      expect(error).to.eq("Error: 1010: Invalid Transaction: Transaction call is not expected");
    });
  }
);

const HUNDRED_UNITS = 100000000000000;

describeDevMoonbeam(
  "Pallet Maintenance Mode - no xtokens transfer with maintenance mode",
  (context) => {
    let baltathar: KeyringPair;
    before("First send relay chain asset to parachain", async function () {
      const keyring = new Keyring({ type: "ethereum" });
      baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // turn maintenance on
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it("shouldn't succeed with maintenance on", async function () {
      const error = await expectError(async () => {
        await createBlockWithExtrinsic(
          context,
          baltathar,
          context.polkadotApi.tx.xTokens.transfer(
            "SelfReserve", //enum
            new BN(HUNDRED_UNITS),
            {
              V1: {
                parents: new BN(1),
                interior: {
                  X2: [
                    { Parachain: new BN(2000) },
                    { AccountKey20: { network: "Any", key: hexToU8a(BALTATHAR) } },
                  ],
                },
              },
            },
            new BN(4000000000)
          )
        );
      });
      expect(error).to.eq("Error: 1010: Invalid Transaction: Transaction call is not expected");
    });
  }
);

describeDevMoonbeam(
  "Pallet Maintenance Mode - no xcmTransactor transfer with maintenance mode",
  (context) => {
    let sudoAccount;
    before("First send relay chain asset to parachain", async function () {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      // turn maintenance on
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it("should succeed with maintenance on", async function () {
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(context.polkadotApi.tx.xcmTransactor.register(ALITH, 0))
      );
      const resp = await context.polkadotApi.query.xcmTransactor.indexToAccount(0);
      expect(resp.toString()).to.eq(ALITH);
    });
  }
);

describeDevMoonbeam(
  "Pallet Maintenance Mode - no xcmTransactor transfer with maintenance mode",
  (context) => {
    let sudoAccount;
    before("First send relay chain asset to parachain", async function () {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      // turn maintenance on
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it("should succeedn't with maintenance on", async function () {
      const error = await expectError(async () => {
        await createBlockWithExtrinsic(
          context,
          sudoAccount,
          context.polkadotApi.tx.xcmTransactor.transactThroughDerivative(
            "Relay",
            0,
            "SelfReserve",
            new BN(4000000000),
            []
          )
        );
      });
      expect(error).to.eq("Error: 1010: Invalid Transaction: Transaction call is not expected");
    });
  }
);

describeDevMoonbeam(
  "Pallet Maintenance Mode - dmp messages should queue in maintenance mode",
  (context) => {
    let sudoAccount, assetId;
    before("Register asset and go to maintenance", async function () {
      const assetMetadata = {
        name: "DOT",
        symbol: "DOT",
        decimals: new BN(12),
        isFrozen: false,
      };

      const sourceLocation = { XCM: { parents: 1, interior: "Here" } };

      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.registerAsset(
            sourceLocation,
            assetMetadata,
            new BN(1),
            true
          )
        )
      );
      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // setAssetUnitsPerSecond
      const { events } = await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 0, 0)
        )
      );
      expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
      expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

      // turn maintenance on
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it("should queue xcm with maintenance mode and execute when off", async function () {
      // Send RPC call to inject DMP message
      // You can provide a message, but if you don't a downward transfer is the default
      await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [[]]);

      // Create a block in which the XCM should be executed
      await context.createBlock();

      // Make sure the state does not have ALITH's DOT tokens
      let alithBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH)) as any;

      // Alith balance is 0
      expect(alithBalance.isNone).to.eq(true);

      // turn maintenance off
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.resumeNormalOperation()
      );

      // Create a block in which the XCM will be executed
      await context.createBlock();

      // Make sure the state has ALITH's to DOT tokens
      alithBalance = ((await context.polkadotApi.query.assets.account(assetId, ALITH)) as any)
        .unwrap()
        ["balance"].toBigInt();

      // Alith balance is 10 DOT
      expect(alithBalance).to.eq(BigInt(10000000000000));
    });
  }
);

describeDevMoonbeam(
  "Pallet Maintenance Mode - xcmp messages should queue in maintenance mode",
  (context) => {
    let sudoAccount, assetId, foreignParaId;
    before("Register asset and go to maintenance", async function () {
      foreignParaId = 2000;

      const assetMetadata = {
        name: "FOREIGN",
        symbol: "FOREIGN",
        decimals: new BN(12),
        isFrozen: false,
      };

      const sourceLocation = {
        XCM: { parents: 1, interior: { X1: { Parachain: foreignParaId } } },
      };

      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.registerAsset(
            sourceLocation,
            assetMetadata,
            new BN(1),
            true
          )
        )
      );
      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // setAssetUnitsPerSecond
      const { events } = await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 0, 0)
        )
      );
      expect(events[1].method.toString()).to.eq("UnitsPerSecondChanged");
      expect(events[4].method.toString()).to.eq("ExtrinsicSuccess");

      // turn maintenance on
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it("should queue xcm with maintenance mode and execute when off", async function () {
      // Send RPC call to inject XCMP message
      // You can provide a message, but if you don't a downward transfer is the default
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [foreignParaId, []]);

      // Create a block in which the XCM should be executed
      await context.createBlock();

      // Make sure the state does not have ALITH's foreign asset tokens
      let alithBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH)) as any;
      // Alith balance is 0
      expect(alithBalance.isNone).to.eq(true);

      // turn maintenance off
      await execFromAllMembersOfTechCommittee(
        context,
        context.polkadotApi.tx.maintenanceMode.resumeNormalOperation()
      );

      // Create a block in which the XCM will be executed
      await context.createBlock();

      // Make sure the state has ALITH's to foreign assets tokens
      alithBalance = (
        (await context.polkadotApi.query.assets.account(assetId, ALITH)) as any
      ).unwrap()["balance"];

      expect(alithBalance.toBigInt()).to.eq(BigInt(10000000000000));
    });
  }
);
