import "@moonbeam-network/api-augment";
import { ApiBase } from "@polkadot/api/base";

import {
  BN,
  bnToHex,
  hexToU8a,
  numberToHex,
  stringToHex,
  u8aToHex,
  u8aToString,
} from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";

import {
  alith,
  ALITH_ADDRESS,
  baltathar,
  BALTATHAR_ADDRESS,
  charleth,
  CHARLETH_ADDRESS,
  DOROTHY_ADDRESS,
} from "../../util/accounts";
import { registerLocalAssetWithMeta } from "../../util/assets";
import { getCompiled } from "../../util/contracts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";

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
const GAS_PRICE = "0x" + (10_000_000_000).toString(16);
const LOCAL_ASSET_EXTENDED_ERC20_CONTRACT = getCompiled("LocalAssetExtendedErc20Instance");
const ROLES_CONTRACT = getCompiled("precompiles/assets-erc20/Roles");

const LOCAL_ASSET_EXTENDED_ERC20_INTERFACE = new ethers.utils.Interface(
  LOCAL_ASSET_EXTENDED_ERC20_CONTRACT.contract.abi
);

const ROLES_INTERFACE = new ethers.utils.Interface(ROLES_CONTRACT.contract.abi);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetAddress: string;
    let assetId: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: baltathar, amount: 100000000000000n }],
      }));

      // Set team
      await context.createBlock(
        context.polkadotApi.tx.localAssets
          // Issuer, admin, freezer
          .setTeam(assetId, BALTATHAR_ADDRESS, CHARLETH_ADDRESS, DOROTHY_ADDRESS)
          .signAsync(baltathar)
      );

      // Set owner
      await context.createBlock(
        context.polkadotApi.tx.localAssets
          // owner
          .transferOwnership(assetId, ALITH_ADDRESS)
          .signAsync(baltathar)
      );

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });

    it("allows to call name", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "name",
        []
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
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
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "symbol",
        []
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
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
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "decimals",
        []
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
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
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "balanceOf",
        [baltathar.address]
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
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
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "totalSupply",
        []
      );
      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
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

    it("allows to call owner", async function () {
      const data = ROLES_INTERFACE.encodeFunctionData(
        // action
        "owner",
        []
      );
      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
          value: "0x0",
          gas: "0x10000",
          gasPrice: GAS_PRICE,
          to: assetAddress,
          data: data,
        },
      ]);

      const account = "0x" + ALITH_ADDRESS.slice(2).padStart(64, "0");
      expect(tx_call.result).equals(account.toLocaleLowerCase());
    });

    it("allows to call freezer", async function () {
      const data = ROLES_INTERFACE.encodeFunctionData(
        // action
        "freezer",
        []
      );
      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
          value: "0x0",
          gas: "0x10000",
          gasPrice: GAS_PRICE,
          to: assetAddress,
          data: data,
        },
      ]);

      const account = "0x" + DOROTHY_ADDRESS.slice(2).padStart(64, "0");
      expect(tx_call.result).equals(account.toLocaleLowerCase());
    });

    it("allows to call admin", async function () {
      const data = ROLES_INTERFACE.encodeFunctionData(
        // action
        "admin",
        []
      );
      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
          value: "0x0",
          gas: "0x10000",
          gasPrice: GAS_PRICE,
          to: assetAddress,
          data: data,
        },
      ]);

      const account = "0x" + CHARLETH_ADDRESS.slice(2).padStart(64, "0");
      expect(tx_call.result).equals(account.toLocaleLowerCase());
    });

    it("allows to call issuer", async function () {
      const data = ROLES_INTERFACE.encodeFunctionData(
        // action
        "issuer",
        []
      );
      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
          value: "0x0",
          gas: "0x10000",
          gasPrice: GAS_PRICE,
          to: assetAddress,
          data: data,
        },
      ]);

      const account = "0x" + baltathar.address.slice(2).padStart(64, "0");
      expect(tx_call.result).equals(account.toLocaleLowerCase());
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetAddress: string;
    let assetId: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: alith, amount: 100000000000000n }],
      }));

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to approve transfers, and allowance matches", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "approve",
        [baltathar.address, 1000]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);
      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);
      let approvals = await context.polkadotApi.query.localAssets.approvals(
        assetId,
        alith.address,
        baltathar.address
      );

      expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
    });
    it("should gather the allowance", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "allowance",
        [alith.address, baltathar.address]
      );

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
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
    let assetAddress: string;
    let assetId: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: baltathar, amount: 100000000000000n }],
      }));

      assetAddress = u8aToHex(new Uint8Array([...hexToU8a("0xFFFFFFFE"), ...hexToU8a(assetId)]));

      // Set metadata
      await context.createBlock(
        context.polkadotApi.tx.localAssets
          .setMetadata(assetId, "Local", "Local", new BN(12))
          .signAsync(baltathar)
      );

      // mint asset
      await context.createBlock(
        context.polkadotApi.tx.localAssets
          .mint(assetId, alith.address, 100000000000000)
          .signAsync(baltathar)
      );

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });

    it("allows to approve transfer and use transferFrom", async function () {
      // Create approval
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "approve",
        [baltathar.address, 1000]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      let approvals = await context.polkadotApi.query.localAssets.approvals(
        assetId,
        alith.address,
        baltathar.address
      );

      expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
      // We are gonna spend 1000 from alith to send it to charleth
      data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "transferFrom",
        [alith.address, charleth.address, 1000]
      );

      const { result: newResult } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );
      const receipt = await context.web3.eth.getTransactionReceipt(newResult.hash);

      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address.toLocaleLowerCase()).to.eq(assetAddress);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
      expect(receipt.status).to.equal(true);

      // Approve amount is null now
      approvals = await context.polkadotApi.query.localAssets.approvals(
        assetId,
        alith.address,
        baltathar.address
      );
      expect(approvals.isNone).to.eq(true);

      // Charleth balance is 1000
      let charletBalance = await context.polkadotApi.query.localAssets.account(
        assetId,
        charleth.address
      );
      expect(charletBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetAddress: string;
    let assetId: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: alith, amount: 100000000000000n }],
      }));

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to transfer", async function () {
      // Create approval
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "transfer",
        [baltathar.address, 1000]
      );
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
      expect(receipt.status).to.equal(true);

      // Baltathar balance is 1000
      let baltatharBalance = await context.polkadotApi.query.localAssets.account(
        assetId,
        baltathar.address
      );
      expect(baltatharBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes("Precompiles - Assets-ERC20 Wasm", (context) => {
  let assetId: string;
  let assetAddress: string;
  let contractInstanceAddress: string;
  before("Setup contract and mock balance", async () => {
    // register, setMeta & mint local Asset
    ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
      registrerAccount: baltathar,
    }));

    const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
    contractInstanceAddress = contract.options.address;
    await context.createBlock(rawTx);

    // before we mint asset, since these are non-sufficient, we need to transfer native balance
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(contractInstanceAddress, 1000).signAsync(baltathar)
    );

    // mint asset
    await context.createBlock(
      context.polkadotApi.tx.localAssets
        .mint(assetId, contractInstanceAddress, 100000000000000)
        .signAsync(baltathar)
    );
    // set asset address
    let setAddressData = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
      // action
      "set_address_interface",
      [context.web3.utils.toChecksumAddress(assetAddress)]
    );

    // We need this because the asset addres is random,
    // so we need a way to correctly reference it in the contract
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: contractInstanceAddress,
        data: setAddressData,
      })
    );
  });
  it("allows to approve transfer and use transferFrom from contract calls", async function () {
    // Create approval
    let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
      // action
      "approve",
      [baltathar.address, 1000]
    );

    await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: contractInstanceAddress,
        data: data,
      },
    ]);

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: contractInstanceAddress,
        data: data,
      })
    );

    let receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.status).to.equal(true);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);

    let approvals = await context.polkadotApi.query.localAssets.approvals(
      assetId,
      contractInstanceAddress,
      baltathar.address
    );

    expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
    // We are gonna spend 1000 from contractInstanceAddress to send it to charleth
    // Since this is a regular call, it will take contractInstanceAddress as msg.sender
    // thus from & to will be the same, and approval wont be touched
    data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
      // action
      "transferFrom",
      [contractInstanceAddress, charleth.address, 1000]
    );

    const { result: newResult } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: contractInstanceAddress,
        data: data,
      })
    );
    receipt = await context.web3.eth.getTransactionReceipt(newResult.hash);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(receipt.status).to.equal(true);

    // approvals are untouched
    approvals = await context.polkadotApi.query.localAssets.approvals(
      assetId,
      contractInstanceAddress,
      baltathar.address
    );
    expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);

    // this time we call directly from Baltathar the ERC20 contract
    const { result: baltatharResult } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: assetAddress,
        data: data,
      })
    );
    receipt = await context.web3.eth.getTransactionReceipt(baltatharResult.hash);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(receipt.status).to.equal(true);

    // Approve amount is null now
    approvals = await context.polkadotApi.query.localAssets.approvals(
      assetId,
      contractInstanceAddress,
      baltathar.address
    );
    expect(approvals.isNone).to.eq(true);

    // Charleth balance is 2000
    let charletBalance = await context.polkadotApi.query.localAssets.account(
      assetId,
      charleth.address
    );
    expect(charletBalance.unwrap()["balance"].eq(new BN(2000))).to.equal(true);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - Assets-ERC20 Wasm", (context) => {
  let assetId: string;
  let assetAddress: string;
  let contractInstanceAddress: string;
  before("Setup contract and mock balance", async () => {
    // register, setMeta & mint local Asset
    ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
      registrerAccount: baltathar,
      mints: [{ account: alith, amount: 100000000000000n }],
    }));

    const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
    contractInstanceAddress = contract.options.address;
    await context.createBlock(rawTx);
    // set asset address
    let setAddressData = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
      // action
      "set_address_interface",
      [context.web3.utils.toChecksumAddress(assetAddress)]
    );

    // We need this because the asset addres is random,
    // so we need a way to correctly reference it in the contract
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: contractInstanceAddress,
        data: setAddressData,
      })
    );
  });

  it("Bob approves contract and use transferFrom from contract calls", async function () {
    // Create approval
    let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
      // action
      "approve",
      [contractInstanceAddress, 1000]
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: assetAddress,
        data: data,
      })
    );

    let receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.status).to.equal(true);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);

    let approvals = await context.polkadotApi.query.localAssets.approvals(
      assetId,
      alith.address,
      contractInstanceAddress
    );

    expect(approvals.unwrap().amount.eq(new BN(1000))).to.equal(true);
    // We are gonna spend 1000 from alith.address to send it to charleth from contract address
    // even if Bob calls, msg.sender will become the contract with regular calls
    data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
      // action
      "transferFrom",
      [alith.address, charleth.address, 1000]
    );

    const { result: baltatharResult } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: contractInstanceAddress,
        data: data,
      })
    );
    receipt = await context.web3.eth.getTransactionReceipt(baltatharResult.hash);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(receipt.status).to.equal(true);

    // Approve amount is null now
    approvals = await context.polkadotApi.query.localAssets.approvals(
      assetId,
      alith.address,
      contractInstanceAddress
    );
    expect(approvals.isNone).to.eq(true);

    // Charleth balance is 1000
    let charletBalance = await context.polkadotApi.query.localAssets.account(
      assetId,
      charleth.address
    );
    expect(charletBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
  });
});

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    let contractInstanceAddress: string;
    before("Setup contract and mock balance", async () => {
      const { contract, rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      contractInstanceAddress = contract.options.address;
      await context.createBlock(rawTx);

      // before we mint asset, since these are non-sufficient, we need to transfer native balance
      await context.createBlock(
        context.polkadotApi.tx.balances.transfer(contractInstanceAddress, 1000).signAsync(baltathar)
      );

      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: contractInstanceAddress, amount: 100000000000000n }],
      }));

      // set asset address
      let setAddressData = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "set_address_interface",
        [context.web3.utils.toChecksumAddress(assetAddress)]
      );

      // We need this because the asset addres is random,
      // so we need a way to correctly reference it in the contract
      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: contractInstanceAddress,
          data: setAddressData,
        })
      );
    });
    it("allows to transfer through call from SC ", async function () {
      // Create approval
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "transfer",
        [baltathar.address, 1000]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: contractInstanceAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
      expect(receipt.status).to.equal(true);

      // Baltathar balance is 1000
      let baltatharBalance = await context.polkadotApi.query.localAssets.account(
        assetId,
        baltathar.address
      );
      expect(baltatharBalance.unwrap()["balance"].eq(new BN(1000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
      }));

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to mint", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "mint",
        [baltathar.address, 1000]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);
      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);

      let baltatharBalance = await context.polkadotApi.query.localAssets.account(
        assetId,
        baltathar.address
      );

      expect(baltatharBalance.unwrap().balance.toBigInt()).to.equal(1000n);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    let contractInstanceAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: alith, amount: 100000000000000n }],
      }));

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to burn", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "burn",
        [alith.address, 1000000000000]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);
      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address.toLowerCase()).to.eq(assetAddress);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);

      let alithBalance = await context.polkadotApi.query.localAssets.account(
        assetId,
        alith.address
      );

      expect(alithBalance.unwrap()["balance"].eq(new BN(99000000000000))).to.equal(true);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: alith, amount: 100000000000000n }],
      }));

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to freeze account", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "freeze",
        [alith.address]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);

      let alithFrozen = await context.polkadotApi.query.localAssets.account(assetId, alith.address);

      expect(alithFrozen.unwrap().isFrozen.isTrue).to.be.true;
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: alith, amount: 100000000000000n }],
      }));

      await context.createBlock(
        await context.polkadotApi.tx.localAssets.freeze(assetId, alith.address).signAsync(baltathar)
      );

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to thaw account", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "thaw",
        [alith.address]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);

      let baltatharFrozen = await context.polkadotApi.query.localAssets.account(
        assetId,
        alith.address
      );

      expect(baltatharFrozen.unwrap().isFrozen.isFalse).to.be.true;
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
      }));

      await context.createBlock(
        context.polkadotApi.tx.localAssets.freeze(assetId, alith.address).signAsync(baltathar)
      );

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to freeze an asset", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "freeze_asset"
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);

      const registeredAsset = (await context.polkadotApi.query.localAssets.asset(assetId)).unwrap();

      expect(registeredAsset.status.isFrozen).to.be.true;
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
      }));

      await context.createBlock(
        context.polkadotApi.tx.localAssets.freezeAsset(assetId).signAsync(baltathar)
      );

      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });

    it("allows to thaw an asset", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "thaw_asset"
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);

      const registeredAsset = (await context.polkadotApi.query.localAssets.asset(assetId)).unwrap();

      expect(registeredAsset.status.isFrozen).to.be.false;
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
      }));

      await context.createBlock(
        context.polkadotApi.tx.localAssets.freeze(assetId, alith.address).signAsync(baltathar)
      );
      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });

    it("allows to transfer ownership", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "transfer_ownership",
        [alith.address]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);

      const registeredAsset = (await context.polkadotApi.query.localAssets.asset(assetId)).unwrap();

      expect(registeredAsset.owner.toHex()).to.eq(alith.address.toLowerCase());
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
      }));
      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });

    it("allows to set team", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "set_team",
        [alith.address, alith.address, alith.address]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);

      const registeredAsset = (await context.polkadotApi.query.localAssets.asset(assetId)).unwrap();

      expect(registeredAsset.admin.toHex()).to.eq(alith.address.toLowerCase());
      expect(registeredAsset.freezer.toHex()).to.eq(alith.address.toLowerCase());
      expect(registeredAsset.issuer.toHex()).to.eq(alith.address.toLowerCase());
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: string;
    let assetAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
      }));
      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });

    it("allows to set metadata", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "set_metadata",
        ["Local", "LOC", 12]
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);

      const metadata = await context.polkadotApi.query.localAssets.metadata(assetId);

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
    let assetId: string;
    let assetAddress: string;
    before("Setup contract and mock balance", async () => {
      // register, setMeta & mint local Asset
      ({ assetId, assetAddress } = await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
      }));
      const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
      await context.createBlock(rawTx);
    });

    it("allows to clear metadata", async function () {
      let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
        // action
        "clear_metadata",
        []
      );

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: assetAddress,
          data: data,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);

      const metadata = await context.polkadotApi.query.localAssets.metadata(assetId);

      expect(u8aToString(metadata.name)).to.eq("");
      expect(u8aToString(metadata.symbol)).to.eq("");
      expect(metadata.decimals.toString()).to.eq("0");
    });
  },
  true
);

describeDevMoonbeam("Precompiles - Assets-ERC20 Wasm", (context) => {
  let assetAddress: string[] = [];
  before("Setup contract and mock balance", async () => {
    // register, setMeta & mint local Asset
    assetAddress[0] = (
      await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: alith, amount: 2n ** 128n - 3n }],
      })
    ).assetAddress;
    assetAddress[1] = (
      await registerLocalAssetWithMeta(context, alith, {
        registrerAccount: baltathar,
        mints: [{ account: alith, amount: 2n ** 128n - 3n }],
      })
    ).assetAddress;

    const { rawTx } = await createContract(context, "LocalAssetExtendedErc20Instance");
    await context.createBlock(rawTx);
  });

  it("succeeds to mint to 2^128 - 1", async function () {
    let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
      // action
      "mint",
      [baltathar.address, 2]
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: assetAddress[0],
        data: data,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.status).to.equal(true);
  });

  // Depends on previous test
  it("fails to mint over 2^128 total supply", async function () {
    let data = LOCAL_ASSET_EXTENDED_ERC20_INTERFACE.encodeFunctionData(
      // action
      "mint",
      [baltathar.address, 3]
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: assetAddress[1],
        data: data,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.status).to.equal(false);
  });
});
