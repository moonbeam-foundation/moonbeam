import "@moonbeam-network/api-augment";
import { BN } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import { alith } from "../../util/accounts";
import { mockAssetBalance, RELAY_V3_SOURCE_LOCATION } from "../../util/assets";
import { verifyLatestBlockFees } from "../../util/block";
import {
  PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V1,
  PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V2,
} from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";

const ADDRESS_RELAY_ASSETS = "0xffffffff1fcacbd218edc0eba20fc2308c778080";
const XCM_TRANSACTOR_CONTRACT_V1 = getCompiled("precompiles/xcm-transactor/src/v1/XcmTransactorV1");
const XCM_TRANSACTOR_INTERFACE_V1 = new ethers.utils.Interface(
  XCM_TRANSACTOR_CONTRACT_V1.contract.abi
);
const XCM_TRANSACTOR_CONTRACT_V2 = getCompiled("precompiles/xcm-transactor/src/v2/XcmTransactorV2");
const XCM_TRANSACTOR_INTERFACE_V2 = new ethers.utils.Interface(
  XCM_TRANSACTOR_CONTRACT_V2.contract.abi
);

const registerXcmTransactorAndContract = async (context: DevTestContext) => {
  await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.xcmTransactor.register(alith.address, 0)
    )
  );

  await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.xcmTransactor.setTransactInfo(
        RELAY_V3_SOURCE_LOCATION,
        { refTime: 1, proofSize: 64 * 1024 } as any,
        { refTime: 20_000_000_000, proofSize: 256 * 1024 } as any,
        { refTime: 1, proofSize: 64 * 1024 } as any
      )
    )
  );

  await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.xcmTransactor.setFeePerSecond(
        RELAY_V3_SOURCE_LOCATION,
        new BN(1000000000000)
      )
    )
  );
};

