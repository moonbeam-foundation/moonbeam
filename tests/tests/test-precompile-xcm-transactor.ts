import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";
import { ethers } from "ethers";
import { getCompiled } from "../util/contracts";
import { createContract, createTransaction } from "../util/transactions";
import { BN, hexToU8a, bnToHex, u8aToHex } from "@polkadot/util";
import Keyring from "@polkadot/keyring";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import { ALITH, ALITH_PRIV_KEY } from "../util/constants";
import { ApiPromise } from "@polkadot/api";
import { typesBundle } from "../../moonbeam-types-bundle";

const ADDRESS_XCM_TRANSACTOR = "0x0000000000000000000000000000000000000806";
const ADDRESS_RELAY_ASSETS = "0xffffffff1fcacbd218edc0eba20fc2308c778080";

const GAS_PRICE = "0x" + (1_000_000_000).toString(16);

async function mockAssetBalance(context, assetBalance, assetDetails, sudoAccount, assetId) {
  // Register the asset
  await context.polkadotApi.tx.sudo
    .sudo(
      context.polkadotApi.tx.assetManager.registerAsset(
        sourceLocationRelayAssetType,
        relayAssetMetadata,
        new BN(1)
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
    ...blake2AsU8a(hexToU8a(ALITH), 128),
    ...hexToU8a(ALITH),
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

const sourceLocationRelay = { parents: 1, interior: "Here" };

const sourceLocationRelayAssetType = { XCM: { parents: 1, interior: "Here" } };

describeDevMoonbeam("Precompiles - xcm transactor", (context) => {
  let sudoAccount, iFace, alith;
  before("Setup genesis account and relay accounts", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    // register index 0 for Alith
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.xcmTransactor.register(ALITH, 0))
      .signAndSend(sudoAccount);
    await context.createBlock();

    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.xcmTransactor.setTransactInfo(
          sourceLocationRelay,
          new BN(0),
          new BN(10000000000000)
        )
      )
      .signAndSend(sudoAccount);
    await context.createBlock();

    const contractData = await getCompiled("XcmTransactorInstance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context.web3, "XcmTransactorInstance");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });
    alith = keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });

  it("allows to retrieve index through precompiles", async function () {
    let data = iFace.encodeFunctionData(
      // action
      "index_to_account",
      [0]
    );
    let tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: ALITH,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_XCM_TRANSACTOR,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac"
    );
  });

  it("allows to retrieve transactor info through precompiles", async function () {
    let asset =
      // Destination as multilocation
      [
        // one parent
        1,
        [],
      ];
    let data = iFace.encodeFunctionData(
      // action
      "transact_info",
      [asset]
    );
    let tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: ALITH,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_XCM_TRANSACTOR,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000000" +
        "000000000000000000000000000000000000000000000000000009184e72a000"
    );
  });

  it("allows to issue transfer xcm transactor", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("AssetBalance", { balance: balance });

    const assetId = context.polkadotApi.createType(
      "AssetId",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails = context.polkadotApi.createType("AssetDetails", { supply: balance });

    await mockAssetBalance(context, assetBalance, assetDetails, sudoAccount, assetId);
    let beforeAssetBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH))
      .balance as BN;

    let beforeAssetDetails = (await context.polkadotApi.query.assets.asset(assetId)) as any;

    // supply and balance should be the same
    expect(beforeAssetBalance.eq(new BN(100000000000000))).to.equal(true);
    expect(beforeAssetDetails.unwrap()["supply"].eq(new BN(100000000000000))).to.equal(true);

    let transactor = 0;
    let index = 0;
    let asset =
      // Destination as multilocation
      [
        // one parent
        1,
        [],
      ];
    // we dont care, the call wont be executed
    let transact_call = new Uint8Array([0x01]);
    // weight
    let weight = 100;
    // Call the precompile
    let data = iFace.encodeFunctionData(
      // action
      "transact_through_derivative_multilocation",
      [transactor, index, asset, weight, transact_call]
    );

    const tx = await createTransaction(context.web3, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_XCM_TRANSACTOR,
      data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    // We have used 1000 units to pay for the fees in the relay, so balance and supply should
    // have changed
    let afterAssetBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH))
      .balance as BN;

    let expectedBalance = new BN(100000000000000).sub(new BN(1000));
    expect(afterAssetBalance.eq(expectedBalance)).to.equal(true);

    let AfterAssetDetails = (await context.polkadotApi.query.assets.asset(assetId)) as any;

    expect(AfterAssetDetails.unwrap()["supply"].eq(expectedBalance)).to.equal(true);
  });
});

describeDevMoonbeam("Precompiles - xcm transactor", (context) => {
  let sudoAccount, iFace, alith;
  before("Setup genesis account and relay accounts", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    // register index 0 for Alith
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.xcmTransactor.register(ALITH, 0))
      .signAndSend(sudoAccount);
    await context.createBlock();

    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.xcmTransactor.setTransactInfo(
          sourceLocationRelay,
          new BN(0),
          new BN(10000000000000)
        )
      )
      .signAndSend(sudoAccount);
    await context.createBlock();

    const contractData = await getCompiled("XcmTransactorInstance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context.web3, "XcmTransactorInstance");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });
    alith = keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });

  it("allows to issue transfer xcm transactor with currency Id", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay

    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("AssetBalance", { balance: balance });

    const assetId = context.polkadotApi.createType(
      "AssetId",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails = context.polkadotApi.createType("AssetDetails", { supply: balance });

    await mockAssetBalance(context, assetBalance, assetDetails, sudoAccount, assetId);

    let beforeAssetBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH))
      .balance as BN;

    let beforeAssetDetails = (await context.polkadotApi.query.assets.asset(assetId)) as any;

    // supply and balance should be the same
    expect(beforeAssetBalance.eq(new BN(100000000000000))).to.equal(true);
    expect(beforeAssetDetails.unwrap()["supply"].eq(new BN(100000000000000))).to.equal(true);

    let transactor = 0;
    let index = 0;
    // Destination as currency Id address
    let asset = ADDRESS_RELAY_ASSETS;
    // we dont care, the call wont be executed
    let transact_call = new Uint8Array([0x01]);
    // weight
    let weight = 100;
    // Call the precompile
    let data = iFace.encodeFunctionData(
      // action
      "transact_through_derivative",
      [transactor, index, asset, weight, transact_call]
    );

    const tx = await createTransaction(context.web3, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_XCM_TRANSACTOR,
      data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    // We have used 1000 units to pay for the fees in the relay, so balance and supply should
    // have changed
    let afterAssetBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH))
      .balance as BN;

    let expectedBalance = new BN(100000000000000).sub(new BN(1000));
    expect(afterAssetBalance.eq(expectedBalance)).to.equal(true);

    let AfterAssetDetails = (await context.polkadotApi.query.assets.asset(assetId)) as any;

    expect(AfterAssetDetails.unwrap()["supply"].eq(expectedBalance)).to.equal(true);
  });
});
