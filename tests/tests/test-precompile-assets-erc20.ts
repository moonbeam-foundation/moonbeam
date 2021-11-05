import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";
import {
  GENESIS_ACCOUNT,
  ALITH,
  BALTATHAR,
  ALITH_PRIV_KEY,
  CHARLETH,
  BALTATHAR_PRIV_KEY,
} from "../util/constants";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import { BN, hexToU8a, bnToHex, u8aToHex } from "@polkadot/util";
import Keyring from "@polkadot/keyring";
import { getCompiled } from "../util/contracts";
import { ethers } from "ethers";
import { createContract, createTransaction } from "../util/transactions";

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

const ADDRESS_ERC20 = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
const SELECTORS = {
  balanceOf: "70a08231",
  totalSupply: "18160ddd",
  approve: "095ea7b3",
  allowance: "dd62ed3e",
  transfer: "a9059cbb",
  transferFrom: "23b872dd",
  logApprove: "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925",
  logTransfer: "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
};
const GAS_PRICE = "0x" + (1_000_000_000).toString(16);

describeDevMoonbeam("Precompiles - ERC20 Native", (context) => {
  let sudoAccount, assetId, iFace;
  before("Setup genesis account and relay accounts", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
        // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("AssetBalance", { balance: balance });

    assetId = context.polkadotApi.createType(
      "AssetId",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails = context.polkadotApi.createType("AssetDetails", { supply: balance });

    await mockAssetBalance(context, assetBalance, assetDetails, sudoAccount, assetId);

    let beforeAssetBalance = (await context.polkadotApi.query.assets.account(assetId, ALITH) as any)
      .balance as BN;
    
    const contractData = await getCompiled("ERC20Instance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context.web3, "ERC20Instance");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });

  });
  it("allows to call getBalance", async function () {
    let data = iFace.encodeFunctionData(
      // action
      "balanceOf",
      [ALITH]
    );


    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_ERC20,
        data: data,
      },
    ]);
    let amount = new BN(100000000000000);

    let amount_hex = "0x" + bnToHex(amount).slice(2).padStart(64, "0");
    expect(tx_call.result).equals(amount_hex);
  });

  it("allows to call totalSupply", async function () {
    let data = iFace.encodeFunctionData(
      // action
      "totalSupply",
      []
    );
    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_ERC20,
        data: data,
      },
    ]);

    let amount = new BN(100000000000000);

    let amount_hex = "0x" + bnToHex(amount).slice(2).padStart(64, "0");
    expect(tx_call.result).equals(amount_hex);
  });
});

describeDevMoonbeam("Precompiles - ERC20 Native", (context) => {
  let sudoAccount, assetId, iFace;
  before("Setup genesis account and relay accounts", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
        // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("AssetBalance", { balance: balance });

    assetId = context.polkadotApi.createType(
      "AssetId",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails = context.polkadotApi.createType("AssetDetails", { supply: balance });

    await mockAssetBalance(context, assetBalance, assetDetails, sudoAccount, assetId);
    
    const contractData = await getCompiled("ERC20Instance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context.web3, "ERC20Instance");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });

  });
  it("allows to approve transfers, and allowance matches", async function () {
    let data = iFace.encodeFunctionData(
      // action
      "approve",
      [BALTATHAR, 1000]
    );

    const tx = await createTransaction(context.web3, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_ERC20,
      data: data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

  const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
  
  expect(receipt.status).to.equal(true);
  expect(receipt.logs.length).to.eq(1);
  expect(receipt.logs[0].address).to.eq(ADDRESS_ERC20);
  expect(receipt.logs[0].topics.length).to.eq(3);
  expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);
  let approvals = (await context.polkadotApi.query.assets.approvals(assetId, ALITH, BALTATHAR)) as any;
    
  expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
  });
}, true);

