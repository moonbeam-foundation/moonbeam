import "@moonbeam-network/api-augment";

import { BN, hexToU8a, u8aToHex } from "@polkadot/util";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import { expect } from "chai";

import { ALITH_ADDRESS, alith, baltathar } from "../../util/accounts";
import { RELAY_SOURCE_LOCATION } from "../../util/assets";
import { registerForeignAsset } from "../../util/xcm";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { getCompiled } from "../../util/contracts";
import { ethers } from "ethers";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";
import { DUMMY_REVERT_BYTECODE } from "../../util/constants";

const palletId = "0x6D6f646c617373746d6E67720000000000000000";

const assetMetadata = {
  name: "FOREIGN",
  symbol: "FOREIGN",
  decimals: new BN(10),
  isFrozen: false,
};

const ADDRESS_ERC20 = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
const ASSET_ID = new BN("42259045809535163221576417993425387648");
const ERC20_CONTRACT = getCompiled("ERC20Instance");
const ERC20_INTERFACE = new ethers.utils.Interface(ERC20_CONTRACT.contract.abi);
const UNITS_PER_SEC = 33068783068;

describeDevMoonbeam("Mock XCM - receive horizontal transfer", (context) => {
  before("Should Register an asset and set unit per sec", async function () {
    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      RELAY_SOURCE_LOCATION,
      assetMetadata,
      UNITS_PER_SEC
    );
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());

    // ADD BALANCE TO ALITH
    const assetId = context.polkadotApi.createType("u128", ASSET_ID);
    // Get keys to modify balance
    let module = xxhashAsU8a(new TextEncoder().encode("Assets"), 128);
    let account_key = xxhashAsU8a(new TextEncoder().encode("Account"), 128);
    let blake2concatAssetId = new Uint8Array([
      ...blake2AsU8a(assetId.toU8a(), 128),
      ...assetId.toU8a(),
    ]);

    let blake2concatAccount = new Uint8Array([
      ...blake2AsU8a(hexToU8a(ALITH_ADDRESS), 128),
      ...hexToU8a(ALITH_ADDRESS),
    ]);
    let overallAccountKey = new Uint8Array([
      ...module,
      ...account_key,
      ...blake2concatAssetId,
      ...blake2concatAccount,
    ]);

    // Get keys to modify total supply & dummyCode (TODO: remove once dummy code inserted by node)
    let assetKey = xxhashAsU8a(new TextEncoder().encode("Asset"), 128);
    let overallAssetKey = new Uint8Array([...module, ...assetKey, ...blake2concatAssetId]);
    let evmCodeAssetKey = context.polkadotApi.query.evm.accountCodes.key(
      "0xFfFFfFff" + assetId.toHex().slice(2)
    );

    const balance = new BN("100000000000000");
    const assetBalance = context.polkadotApi.createType("PalletAssetsAssetAccount", {
      balance: balance,
    });
    const assetDetails = context.polkadotApi.createType("PalletAssetsAssetDetails", {
      supply: balance,
    });

    await context.createBlock(
      context.polkadotApi.tx.sudo
        .sudo(
          context.polkadotApi.tx.system.setStorage([
            [u8aToHex(overallAccountKey), u8aToHex(assetBalance.toU8a())],
            [u8aToHex(overallAssetKey), u8aToHex(assetDetails.toU8a())],
            [
              evmCodeAssetKey,
              `0x${((DUMMY_REVERT_BYTECODE.length - 2) * 2)
                .toString(16)
                .padStart(2)}${DUMMY_REVERT_BYTECODE.slice(2)}`,
            ],
          ])
        )
        .signAsync(alith)
    );
  });

  it("should succeed when modifying approve", async function () {
    context.ethTransactionType = "EIP1559";

    const { result } = await context.createBlock(
      await createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: ADDRESS_ERC20,
        data: ERC20_INTERFACE.encodeFunctionData("approve", [baltathar.address, 100000000]),
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);

    let gasEst = await context.web3.eth.estimateGas({
      from: alith.address,
      data: ERC20_INTERFACE.encodeFunctionData("approve", [baltathar.address, 0]),
      to: ADDRESS_ERC20,
    });

    const { result: result2 } = await context.createBlock(
      await createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: ADDRESS_ERC20,
        data: ERC20_INTERFACE.encodeFunctionData("approve", [baltathar.address, 0]),
        gas: gasEst,
      })
    );

    const receipt2 = await context.web3.eth.getTransactionReceipt(result2.hash);
    expect(receipt2.status).to.equal(true);
  });
});
