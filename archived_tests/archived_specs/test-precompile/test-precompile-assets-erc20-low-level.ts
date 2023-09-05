import "@moonbeam-network/api-augment";

import { u128 } from "@polkadot/types";
import { BN, bnToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import { web3EthCall } from "../../util/providers";
import { alith, baltathar, charleth } from "../../util/accounts";
import { mockAssetBalance } from "../../util/assets";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";

import {
  ALITH_TRANSACTION_TEMPLATE,
  CHARLETH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";

const ASSET_ID = new BN("42259045809535163221576417993425387648");
const ADDRESS_ERC20 = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
const ERC20_CONTRACT = getCompiled("ERC20Instance");
const ERC20_INTERFACE = new ethers.utils.Interface(ERC20_CONTRACT.contract.abi);
const MAX_SUPPLY = 100000000000000;

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - Low Level Transactions",
  (context) => {
    let assetId: u128;
    let contractInstanceAddress: string;

    before("Setup contract and mock balance", async () => {
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = context.polkadotApi.createType("Balance", MAX_SUPPLY);
      const assetBalance: PalletAssetsAssetAccount = context.polkadotApi.createType(
        "PalletAssetsAssetAccount",
        {
          balance: balance,
        }
      );

      assetId = context.polkadotApi.createType("u128", ASSET_ID);
      const assetDetails: PalletAssetsAssetDetails = context.polkadotApi.createType(
        "PalletAssetsAssetDetails",
        {
          supply: balance,
        }
      );

      const { contract, rawTx } = await createContract(context, "ERC20Instance");
      contractInstanceAddress = contract.options.address;
      await context.createBlock(rawTx);

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        contractInstanceAddress,
        true
      );

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

    it("can make static calls to view functions", async function () {
      expect(
        (
          await web3EthCall(context.web3, {
            to: contractInstanceAddress,
            data: ERC20_INTERFACE.encodeFunctionData("totalSupply_static", []),
          })
        ).result
      ).equals(bnToHex(new BN(MAX_SUPPLY), { bitLength: 256 }));
    });

    it("can make static calls to view functions and transact", async function () {
      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: contractInstanceAddress,
          data: ERC20_INTERFACE.encodeFunctionData("approve_max_supply", [charleth.address]),
        })
      );

      const approvals = await context.polkadotApi.query.assets.approvals(
        assetId.toU8a(),
        contractInstanceAddress,
        charleth.address
      );

      expect(approvals.unwrap().amount.toNumber()).to.equal(MAX_SUPPLY);
    });

    it("has unchanged state when submitting static call", async function () {
      const {
        result: { successful },
      } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: contractInstanceAddress,
          data: ERC20_INTERFACE.encodeFunctionData("approve_static", [baltathar.address, 1000]),
        })
      );

      const approvals = await context.polkadotApi.query.assets.approvals(
        assetId.toU8a(),
        contractInstanceAddress,
        baltathar.address
      );

      expect(successful, "Call unsuccessful").to.be.true;
      expect(approvals.isNone).to.be.true;
    });

    it("visibility preserved for static calls", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: contractInstanceAddress,
          data: ERC20_INTERFACE.encodeFunctionData("approve_ext_static", [baltathar.address, 1000]),
        })
      );

      const approvals = await context.polkadotApi.query.assets.approvals(
        assetId.toU8a(),
        contractInstanceAddress,
        baltathar.address
      );

      expect(result.successful, "Call unsuccessful").to.be.true;
      expect(approvals.isNone).to.be.true;
    });

    it("visibility preserved for delegate->static calls", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: contractInstanceAddress,
          data: ERC20_INTERFACE.encodeFunctionData("approve_delegate_to_static", [
            baltathar.address,
            1000,
          ]),
        })
      );

      const approvals = await context.polkadotApi.query.assets.approvals(
        assetId.toU8a(),
        contractInstanceAddress,
        baltathar.address
      );

      expect(result.successful, "Call unsuccessful").to.be.true;
      expect(approvals.isNone).to.be.true;
    });

    it("visibility preserved for static->delegate calls", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: contractInstanceAddress,
          data: ERC20_INTERFACE.encodeFunctionData("approve_static_to_delegate", [
            baltathar.address,
            1000,
          ]),
        })
      );

      const approvals = await context.polkadotApi.query.assets.approvals(
        assetId.toU8a(),
        contractInstanceAddress,
        baltathar.address
      );

      expect(result.successful, "Call unsuccessful").to.be.true;
      expect(approvals.isNone).to.be.true;
    });
  },
  true
);
