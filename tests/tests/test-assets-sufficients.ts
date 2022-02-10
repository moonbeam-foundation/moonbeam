import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { ALITH, BALTATHAR, ALITH_PRIV_KEY } from "../util/constants";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import { BN, hexToU8a, bnToHex, u8aToHex } from "@polkadot/util";
import Keyring from "@polkadot/keyring";

import { randomAsHex } from "@polkadot/util-crypto";

const sourceLocationRelay = { parents: 1, interior: "Here" };

const sourceLocationRelayAssetType = { XCM: { parents: 1, interior: "Here" } };

interface AssetMetadata {
  name: string;
  symbol: string;
  decimals: BN;
  isFrozen: boolean;
}

const relayAssetMetadata: AssetMetadata = {
  name: "DOT",
  symbol: "DOT",
  decimals: new BN(12),
  isFrozen: false,
};

async function mockAssetBalance(
  context,
  assetBalance,
  assetDetails,
  sudoAccount,
  assetId,
  account,
  is_sufficient
) {
  // Register the asset
  await context.polkadotApi.tx.sudo
    .sudo(
      context.polkadotApi.tx.assetManager.registerAsset(
        sourceLocationRelayAssetType,
        relayAssetMetadata,
        new BN(1),
        is_sufficient
      )
    )
    .signAndSend(sudoAccount);
  await context.createBlock();

  let assets = (
    (await context.polkadotApi.query.assetManager.assetIdType(assetId)) as any
  ).toJSON();
  // make sure we created it
  expect(assets["xcm"]["parents"]).to.equal(1);

  // Get keys to modify balance
  let module = xxhashAsU8a(new TextEncoder().encode("Assets"), 128);
  let account_key = xxhashAsU8a(new TextEncoder().encode("Account"), 128);
  let blake2concatAssetId = new Uint8Array([
    ...blake2AsU8a(assetId.toU8a(), 128),
    ...assetId.toU8a(),
  ]);

  let blake2concatAccount = new Uint8Array([
    ...blake2AsU8a(hexToU8a(account), 128),
    ...hexToU8a(account),
  ]);
  let overallAccountKey = new Uint8Array([
    ...module,
    ...account_key,
    ...blake2concatAssetId,
    ...blake2concatAccount,
  ]);

  // Get keys to modify total supply
  let assetKey = xxhashAsU8a(new TextEncoder().encode("Asset"), 128);
  let overallAssetKey = new Uint8Array([...module, ...assetKey, ...blake2concatAssetId]);
  await context.polkadotApi.tx.sudo
    .sudo(
      context.polkadotApi.tx.system.setStorage([
        [u8aToHex(overallAccountKey), u8aToHex(assetBalance.toU8a())],
        [u8aToHex(overallAssetKey), u8aToHex(assetDetails.toU8a())],
      ])
    )
    .signAndSend(sudoAccount);
  await context.createBlock();
  return;
}

