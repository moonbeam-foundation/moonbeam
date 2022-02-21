import { expect } from "chai";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { customWeb3Request } from "../../util/providers";
import { createBlockWithExtrinsic } from "../../util/substrate-rpc";

import {
  GENESIS_ACCOUNT,
  ALITH,
  BALTATHAR,
  ALITH_PRIV_KEY,
  CHARLETH,
  BALTATHAR_PRIV_KEY,
} from "../../util/constants";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import {
  BN,
  hexToU8a,
  bnToHex,
  u8aToHex,
  stringToHex,
  numberToHex,
  u8aToString,
} from "@polkadot/util";
import Keyring from "@polkadot/keyring";
import { getCompiled } from "../../util/contracts";
import { ethers } from "ethers";
import { createContract, createTransaction } from "../../util/transactions";

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

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress, contractInstanceAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
      );

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, baltatharAccount.address, 100000000000000)
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any
      ).balance as BN;
      let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
        .balance as BN;
      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      contractInstanceAddress = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });

    it("allows to call name", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "name",
        []
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: ALITH,
          value: "0x0",
          gas: "0x10000",
          gasPrice: GAS_PRICE,
          to: assetAddress,
          data: data,
        },
      ]);

      let expected = stringToHex("Local");
      let offset = numberToHex(32).slice(2).padStart(64, "0");
      let length = numberToHex(5).slice(2).padStart(64, "0");
      // Bytes are padded at the end
      let expected_hex = expected.slice(2).padEnd(64, "0");
      expect(tx_call.result).equals("0x" + offset + length + expected_hex);
    });

    it("allows to call symbol", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "symbol",
        []
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: GENESIS_ACCOUNT,
          value: "0x0",
          gas: "0x10000",
          gasPrice: GAS_PRICE,
          to: assetAddress,
          data: data,
        },
      ]);

      let expected = stringToHex("Local");
      let offset = numberToHex(32).slice(2).padStart(64, "0");
      let length = numberToHex(5).slice(2).padStart(64, "0");
      // Bytes are padded at the end
      let expected_hex = expected.slice(2).padEnd(64, "0");
      expect(tx_call.result).equals("0x" + offset + length + expected_hex);
    });

    it("allows to call decimals", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "decimals",
        []
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: GENESIS_ACCOUNT,
          value: "0x0",
          gas: "0x10000",
          gasPrice: GAS_PRICE,
          to: assetAddress,
          data: data,
        },
      ]);

      let expected = "0x" + numberToHex(12).slice(2).padStart(64, "0");
      expect(tx_call.result).equals(expected);
    });

    it("allows to call getBalance", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "balanceOf",
        [BALTATHAR]
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: GENESIS_ACCOUNT,
          value: "0x0",
          gas: "0x10000",
          gasPrice: GAS_PRICE,
          to: assetAddress,
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
          to: assetAddress,
          data: data,
        },
      ]);

      let amount = new BN(100000000000000);

      let amount_hex = "0x" + bnToHex(amount).slice(2).padStart(64, "0");
      expect(tx_call.result).equals(amount_hex);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
      );

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any
      ).balance as BN;
      let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
        .balance as BN;
      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to approve transfers, and allowance matches", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "approve",
        [BALTATHAR, 1000]
      );

      const tx = await createTransaction(context, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);
      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);
      let approvals = (await context.polkadotApi.query.localAssets.approvals(
        assetId,
        ALITH,
        BALTATHAR
      )) as any;

      expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
    });
    it("should gather the allowance", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "allowance",
        [ALITH, BALTATHAR]
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: GENESIS_ACCOUNT,
          value: "0x0",
          gas: "0x10000",
          gasPrice: GAS_PRICE,
          to: assetAddress,
          data: data,
        },
      ]);
      let amount = new BN(1000);

      let amount_hex = "0x" + bnToHex(amount).slice(2).padStart(64, "0");
      expect(tx_call.result).equals(amount_hex);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
      );

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any
      ).balance as BN;
      let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
        .balance as BN;
      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to approve transfer and use transferFrom", async function () {
      // Create approval
      let data = iFace.encodeFunctionData(
        // action
        "approve",
        [BALTATHAR, 1000]
      );

      let tx = await createTransaction(context, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      let block = await context.createBlock({
        transactions: [tx],
      });

      let approvals = (await context.polkadotApi.query.localAssets.approvals(
        assetId,
        ALITH,
        BALTATHAR
      )) as any;

      expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
      // We are gonna spend 1000 from alith to send it to charleth
      data = iFace.encodeFunctionData(
        // action
        "transferFrom",
        [ALITH, CHARLETH, 1000]
      );

      tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      block = await context.createBlock({
        transactions: [tx],
      });
      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address.toLocaleLowerCase()).to.eq(assetAddress);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
      expect(receipt.status).to.equal(true);

      // Approve amount is null now
      approvals = (await context.polkadotApi.query.localAssets.approvals(
        assetId,
        ALITH,
        BALTATHAR
      )) as any;
      expect(approvals.isNone).to.eq(true);

      // Charleth balance is 1000
      let charletBalance = (await context.polkadotApi.query.localAssets.account(
        assetId,
        CHARLETH
      )) as any;
      expect(charletBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
      );

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any
      ).balance as BN;
      let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
        .balance as BN;
      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to transfer", async function () {
      // Create approval
      let data = iFace.encodeFunctionData(
        // action
        "transfer",
        [BALTATHAR, 1000]
      );

      let tx = await createTransaction(context, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      let block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
      expect(receipt.status).to.equal(true);

      // Baltathar balance is 1000
      let baltatharBalance = (await context.polkadotApi.query.localAssets.account(
        assetId,
        BALTATHAR
      )) as any;
      expect(baltatharBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes("Precompiles - Assets-ERC20 Wasm", (context) => {
  let sudoAccount, baltatharAccount, assetId, iFace, assetAddress, contractInstanceAddress;
  before("Setup contract and mock balance", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

    // registerAsset
    await createBlockWithExtrinsic(
      context,
      sudoAccount,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.authorizeLocalAssset(
          baltatharAccount.address,
          baltatharAccount.address,
          new BN(1)
        )
      )
    );

    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.assetManager.registerLocalAsset()
    );

    // Look for assetId in events
    eventsRegister.forEach((e) => {
      if (e.section.toString() === "assetManager") {
        assetId = e.data[0].toHex();
      }
    });
    assetId = assetId.replace(/,/g, "");

    // registerAsset
    await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
    );

    // mint asset
    await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
    );

    let beforeAssetBalance = (
      (await context.polkadotApi.query.localAssets.account(assetId, ALITH)) as any
    ).balance as BN;
    let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
      .balance as BN;
    assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

    const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
    contractInstanceAddress = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });

    // set asset address
    let setAddressData = iFace.encodeFunctionData(
      // action
      "set_address_interface",
      [context.web3.utils.toChecksumAddress(assetAddress)]
    );

    // We need this because the asset addres is random, so we need a way to correctly reference it in the contract
    let setDataTx = await createTransaction(context, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: contractInstanceAddress,
      data: setAddressData,
    });

    await context.createBlock({
      transactions: [setDataTx],
    });
  });
  it("allows to approve transfer and use transferFrom through delegateCalls", async function () {
    // Create approval
    let data = iFace.encodeFunctionData(
      // action
      "approve_delegate",
      [BALTATHAR, 1000]
    );

    let tx = await createTransaction(context, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: contractInstanceAddress,
      data: data,
    });

    let block = await context.createBlock({
      transactions: [tx],
    });

    let receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

    expect(receipt.status).to.equal(true);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address).to.eq(contractInstanceAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);

    let approvals = (await context.polkadotApi.query.localAssets.approvals(
      assetId,
      ALITH,
      BALTATHAR
    )) as any;

    expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
    // We are gonna spend 1000 from alith to send it to charleth
    data = iFace.encodeFunctionData(
      // action
      "transferFrom_delegate",
      [ALITH, CHARLETH, 1000]
    );

    tx = await createTransaction(context, {
      from: BALTATHAR,
      privateKey: BALTATHAR_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: contractInstanceAddress,
      data: data,
    });

    block = await context.createBlock({
      transactions: [tx],
    });
    receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address).to.eq(contractInstanceAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(receipt.status).to.equal(true);

    // Approve amount is null now
    approvals = (await context.polkadotApi.query.localAssets.approvals(
      assetId,
      ALITH,
      BALTATHAR
    )) as any;
    expect(approvals.isNone).to.eq(true);

    // Charleth balance is 1000
    let charletBalance = (await context.polkadotApi.query.localAssets.account(
      assetId,
      CHARLETH
    )) as any;
    expect(charletBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
  });
});

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress, contractInstanceAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
      );

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.localAssets.account(assetId, ALITH)) as any
      ).balance as BN;
      let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
        .balance as BN;
      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      contractInstanceAddress = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
      // set asset address
      let setAddressData = iFace.encodeFunctionData(
        // action
        "set_address_interface",
        [context.web3.utils.toChecksumAddress(assetAddress)]
      );

      // We need this because the asset addres is random, so we need a way to correctly reference it in the contract
      let setDataTx = await createTransaction(context, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: contractInstanceAddress,
        data: setAddressData,
      });

      await context.createBlock({
        transactions: [setDataTx],
      });
    });
    it("allows to transfer through delegateCall", async function () {
      // Create approval
      let data = iFace.encodeFunctionData(
        // action
        "transfer_delegate",
        [BALTATHAR, 1000]
      );

      let tx = await createTransaction(context, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: contractInstanceAddress,
        data: data,
      });

      let block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
      expect(receipt.status).to.equal(true);

      // Baltathar balance is 1000
      let baltatharBalance = (await context.polkadotApi.query.localAssets.account(
        assetId,
        BALTATHAR
      )) as any;
      expect(baltatharBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes("Precompiles - Assets-ERC20 Wasm", (context) => {
  let sudoAccount, baltatharAccount, assetId, iFace, assetAddress, contractInstanceAddress;
  before("Setup contract and mock balance", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

    // registerAsset
    await createBlockWithExtrinsic(
      context,
      sudoAccount,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.authorizeLocalAssset(
          baltatharAccount.address,
          baltatharAccount.address,
          new BN(1)
        )
      )
    );

    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.assetManager.registerLocalAsset()
    );

    // Look for assetId in events
    eventsRegister.forEach((e) => {
      if (e.section.toString() === "assetManager") {
        assetId = e.data[0].toHex();
      }
    });
    assetId = assetId.replace(/,/g, "");

    // registerAsset
    await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
    );

    assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

    const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
    contractInstanceAddress = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });

    // before we mint asset, since these are non-sufficient, we need to transfer native balance
    await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.balances.transfer(contractInstanceAddress, 1000)
    );

    // mint asset
    await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.localAssets.mint(assetId, contractInstanceAddress, 100000000000000)
    );
    // set asset address
    let setAddressData = iFace.encodeFunctionData(
      // action
      "set_address_interface",
      [context.web3.utils.toChecksumAddress(assetAddress)]
    );

    // We need this because the asset addres is random, so we need a way to correctly reference it in the contract
    let setDataTx = await createTransaction(context, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: contractInstanceAddress,
      data: setAddressData,
    });

    await context.createBlock({
      transactions: [setDataTx],
    });
  });
  it("allows to approve transfer and use transferFrom from contract calls", async function () {
    // Create approval
    let data = iFace.encodeFunctionData(
      // action
      "approve",
      [BALTATHAR, 1000]
    );

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: ALITH,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: contractInstanceAddress,
        data: data,
      },
    ]);

    let tx = await createTransaction(context, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: contractInstanceAddress,
      data: data,
    });

    let block = await context.createBlock({
      transactions: [tx],
    });

    let receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

    expect(receipt.status).to.equal(true);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);

    let approvals = (await context.polkadotApi.query.localAssets.approvals(
      assetId,
      contractInstanceAddress,
      BALTATHAR
    )) as any;

    expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
    // We are gonna spend 1000 from contractInstanceAddress to send it to charleth
    // Since this is a regular call, it will take contractInstanceAddress as msg.sender
    // thus from & to will be the same, and approval wont be touched
    data = iFace.encodeFunctionData(
      // action
      "transferFrom",
      [contractInstanceAddress, CHARLETH, 1000]
    );

    tx = await createTransaction(context, {
      from: BALTATHAR,
      privateKey: BALTATHAR_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: contractInstanceAddress,
      data: data,
    });
    block = await context.createBlock({
      transactions: [tx],
    });
    receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(receipt.status).to.equal(true);

    // approvals are untouched
    approvals = (await context.polkadotApi.query.localAssets.approvals(
      assetId,
      contractInstanceAddress,
      BALTATHAR
    )) as any;
    expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);

    // this time we call directly from Baltathar the ERC20 contract
    tx = await createTransaction(context, {
      from: BALTATHAR,
      privateKey: BALTATHAR_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: assetAddress,
      data: data,
    });
    block = await context.createBlock({
      transactions: [tx],
    });
    receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(receipt.status).to.equal(true);

    // Approve amount is null now
    approvals = (await context.polkadotApi.query.localAssets.approvals(
      assetId,
      contractInstanceAddress,
      BALTATHAR
    )) as any;
    expect(approvals.isNone).to.eq(true);

    // Charleth balance is 2000
    let charletBalance = (await context.polkadotApi.query.localAssets.account(
      assetId,
      CHARLETH
    )) as any;
    expect(charletBalance.unwrap()["balance"].eq(new BN(2000))).to.equal(true);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - Assets-ERC20 Wasm", (context) => {
  let sudoAccount, baltatharAccount, assetId, iFace, assetAddress, contractInstanceAddress;
  before("Setup contract and mock balance", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

    // registerAsset
    await createBlockWithExtrinsic(
      context,
      sudoAccount,
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.assetManager.authorizeLocalAssset(
          baltatharAccount.address,
          baltatharAccount.address,
          new BN(1)
        )
      )
    );

    // registerAsset
    const { events: eventsRegister } = await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.assetManager.registerLocalAsset()
    );

    // Look for assetId in events
    eventsRegister.forEach((e) => {
      if (e.section.toString() === "assetManager") {
        assetId = e.data[0].toHex();
      }
    });
    assetId = assetId.replace(/,/g, "");

    // registerAsset
    await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
    );

    // mint asset
    await createBlockWithExtrinsic(
      context,
      baltatharAccount,
      context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
    );

    let beforeAssetBalance = (
      (await context.polkadotApi.query.localAssets.account(assetId, ALITH)) as any
    ).balance as BN;
    let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
      .balance as BN;
    assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

    const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
    contractInstanceAddress = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });
    // set asset address
    let setAddressData = iFace.encodeFunctionData(
      // action
      "set_address_interface",
      [context.web3.utils.toChecksumAddress(assetAddress)]
    );

    // We need this because the asset addres is random, so we need a way to correctly reference it in the contract
    let setDataTx = await createTransaction(context, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: contractInstanceAddress,
      data: setAddressData,
    });

    await context.createBlock({
      transactions: [setDataTx],
    });
  });
  it("Bob approves contract and use transferFrom from contract calls", async function () {
    // Create approval
    let data = iFace.encodeFunctionData(
      // action
      "approve",
      [contractInstanceAddress, 1000]
    );

    let tx = await createTransaction(context, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: assetAddress,
      data: data,
    });

    let block = await context.createBlock({
      transactions: [tx],
    });

    let receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

    expect(receipt.status).to.equal(true);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);

    let approvals = (await context.polkadotApi.query.localAssets.approvals(
      assetId,
      ALITH,
      contractInstanceAddress
    )) as any;

    expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
    // We are gonna spend 1000 from ALITH to send it to charleth from contract address
    // even if Bob calls, msg.sender will become the contract with regular calls
    data = iFace.encodeFunctionData(
      // action
      "transferFrom",
      [ALITH, CHARLETH, 1000]
    );

    tx = await createTransaction(context, {
      from: BALTATHAR,
      privateKey: BALTATHAR_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: contractInstanceAddress,
      data: data,
    });
    block = await context.createBlock({
      transactions: [tx],
    });
    receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(receipt.status).to.equal(true);

    // Approve amount is null now
    approvals = (await context.polkadotApi.query.localAssets.approvals(
      assetId,
      ALITH,
      contractInstanceAddress
    )) as any;
    expect(approvals.isNone).to.eq(true);

    // Charleth balance is 1000
    let charletBalance = (await context.polkadotApi.query.localAssets.account(
      assetId,
      CHARLETH
    )) as any;
    expect(charletBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
  });
});

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress, contractInstanceAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
      );

      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      contractInstanceAddress = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });

      // before we mint asset, since these are non-sufficient, we need to transfer native balance
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.balances.transfer(contractInstanceAddress, 1000)
      );

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, contractInstanceAddress, 100000000000000)
      );
      // set asset address
      let setAddressData = iFace.encodeFunctionData(
        // action
        "set_address_interface",
        [context.web3.utils.toChecksumAddress(assetAddress)]
      );

      // We need this because the asset addres is random, so we need a way to correctly reference it in the contract
      let setDataTx = await createTransaction(context, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: contractInstanceAddress,
        data: setAddressData,
      });

      await context.createBlock({
        transactions: [setDataTx],
      });
    });
    it("allows to transfer through call from SC ", async function () {
      // Create approval
      let data = iFace.encodeFunctionData(
        // action
        "transfer",
        [BALTATHAR, 1000]
      );

      let tx = await createTransaction(context, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: contractInstanceAddress,
        data: data,
      });

      let block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
      expect(receipt.status).to.equal(true);

      // Baltathar balance is 1000
      let baltatharBalance = (await context.polkadotApi.query.localAssets.account(
        assetId,
        BALTATHAR
      )) as any;
      expect(baltatharBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
      );

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any
      ).balance as BN;
      let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
        .balance as BN;
      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to mint", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "mint",
        [BALTATHAR, 1000]
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);
      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);

      let baltatharBalance = (await context.polkadotApi.query.localAssets.account(
        assetId,
        BALTATHAR
      )) as any;

      expect(baltatharBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
      );

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any
      ).balance as BN;
      let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
        .balance as BN;
      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to burn", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "burn",
        [ALITH, 1000000000000]
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);
      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);

      let alithBalance = (await context.polkadotApi.query.localAssets.account(
        assetId,
        ALITH
      )) as any;

      expect(alithBalance.unwrap()["balance"].eq(new BN(99000000000000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any
      ).balance as BN;
      let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
        .balance as BN;
      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to freeze account", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "freeze",
        [ALITH]
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);

      let alithFrozen = (await context.polkadotApi.query.localAssets.account(
        assetId,
        ALITH
      )) as any;

      expect(alithFrozen.unwrap()["isFrozen"].toHuman()).to.be.true;
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.freeze(assetId, sudoAccount.address)
      );

      let beforeAssetBalance = (
        (await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any
      ).balance as BN;
      let sup = ((await context.polkadotApi.query.localAssets.account(assetId, BALTATHAR)) as any)
        .balance as BN;
      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to thaw account", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "thaw",
        [ALITH]
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);

      let baltatharFrozen = (await context.polkadotApi.query.localAssets.account(
        assetId,
        ALITH
      )) as any;

      expect(baltatharFrozen.unwrap()["isFrozen"].toHuman()).to.be.false;
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.freeze(assetId, sudoAccount.address)
      );

      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to freeze an asset", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "freeze_asset"
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);

      const registeredAsset = (
        (await context.polkadotApi.query.localAssets.asset(assetId)) as any
      ).unwrap();

      expect(registeredAsset.isFrozen.toHuman()).to.be.true;
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // mint asset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.mint(assetId, sudoAccount.address, 100000000000000)
      );

      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.freezeAsset(assetId)
      );

      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to thaw an asset", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "thaw_asset"
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);

      const registeredAsset = (
        (await context.polkadotApi.query.localAssets.asset(assetId)) as any
      ).unwrap();

      expect(registeredAsset.isFrozen.toHuman()).to.be.false;
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to transfer ownership", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "transfer_ownership",
        [ALITH]
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);

      const registeredAsset = (
        (await context.polkadotApi.query.localAssets.asset(assetId)) as any
      ).unwrap();

      expect(registeredAsset.owner.toHex()).to.eq(ALITH.toLowerCase());
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to set team", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "set_team",
        [ALITH, ALITH, ALITH]
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);

      const registeredAsset = (
        (await context.polkadotApi.query.localAssets.asset(assetId)) as any
      ).unwrap();

      expect(registeredAsset.admin.toHex()).to.eq(ALITH.toLowerCase());
      expect(registeredAsset.freezer.toHex()).to.eq(ALITH.toLowerCase());
      expect(registeredAsset.issuer.toHex()).to.eq(ALITH.toLowerCase());
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to set metadata", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "set_metadata",
        ["Local", "LOC", 12]
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);

      const metadata = (await context.polkadotApi.query.localAssets.metadata(assetId)) as any;

      expect(u8aToString(metadata.name)).to.eq("Local");
      expect(u8aToString(metadata.symbol)).to.eq("LOC");
      expect(metadata.decimals.toString()).to.eq("12");
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let sudoAccount, baltatharAccount, assetId, iFace, assetAddress;
    before("Setup contract and mock balance", async () => {
      const keyring = new Keyring({ type: "ethereum" });
      sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      baltatharAccount = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        sudoAccount,
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.assetManager.authorizeLocalAssset(
            baltatharAccount.address,
            baltatharAccount.address,
            new BN(1)
          )
        )
      );

      // registerAsset
      const { events: eventsRegister } = await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.assetManager.registerLocalAsset()
      );

      // Look for assetId in events
      eventsRegister.forEach((e) => {
        if (e.section.toString() === "assetManager") {
          assetId = e.data[0].toHex();
        }
      });
      assetId = assetId.replace(/,/g, "");

      // registerAsset
      await createBlockWithExtrinsic(
        context,
        baltatharAccount,
        context.polkadotApi.tx.localAssets.setMetadata(assetId, "Local", "Local", new BN(12))
      );

      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      const contractData = await getCompiled("LocalAssetExtendedErc20Instance");
      iFace = new ethers.utils.Interface(contractData.contract.abi);
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      const address = contract.options.address;
      await context.createBlock({ transactions: [rawTx] });
    });
    it("allows to clear metadata", async function () {
      let data = iFace.encodeFunctionData(
        // action
        "clear_metadata",
        []
      );

      const tx = await createTransaction(context, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: assetAddress,
        data: data,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      expect(receipt.status).to.equal(true);

      const metadata = (await context.polkadotApi.query.localAssets.metadata(assetId)) as any;

      expect(u8aToString(metadata.name)).to.eq("");
      expect(u8aToString(metadata.symbol)).to.eq("");
      expect(metadata.decimals.toString()).to.eq("0");
    });
  },
  true
);
