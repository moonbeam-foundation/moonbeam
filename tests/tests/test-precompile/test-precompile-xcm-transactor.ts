import "@moonbeam-network/api-augment";

import { BN } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";

import { alith } from "../../util/accounts";
import { mockAssetBalance, RELAY_V1_SOURCE_LOCATION } from "../../util/assets";
import { verifyLatestBlockFees } from "../../util/block";
import { PRECOMPILE_XCM_TRANSACTOR_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";

const ADDRESS_RELAY_ASSETS = "0xffffffff1fcacbd218edc0eba20fc2308c778080";
const XCM_TRANSACTOR_CONTRACT = getCompiled("XcmTransactorInstance");
const XCM_TRANSACTOR_INTERFACE = new ethers.utils.Interface(XCM_TRANSACTOR_CONTRACT.contract.abi);

const registerXcmTransactorAndContract = async (context: DevTestContext) => {
  await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.xcmTransactor.register(alith.address, 0)
    )
  );

  await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.xcmTransactor.setTransactInfo(
        RELAY_V1_SOURCE_LOCATION,
        new BN(0),
        new BN(20000000000),
        new BN(0)
      )
    )
  );

  await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.xcmTransactor.setFeePerSecond(
        RELAY_V1_SOURCE_LOCATION,
        new BN(1000000000000)
      )
    )
  );

  const { rawTx } = await createContract(context, "XcmTransactorInstance");
  await context.createBlock(rawTx);
};

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm transactor", (context) => {
  before("Setup genesis account and relay accounts", async () => {
    await registerXcmTransactorAndContract(context);
  });

  it("allows to retrieve index through precompiles", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS,
          data: XCM_TRANSACTOR_INTERFACE.encodeFunctionData("index_to_account", [0]),
        })
      ).result
    ).to.equal("0x000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac");
  });

  it("allows to retrieve transactor info through precompiles old interface", async function () {
    // Destination as multilocation, one parent
    const asset: [number, {}[]] = [1, []];

    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS,
          data: XCM_TRANSACTOR_INTERFACE.encodeFunctionData("transact_info", [asset]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000000" +
        "000000000000000000000000000000000000000000000000000000e8d4a51000" +
        "00000000000000000000000000000000000000000000000000000004a817c800"
    );
  });

  it("allows to issue transfer xcm transactor", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
      balance: balance,
    });

    const assetId = context.polkadotApi.createType(
      "u128",
      new BN("42259045809535163221576417993425387648")
    );
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

    const beforeAssetBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      alith.address
    );
    const beforeAssetDetails = await context.polkadotApi.query.assets.asset(assetId.toU8a());

    // supply and balance should be the same
    expect(beforeAssetBalance.unwrap().balance.toBigInt()).to.equal(100000000000000n);
    expect(beforeAssetDetails.unwrap().supply.toBigInt()).to.equal(100000000000000n);

    const transactor = 0;
    const index = 0;
    const asset: [number, {}[]] =
      // Destination as multilocation
      [
        // one parent
        1,
        [],
      ];
    // we dont care, the call wont be executed
    const transact_call = new Uint8Array([0x01]);
    // weight
    const weight = 1000;
    // Call the precompile
    const data = XCM_TRANSACTOR_INTERFACE.encodeFunctionData(
      // action
      "transact_through_derivative_multilocation",
      [transactor, index, asset, weight, transact_call]
    );
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS,
        data,
      })
    );

    // We have used 1000 units to pay for the fees in the relay, so balance and supply should
    // have changed
    const afterAssetBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      alith.address
    );

    const expectedBalance = 100000000000000n - 1000n;
    expect(afterAssetBalance.unwrap().balance.toBigInt()).to.equal(expectedBalance);

    const AfterAssetDetails = await context.polkadotApi.query.assets.asset(assetId.toU8a());

    expect(AfterAssetDetails.unwrap().supply.toBigInt()).to.equal(expectedBalance);

    // 1000 fee for the relay is paid with relay assets
    await verifyLatestBlockFees(context);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm transactor", (context) => {
  before("Setup genesis account and relay accounts", async () => {
    await registerXcmTransactorAndContract(context);
  });

  it("allows to issue transfer xcm transactor with currency Id", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay

    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
      balance: balance,
    });

    const assetId = context.polkadotApi.createType(
      "u128",
      new BN("42259045809535163221576417993425387648")
    );
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

    const beforeAssetBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      alith.address
    );

    const beforeAssetDetails = await context.polkadotApi.query.assets.asset(assetId.toU8a());

    // supply and balance should be the same
    expect(beforeAssetBalance.unwrap().balance.toBigInt()).to.equal(100000000000000n);
    expect(beforeAssetDetails.unwrap().supply.toBigInt()).to.equal(100000000000000n);

    const transactor = 0;
    const index = 0;
    // Destination as currency Id address
    const asset = ADDRESS_RELAY_ASSETS;
    // we dont care, the call wont be executed
    const transact_call = new Uint8Array([0x01]);
    // weight
    const weight = 1000;
    // Call the precompile
    const data = XCM_TRANSACTOR_INTERFACE.encodeFunctionData(
      // action
      "transact_through_derivative",
      [transactor, index, asset, weight, transact_call]
    );
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS,
        data,
      })
    );

    // We have used 1000 units to pay for the fees in the relay, so balance and supply should
    // have changed
    const afterAssetBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      alith.address
    );

    const expectedBalance = 100000000000000n - 1000n;
    expect(afterAssetBalance.unwrap().balance.toBigInt()).to.equal(expectedBalance);

    const AfterAssetDetails = await context.polkadotApi.query.assets.asset(assetId.toU8a());

    expect(AfterAssetDetails.unwrap().supply.toBigInt()).to.equal(expectedBalance);

    // 1000 fee for the relay is paid with relay assets
    await verifyLatestBlockFees(context);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm transactor", (context) => {
  before("Setup genesis account and relay accounts", async () => {
    await registerXcmTransactorAndContract(context);
  });

  it("allows to retrieve fee per second through precompiles", async function () {
    const asset: [number, {}[]] =
      // asset as multilocation
      [
        // one parent
        1,
        [],
      ];
    const data = XCM_TRANSACTOR_INTERFACE.encodeFunctionData(
      // action
      "fee_per_second",
      [asset]
    );
    const tx_call = await web3EthCall(context.web3, {
      to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS,
      data,
    });

    expect(tx_call.result).to.equal(
      "0x000000000000000000000000000000000000000000000000000000e8d4a51000"
    );
  });

  it("allows to retrieve transactor info through precompiles", async function () {
    const asset: [number, {}[]] =
      // Destination as multilocation
      [
        // one parent
        1,
        [],
      ];
    const data = XCM_TRANSACTOR_INTERFACE.encodeFunctionData(
      // action
      "transact_info_with_signed",
      [asset]
    );
    const tx_call = await web3EthCall(context.web3, {
      to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS,
      data,
    });

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000000" +
        "0000000000000000000000000000000000000000000000000000000000000000" +
        "00000000000000000000000000000000000000000000000000000004a817c800"
    );
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm transactor", (context) => {
  before("Setup genesis account and relay accounts", async () => {
    await registerXcmTransactorAndContract(context);
  });

  it("allows to issue transfer signed xcm transactor with currency Id", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const dest: [number, {}[]] =
      // Destination as multilocation
      [
        // one parent
        1,
        [],
      ];
    // Destination as currency Id address
    const asset = ADDRESS_RELAY_ASSETS;
    // we dont care, the call wont be executed
    const transact_call = new Uint8Array([0x01]);
    // weight
    const weight = 1000;
    // Call the precompile
    const data = XCM_TRANSACTOR_INTERFACE.encodeFunctionData(
      // action
      "transact_through_signed",
      [dest, asset, weight, transact_call]
    );

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS,
        data,
      })
    );

    // 1000 fee for the relay is paid with relay assets
    await verifyLatestBlockFees(context);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm transactor", (context) => {
  before("Setup genesis account and relay accounts", async () => {
    await registerXcmTransactorAndContract(context);
  });

  it("allows to issue transfer signed xcm transactor with multilocation", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const dest: [number, {}[]] =
      // Destination as multilocation
      [
        // one parent
        1,
        [],
      ];
    // asset as multilocation
    const asset: [number, {}[]] =
      // Destination as multilocation
      [
        // one parent
        1,
        [],
      ];
    // we dont care, the call wont be executed
    const transact_call = new Uint8Array([0x01]);
    // weight
    const weight = 1000;
    // Call the precompile
    const data = XCM_TRANSACTOR_INTERFACE.encodeFunctionData(
      // action
      "transact_through_signed_multilocation",
      [dest, asset, weight, transact_call]
    );

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS,
        data,
      })
    );

    // 1000 fee for the relay is paid with relay assets
    await verifyLatestBlockFees(context);
  });
});