describeDevMoonbeam(
  "Pallet Assets - Sufficient tests: is_sufficient to true",
  (context) => {
    let sudoAccount, assetId, iFace;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = new BN("100000000000000");
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });
      assetId = context.polkadotApi.createType(
        "u128",
        new BN("42259045809535163221576417993425387648")
      );

      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
        isSufficient: true,
        minBalance: 1,
      });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        sudoAccount,
        assetId,
        ALITH,
        true
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.assets.account(assetId, ALITH)) as any
      ).balance as BN;
      await context.createBlock();
      let alithBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH)) as any;
      expect(alithBalance.unwrap()["balance"].eq(new BN(100000000000000))).to.equal(true);
    });

    it("Send MOVR and assets to an account, then drain assets, then MOVR", async function () {
      // We are going to use a fresh account here, since we mocked the asset balance
      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      const seed = randomAsHex(32);
      const transferAmount = new BN("10000000000000");

      let freshAccount = await keyring.addFromUri(seed, null, "ethereum");

      // We transfer Assets to freshAccount, which should increase sufficients
      await context.polkadotApi.tx.assets
        .transfer(assetId, freshAccount.address, transferAmount)
        .signAndSend(alith);

      await context.createBlock();

      let freshAccountBalance = (await context.polkadotApi.query.assets.account(
        assetId,
        freshAccount.address
      )) as any;

      expect(freshAccountBalance.unwrap()["balance"].eq(new BN(10000000000000))).to.equal(true);

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(1n);
      // Providers should still be 0
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(0n);

      // Lets transfer it the native token. We want to transfer enough to cover for a future fee.
      const fee = (
        await context.polkadotApi.tx.assets
          .transfer(assetId, BALTATHAR, transferAmount)
          .paymentInfo(freshAccount)
      ).partialFee as any;

      // For some reason paymentInfo overestimates by 4359
      await context.polkadotApi.tx.balances
        .transfer(freshAccount.address, BigInt(fee) - BigInt(4359))
        .signAndSend(alith);
      await context.createBlock();

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(1n);
      // Providers should now be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // What happens now when we execute such transaction? both MOVR and Assets should be drained.
      await context.polkadotApi.tx.assets
        .transfer(assetId, BALTATHAR, transferAmount)
        .signAndSend(freshAccount);

      await context.createBlock();

      freshAccountBalance = (await context.polkadotApi.query.assets.account(
        assetId,
        freshAccount.address
      )) as any;
      expect(freshAccountBalance.isNone).to.equal(true);

      // Sufficients should go to 0
      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);
      // Providers should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // Nonce should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // But balance of MOVR should be 0
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).data.free.toBigInt()
      ).to.eq(0n);
    });
  },
  "Legacy",
  true
);

describeDevMoonbeam(
  "Pallet Assets - Sufficient tests: is_sufficient to true",
  (context) => {
    let sudoAccount, assetId, iFace;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = new BN("100000000000000");
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });
      assetId = context.polkadotApi.createType(
        "u128",
        new BN("42259045809535163221576417993425387648")
      );

      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
        isSufficient: true,
        minBalance: 1,
      });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        sudoAccount,
        assetId,
        ALITH,
        true
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.assets.account(assetId, ALITH)) as any
      ).balance as BN;
      await context.createBlock();
      let alithBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH)) as any;
      expect(alithBalance.unwrap()["balance"].eq(new BN(100000000000000))).to.equal(true);
    });

    it("Send MOVR and assets to an account, then drain assets, dont drain MOVR", async function () {
      // We are going to use a fresh account here, since we mocked the asset balance
      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      const seed = randomAsHex(32);
      const transferAmount = new BN("10000000000000");

      let freshAccount = await keyring.addFromUri(seed, null, "ethereum");

      // We transfer Assets to freshAccount, which should increase sufficients
      await context.polkadotApi.tx.assets
        .transfer(assetId, freshAccount.address, transferAmount)
        .signAndSend(alith);

      await context.createBlock();

      let freshAccountBalance = (await context.polkadotApi.query.assets.account(
        assetId,
        freshAccount.address
      )) as any;
      expect(freshAccountBalance.unwrap()["balance"].eq(new BN(10000000000000))).to.equal(true);

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(1n);
      // Providers should still be 0
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(0n);

      // Lets transfer it the native token. We want to transfer enough to cover for a future fee.
      const fee = (
        await context.polkadotApi.tx.assets
          .transfer(assetId, BALTATHAR, transferAmount)
          .paymentInfo(freshAccount)
      ).partialFee as any;

      await context.polkadotApi.tx.balances
        .transfer(freshAccount.address, BigInt(fee))
        .signAndSend(alith);
      await context.createBlock();

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(1n);
      // Providers should now be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // What happens now when we execute such transaction? both MOVR and Assets should be drained.
      await context.polkadotApi.tx.assets
        .transfer(assetId, BALTATHAR, transferAmount)
        .signAndSend(freshAccount);

      await context.createBlock();

      freshAccountBalance = (await context.polkadotApi.query.assets.account(
        assetId,
        freshAccount.address
      )) as any;
      expect(freshAccountBalance.isNone).to.equal(true);

      // Sufficients should go to 0
      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);
      // Providers should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // Nonce should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // But balance of MOVR should be 0
      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).data.free.toBigInt() > 0n
      ).to.eq(true);
    });
  },
  "Legacy",
  true
);