const registerXcmTransactorDerivativeIndex = async (context: DevTestContext) => {
  await context.createBlock(
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.xcmTransactor.register(alith.address, 0)
    )
  );
};

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm transactor", (context) => {
  before("Setup genesis account and relay accounts", async () => {
    await registerXcmTransactorAndContract(context);
  });

  it("allows to retrieve index through precompiles", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V1,
          data: XCM_TRANSACTOR_INTERFACE_V1.encodeFunctionData("indexToAccount", [0]),
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
          to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V1,
          data: XCM_TRANSACTOR_INTERFACE_V1.encodeFunctionData("transactInfo", [asset]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000001" +
        "000000000000000000000000000000000000000000000000000000e8d4a51000" +
        "00000000000000000000000000000000000000000000000000000004a817c800"
    );
  });

  it("allows to issue transfer xcm transactor", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance: PalletAssetsAssetAccount = context.polkadotApi.createType(
      "PalletAssetsAssetAccount",
      {
        balance: balance,
      }
    );

    const assetId = context.polkadotApi.createType(
      "u128",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails: PalletAssetsAssetDetails = context.polkadotApi.createType(
      "PalletAssetsAssetDetails",
      {
        supply: balance,
      }
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
    const data = XCM_TRANSACTOR_INTERFACE_V1.encodeFunctionData(
      // action
      "transactThroughDerivativeMultilocation",
      [transactor, index, asset, weight, transact_call]
    );
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V1,
        data,
      })
    );

    // We have used 1000 units to pay for the fees in the relay  (plus 1 transact_extra_weight), so
    // balance and supply should have changed
    const afterAssetBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      alith.address
    );

    const expectedBalance = 100000000000000n - 1000n - 1n;
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
    const assetBalance: PalletAssetsAssetAccount = context.polkadotApi.createType(
      "PalletAssetsAssetAccount",
      {
        balance: balance,
      }
    );

    const assetId = context.polkadotApi.createType(
      "u128",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails: PalletAssetsAssetDetails = context.polkadotApi.createType(
      "PalletAssetsAssetDetails",
      {
        supply: balance,
      }
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
    const data = XCM_TRANSACTOR_INTERFACE_V1.encodeFunctionData(
      // action
      "transactThroughDerivative",
      [transactor, index, asset, weight, transact_call]
    );
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V1,
        data,
      })
    );

    // We have used 1000 units to pay for the fees in the relay, so balance and supply should
    // have changed
    const afterAssetBalance = await context.polkadotApi.query.assets.account(
      assetId.toU8a(),
      alith.address
    );

    const expectedBalance = 100000000000000n - 1000n - 1n;
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
    const data = XCM_TRANSACTOR_INTERFACE_V1.encodeFunctionData(
      // action
      "feePerSecond",
      [asset]
    );
    const tx_call = await web3EthCall(context.web3, {
      to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V1,
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
    const data = XCM_TRANSACTOR_INTERFACE_V1.encodeFunctionData(
      // action
      "transactInfoWithSigned",
      [asset]
    );
    const tx_call = await web3EthCall(context.web3, {
      to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V1,
      data,
    });

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000001" +
        "0000000000000000000000000000000000000000000000000000000000000001" +
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
    const data = XCM_TRANSACTOR_INTERFACE_V1.encodeFunctionData(
      // action
      "transactThroughSigned",
      [dest, asset, weight, transact_call]
    );

    const rawTxn = await createTransaction(context, {
      ...ALITH_TRANSACTION_TEMPLATE,
      to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V1,
      data,
    });

    await context.createBlock(rawTxn);

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
    const data = XCM_TRANSACTOR_INTERFACE_V1.encodeFunctionData(
      // action
      "transactThroughSignedMultilocation",
      [dest, asset, weight, transact_call]
    );

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V1,
        data,
      })
    );

    // 1000 fee for the relay is paid with relay assets
    await verifyLatestBlockFees(context);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm transactor", (context) => {
  it("allows to transact signed multilocation with custom weight and fee", async function () {
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
    // transact weight
    const transactWeight = 1000;

    // overall weight
    const overallWeight = 2000;

    // Fee amount
    const feeAmount = 1000;

    // Call the precompile
    const data = XCM_TRANSACTOR_INTERFACE_V2.encodeFunctionData(
      // action
      `transactThroughSignedMultilocation(` +
        `(uint8,bytes[]),` +
        `(uint8,bytes[]),` +
        `uint64,bytes,` +
        `uint256,uint64)`,
      [dest, asset, transactWeight, transact_call, feeAmount, overallWeight]
    );

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V2,
        data,
      })
    );

    // 1000 fee for the relay is paid with relay assets
    await verifyLatestBlockFees(context);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm transactor", (context) => {
  it("allows to transact signed with custom weight and fee", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const dest: [number, {}[]] =
      // Destination as multilocation
      [
        // one parent
        1,
        [],
      ];
    // Asset as currency Id address
    const asset = ADDRESS_RELAY_ASSETS;

    // we dont care, the call wont be executed
    const transact_call = new Uint8Array([0x01]);
    // transact weight
    const transactWeight = 1000;

    // overall weight
    const overallWeight = 2000;

    // Fee amount
    const feeAmount = 1000;

    // Call the precompile
    const data = XCM_TRANSACTOR_INTERFACE_V2.encodeFunctionData(
      // action
      "transactThroughSigned((uint8,bytes[]),address,uint64,bytes,uint256,uint64)",
      [dest, asset, transactWeight, transact_call, feeAmount, overallWeight]
    );

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V2,
        data,
      })
    );

    // 1000 fee for the relay is paid with relay assets
    await verifyLatestBlockFees(context);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xcm transactor", (context) => {
  before("Register derivative index", async () => {
    await registerXcmTransactorDerivativeIndex(context);
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V2,
          data: XCM_TRANSACTOR_INTERFACE_V2.encodeFunctionData("indexToAccount", [0]),
        })
      ).result
    ).to.equal("0x000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac");
  });

  it("allows to transact through derivative multiloc custom fee and weight", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay
    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance: PalletAssetsAssetAccount = context.polkadotApi.createType(
      "PalletAssetsAssetAccount",
      {
        balance: balance,
      }
    );

    const assetId = context.polkadotApi.createType(
      "u128",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails: PalletAssetsAssetDetails = context.polkadotApi.createType(
      "PalletAssetsAssetDetails",
      {
        supply: balance,
      }
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
    // transact weight
    const transactWeight = 500;

    // overall weight
    const overallWeight = 1000;

    // Fee amount
    const feeAmount = 1000;
    // Call the precompile
    const data = XCM_TRANSACTOR_INTERFACE_V2.encodeFunctionData(
      // action
      `transactThroughDerivativeMultilocation(` +
        `uint8,` +
        `uint16,` +
        `(uint8,bytes[]),` +
        `uint64,bytes,` +
        `uint256,` +
        `uint64` +
        `)`,
      [transactor, index, asset, transactWeight, transact_call, feeAmount, overallWeight]
    );

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V2,
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
    await registerXcmTransactorDerivativeIndex(context);
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V2,
          data: XCM_TRANSACTOR_INTERFACE_V2.encodeFunctionData("indexToAccount", [0]),
        })
      ).result
    ).to.equal("0x000000000000000000000000f24ff3a9cf04c71dbc94d0b566f7a27b94566cac");
  });

  it("allows to issue transfer xcm transactor with currency Id", async function () {
    // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
    // And we need relay tokens for issuing a transaction to be executed in the relay

    const balance = context.polkadotApi.createType("Balance", 100000000000000);
    const assetBalance: PalletAssetsAssetAccount = context.polkadotApi.createType(
      "PalletAssetsAssetAccount",
      {
        balance: balance,
      }
    );

    const assetId = context.polkadotApi.createType(
      "u128",
      new BN("42259045809535163221576417993425387648")
    );
    const assetDetails: PalletAssetsAssetDetails = context.polkadotApi.createType(
      "PalletAssetsAssetDetails",
      {
        supply: balance,
      }
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
    // transact weight
    const transactWeight = 500;

    // overall weight
    const overallWeight = 1000;

    // Fee amount
    const feeAmount = 1000;

    // Call the precompile
    const data = XCM_TRANSACTOR_INTERFACE_V2.encodeFunctionData(
      // action
      "transactThroughDerivative(uint8,uint16,address,uint64,bytes,uint256,uint64)",
      [transactor, index, asset, transactWeight, transact_call, feeAmount, overallWeight]
    );

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XCM_TRANSACTOR_ADDRESS_V2,
        data,
      })
    );

    // We have used 1000 units to pay for the fees in the relay  (plus 1 transact_extra_weight), so
    // balance and supply should have changed
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
