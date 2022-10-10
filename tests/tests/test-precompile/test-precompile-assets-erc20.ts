import "@moonbeam-network/api-augment";

import { u128 } from "@polkadot/types";
import { BN, bnToHex, numberToHex, stringToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";

import { alith, baltathar, charleth } from "../../util/accounts";
import { mockAssetBalance } from "../../util/assets";
import { getCompiled } from "../../util/contracts";
import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";

const ADDRESS_ERC20 = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
const ASSET_ID = new BN("42259045809535163221576417993425387648");
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

const ERC20_CONTRACT = getCompiled("ERC20Instance");
const ERC20_INTERFACE = new ethers.utils.Interface(ERC20_CONTRACT.contract.abi);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: u128;
    before("Setup contract and mock balance", async () => {
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = new BN("100000000000000");
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });
      assetId = context.polkadotApi.createType("u128", ASSET_ID);

      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
      });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        alith.address,
        true
      );

      const { rawTx } = await createContract(context, "ERC20Instance");
      await context.createBlock(rawTx);
    });

    it("allows to call name", async function () {
      expect(
        (
          await web3EthCall(context.web3, {
            to: ADDRESS_ERC20,
            data: ERC20_INTERFACE.encodeFunctionData("name", []),
          })
        ).result
      ).equals(
        numberToHex(32, 256) +
          numberToHex(3, 256).slice(2) +
          stringToHex("DOT").slice(2).padEnd(64, "0")
      );
    });

    it("allows to call symbol", async function () {
      expect(
        (
          await web3EthCall(context.web3, {
            to: ADDRESS_ERC20,
            data: ERC20_INTERFACE.encodeFunctionData("symbol", []),
          })
        ).result
      ).equals(
        numberToHex(32, 256) +
          numberToHex(3, 256).slice(2) +
          stringToHex("DOT").slice(2).padEnd(64, "0")
      );
    });

    it("allows to call decimals", async function () {
      expect(
        (
          await web3EthCall(context.web3, {
            to: ADDRESS_ERC20,
            data: ERC20_INTERFACE.encodeFunctionData("decimals", []),
          })
        ).result
      ).equals(numberToHex(12, 256));
    });

    it("allows to call getBalance", async function () {
      expect(
        (
          await web3EthCall(context.web3, {
            to: ADDRESS_ERC20,
            data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [alith.address]),
          })
        ).result
      ).equals(bnToHex(100000000000000n, { bitLength: 256 }));
    });

    it("allows to call totalSupply", async function () {
      expect(
        (
          await web3EthCall(context.web3, {
            to: ADDRESS_ERC20,
            data: ERC20_INTERFACE.encodeFunctionData("totalSupply", []),
          })
        ).result
      ).equals(bnToHex(100000000000000n, { bitLength: 256 }));
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: u128;
    before("Setup contract and mock balance", async () => {
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = context.polkadotApi.createType("Balance", 100000000000000);
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });

      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
      });
      assetId = context.polkadotApi.createType("u128", ASSET_ID);

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        alith.address,
        true
      );

      const { rawTx } = await createContract(context, "ERC20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to approve transfers, and allowance matches", async function () {
      const tx = await createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: ADDRESS_ERC20,
        data: ERC20_INTERFACE.encodeFunctionData("approve", [baltathar.address, 1000]),
      });

      const { result } = await context.createBlock(tx);

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.status).to.equal(true);
      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address).to.eq(ADDRESS_ERC20);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);
      const approvals = await context.polkadotApi.query.assets.approvals(
        assetId.toU8a(),
        alith.address,
        baltathar.address
      );

      expect(approvals.unwrap().amount.toBigInt()).to.equal(1000n);
    });
    it("should gather the allowance", async function () {
      expect(
        (
          await web3EthCall(context.web3, {
            to: ADDRESS_ERC20,
            data: ERC20_INTERFACE.encodeFunctionData("allowance", [
              alith.address,
              baltathar.address,
            ]),
          })
        ).result
      ).equals(bnToHex(1000n, { bitLength: 256 }));
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: u128;
    before("Setup contract and mock balance", async () => {
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = context.polkadotApi.createType("Balance", 100000000000000);
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });

      assetId = context.polkadotApi.createType("u128", ASSET_ID);
      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
      });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        alith.address,
        true
      );

      const { rawTx } = await createContract(context, "ERC20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to approve transfer and use transferFrom", async function () {
      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: ADDRESS_ERC20,
          data: ERC20_INTERFACE.encodeFunctionData("approve", [baltathar.address, 1000]),
        })
      );

      const approvals = await context.polkadotApi.query.assets.approvals(
        assetId.toU8a(),
        alith.address,
        baltathar.address
      );

      expect(approvals.unwrap().amount.toBigInt()).to.equal(1000n);
      // We are gonna spend 1000 from alith to send it to charleth

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: ADDRESS_ERC20,
          data: ERC20_INTERFACE.encodeFunctionData("transferFrom", [
            alith.address,
            charleth.address,
            1000,
          ]),
        })
      );
      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address).to.eq(ADDRESS_ERC20);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
      expect(receipt.status).to.equal(true);

      // Approve amount is null now
      const newApprovals = await context.polkadotApi.query.assets.approvals(
        assetId.toU8a(),
        alith.address,
        baltathar.address
      );
      expect(newApprovals.isNone).to.eq(true);

      // Charleth balance is 1000
      const charletBalance = await context.polkadotApi.query.assets.account(
        assetId.toU8a(),
        charleth.address
      );
      expect(charletBalance.unwrap().balance.toBigInt()).to.equal(1000n);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: u128;
    before("Setup contract and mock balance", async () => {
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = context.polkadotApi.createType("Balance", 100000000000000);
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });

      assetId = context.polkadotApi.createType("u128", ASSET_ID);
      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
      });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        alith.address,
        true
      );

      const { rawTx } = await createContract(context, "ERC20Instance");
      await context.createBlock(rawTx);
    });
    it("allows to transfer", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: ADDRESS_ERC20,
          data: ERC20_INTERFACE.encodeFunctionData("transfer", [baltathar.address, 1000]),
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
      expect(receipt.status).to.equal(true);

      // Baltathar balance is 1000
      const baltatharBalance = await context.polkadotApi.query.assets.account(
        assetId.toU8a(),
        baltathar.address
      );
      expect(baltatharBalance.unwrap().balance.toBigInt()).to.equal(1000n);
    });
  },
  true
);