describeDevMoonbeam(
  "Pallet Assets - Sufficient tests: is_sufficient to false",
  (context) => {
    let sudoAccount, assetId, iFace;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = new BN("100000000000000");
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });
      assetId = context.polkadotApi.createType(
        "u128",
        new BN("42259045809535163221576417993425387648")
      );

      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
        isSufficient: false,
        minBalance: 1,
      });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        sudoAccount,
        assetId,
        ALITH,
        false
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.assets.account(assetId, ALITH)) as any
      ).balance as BN;
      await context.createBlock();
      let alithBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH)) as any;
      expect(alithBalance.unwrap()["balance"].eq(new BN(100000000000000))).to.equal(true);
    });

    it("Send MOVR and assets to an account, then drain assets, dont drain MOVR", async function () {
      // We are going to use a fresh account here, since we mocked the asset balance
      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      const seed = randomAsHex(32);
      const transferAmount = new BN("10000000000000");

      let freshAccount = await keyring.addFromUri(seed, null, "ethereum");

      // We cannot transfer to freshAccount, since sufficient is false
      await context.polkadotApi.tx.assets
        .transfer(assetId, freshAccount.address, transferAmount)
        .signAndSend(alith);

      await context.createBlock();

      let freshAccountBalance = (await context.polkadotApi.query.assets.account(
        assetId,
        freshAccount.address
      )) as any;
      expect(freshAccountBalance.isNone).to.equal(true);

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);
      // Providers should still be 0
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(0n);

      // Lets transfer it the native token. We want to transfer enough to cover for a future fee.
      const fee = (
        await context.polkadotApi.tx.assets
          .transfer(assetId, BALTATHAR, transferAmount)
          .paymentInfo(freshAccount)
      ).partialFee as any;

      // We transfer Balances, which should increase provider
      await context.polkadotApi.tx.balances
        .transfer(freshAccount.address, BigInt(fee))
        .signAndSend(alith);
      await context.createBlock();

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);
      // Providers should now be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // We now can transfer assets to freshAccount, since it has a provider
      await context.polkadotApi.tx.assets
        .transfer(assetId, freshAccount.address, transferAmount)
        .signAndSend(alith);

      await context.createBlock();

      freshAccountBalance = (await context.polkadotApi.query.assets.account(
        assetId,
        freshAccount.address
      )) as any;
      expect(freshAccountBalance.unwrap()["balance"].eq(transferAmount)).to.equal(true);

      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);

      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).consumers.toBigInt()
      ).to.eq(1n);

      // What happens now when we execute such transaction? both MOVR and Assets should be drained.
      await context.polkadotApi.tx.assets
        .transfer(assetId, BALTATHAR, transferAmount)
        .signAndSend(freshAccount);

      await context.createBlock();

      freshAccountBalance = (await context.polkadotApi.query.assets.account(
        assetId,
        freshAccount.address
      )) as any;
      expect(freshAccountBalance.isNone).to.equal(true);

      // Sufficients should be 0
      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).sufficients.toBigInt()
      ).to.eq(0n);

      // Consumers should be 0
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).consumers.toBigInt()
      ).to.eq(0n);

      // Providers should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // Nonce should be 1
      expect(
        (await context.polkadotApi.query.system.account(freshAccount.address)).providers.toBigInt()
      ).to.eq(1n);

      // But balance of MOVR should be 0
      expect(
        (
          await context.polkadotApi.query.system.account(freshAccount.address)
        ).data.free.toBigInt() > 0n
      ).to.eq(true);
    });
  },
  "Legacy",
  true
);