describeDevMoonbeamAllEthTxTypes("Precompiles - Assets-ERC20 Wasm", (context) => {
  let assetId: u128;
  let contractInstanceAddress: string;
  before("Setup contract and mock balance", async () => {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
      balance: balance,
    });

    assetId = context.polkadotApi.createType("u128", ASSET_ID);
    const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
      supply: balance,
    });

    const { contract, rawTx } = await createContract(context, "ERC20Instance");
    await context.createBlock(rawTx);
    contractInstanceAddress = contract.options.address;

    // We fund the contract address with this test
    await mockAssetBalance(
      context,
      assetBalance,
      assetDetails,
      alith,
      assetId,
      contractInstanceAddress,
      true
    );
  });
  it("allows to approve transfer and use transferFrom from contract calls", async function () {
    // Create approval

    const blockAlith = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: contractInstanceAddress,
        data: ERC20_INTERFACE.encodeFunctionData("approve", [baltathar.address, 1000]),
      })
    );

    const receiptAlith = await context.web3.eth.getTransactionReceipt(blockAlith.result.hash);

    expect(receiptAlith.status).to.equal(true);
    expect(receiptAlith.logs.length).to.eq(1);
    expect(receiptAlith.logs[0].address).to.eq(ADDRESS_ERC20);
    expect(receiptAlith.logs[0].topics.length).to.eq(3);
    expect(receiptAlith.logs[0].topics[0]).to.eq(SELECTORS.logApprove);

    const approvals = await context.polkadotApi.query.assets.approvals(
      assetId.toU8a(),
      contractInstanceAddress,
      baltathar.address
    );

    expect(approvals.unwrap().amount.toBigInt()).to.equal(1000n);
    // We are gonna spend 1000 from contractInstanceAddress to send it to charleth
    // Since this is a regular call, it will take contractInstanceAddress as msg.sender
    // thus from & to will be the same, and approval wont be touched
    const blockBaltathar = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: contractInstanceAddress,
        data: ERC20_INTERFACE.encodeFunctionData("transferFrom", [
          contractInstanceAddress,
          charleth.address,
          1000,
        ]),
      })
    );
    const receiptBaltathar = await context.web3.eth.getTransactionReceipt(
      blockBaltathar.result.hash
    );
    expect(receiptBaltathar.logs.length).to.eq(1);
    expect(receiptBaltathar.logs[0].address).to.eq(ADDRESS_ERC20);
    expect(receiptBaltathar.logs[0].topics.length).to.eq(3);
    expect(receiptBaltathar.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(receiptBaltathar.status).to.equal(true);

    // approvals are untouched
    const newApprovals = await context.polkadotApi.query.assets.approvals(
      assetId.toU8a(),
      contractInstanceAddress,
      baltathar.address
    );
    expect(newApprovals.unwrap().amount.toBigInt()).to.equal(1000n);

    // this time we call directly from Baltathar the ERC20 contract
    const directBlock = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: ADDRESS_ERC20,
        data: ERC20_INTERFACE.encodeFunctionData("transferFrom", [
          contractInstanceAddress,
          charleth.address,
          1000,
        ]),
      })
    );
    const direcReceipt = await context.web3.eth.getTransactionReceipt(directBlock.result.hash);
    expect(direcReceipt.logs.length).to.eq(1);
    expect(direcReceipt.logs[0].address).to.eq(ADDRESS_ERC20);
    expect(direcReceipt.logs[0].topics.length).to.eq(3);
    expect(direcReceipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(direcReceipt.status).to.equal(true);

    // Approve amount is null now
    const directApprovals = await context.polkadotApi.query.assets.approvals(
      assetId.toU8a(),
      contractInstanceAddress,
      baltathar.address
    );
    expect(directApprovals.isNone).to.eq(true);

    // Charleth balance is 2000
    const charletBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      charleth.address
    );
    expect(charletBalance.unwrap().balance.toBigInt()).to.equal(2000n);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - Assets-ERC20 Wasm", (context) => {
  let assetId: u128;
  let contractInstanceAddress: string;
  before("Setup contract and mock balance", async () => {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
      balance: balance,
    });

    assetId = context.polkadotApi.createType("u128", ASSET_ID);
    const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
      supply: balance,
    });

    const { contract, rawTx } = await createContract(context, "ERC20Instance");
    contractInstanceAddress = contract.options.address;
    await context.createBlock(rawTx);
    // We fund Alith with this test
    await mockAssetBalance(
      context,
      assetBalance,
      assetDetails,
      alith,
      assetId,
      alith.address,
      true
    );
  });

  it("Bob approves contract and use transferFrom from contract calls", async function () {
    const tx = await createTransaction(context, {
      ...ALITH_TRANSACTION_TEMPLATE,
      to: ADDRESS_ERC20,
      data: ERC20_INTERFACE.encodeFunctionData("approve", [contractInstanceAddress, 1000]),
    });

    const { result } = await context.createBlock(tx);

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.status).to.equal(true);
    expect(receipt.logs.length).to.eq(1);
    expect(receipt.logs[0].address).to.eq(ADDRESS_ERC20);
    expect(receipt.logs[0].topics.length).to.eq(3);
    expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);

    const approvals = await context.polkadotApi.query.assets.approvals(
      assetId.toU8a(),
      alith.address,
      contractInstanceAddress
    );

    expect(approvals.unwrap().amount.toBigInt()).to.equal(1000n);
    // We are gonna spend 1000 from alith.address to send it to charleth from contract address
    // even if Bob calls, msg.sender will become the contract with regular calls
    const blockBaltathar = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: contractInstanceAddress,
        data: ERC20_INTERFACE.encodeFunctionData("transferFrom", [
          alith.address,
          charleth.address,
          1000,
        ]),
      })
    );
    const receiptBaltathar = await context.web3.eth.getTransactionReceipt(
      blockBaltathar.result.hash
    );
    expect(receiptBaltathar.logs.length).to.eq(1);
    expect(receiptBaltathar.logs[0].address).to.eq(ADDRESS_ERC20);
    expect(receiptBaltathar.logs[0].topics.length).to.eq(3);
    expect(receiptBaltathar.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
    expect(receiptBaltathar.status).to.equal(true);

    // Approve amount is null now
    const approvalBaltathar = await context.polkadotApi.query.assets.approvals(
      assetId.toU8a(),
      alith.address,
      contractInstanceAddress
    );
    expect(approvalBaltathar.isNone).to.eq(true);

    // Charleth balance is 1000
    const charletBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      charleth.address
    );
    expect(charletBalance.unwrap().balance.toBigInt()).to.equal(1000n);
  });
});

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Assets-ERC20 Wasm",
  (context) => {
    let assetId: u128;
    let contractInstanceAddress: string;
    before("Setup contract and mock balance", async () => {
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = context.polkadotApi.createType("Balance", 100000000000000);
      const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });

      assetId = context.polkadotApi.createType("u128", ASSET_ID);
      const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
        supply: balance,
      });

      const { contract, rawTx } = await createContract(context, "ERC20Instance");
      contractInstanceAddress = contract.options.address;
      await context.createBlock(rawTx);
      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        contractInstanceAddress
      );
    });
    it("allows to transfer through call from SC ", async function () {
      // Create approval
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: contractInstanceAddress,
          data: ERC20_INTERFACE.encodeFunctionData("transfer", [baltathar.address, 1000]),
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
      expect(receipt.status).to.equal(true);

      // Baltathar balance is 1000
      const baltatharBalance = await context.polkadotApi.query.assets.account(
        assetId.toU8a(),
        baltathar.address
      );
      expect(baltatharBalance.unwrap().balance.toBigInt()).to.equal(1000n);
    });
  },
  true
);
