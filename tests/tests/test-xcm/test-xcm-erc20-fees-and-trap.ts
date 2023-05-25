import "@moonbeam-network/api-augment";
import { bnToHex, stringToU8a } from "@polkadot/util";

import { expect } from "chai";
import { ethers } from "ethers";
import { Contract } from "web3-eth-contract";

import { CHARLETH_ADDRESS } from "../../util/accounts";
import { web3EthCall } from "../../util/providers";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createTransaction,
  setupErc20Contract,
} from "../../util/transactions";
import { expectEVMResult } from "../../util/eth-transactions";
import {
  injectHrmpMessage,
  RawXcmMessage,
  sovereignAccountOfSibling,
  XcmFragment,
  weightMessage,
  XcmFragmentConfig,
} from "../../util/xcm";

const ERC20_CONTRACT = getCompiled("ERC20WithInitialSupply");
const ERC20_INTERFACE = new ethers.utils.Interface(ERC20_CONTRACT.contract.abi);

describeDevMoonbeam("Mock XCM - Fails trying to pay fees with ERC20", (context) => {
  let erc20Contract: Contract;
  let erc20ContractAddress: string;

  before("Should deploy erc20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "FirstToken", "FTK");
    erc20Contract = contract;
    erc20ContractAddress = contractAddress;
  });

  it("Should fail as tries to pay fees with ERC20", async function () {
    this.timeout(20_000);
    const paraId = 888;
    const paraSovereign = sovereignAccountOfSibling(context, paraId);
    const amountTransferred = 1_000_000n;

    // Get pallet index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const erc20XcmPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Erc20XcmBridge"
    ).index;

    // Send some erc20 tokens to the sovereign account of paraId
    const { result: result2 } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: erc20ContractAddress,
        data: ERC20_INTERFACE.encodeFunctionData("transfer", [paraSovereign, amountTransferred]),
      })
    );
    expectEVMResult(result2.events, "Succeed");

    // Check the sovereign account has reveived erc20 tokens
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [paraSovereign]),
        })
      ).result
    ).equals(bnToHex(amountTransferred, { bitLength: 256 }));

    // Create xcm message to send ERC20 tokens to Charleth
    // We don't buy any execution with native currency
    const config: XcmFragmentConfig = {
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X2: [
                {
                  PalletInstance: erc20XcmPalletIndex,
                },
                {
                  AccountKey20: {
                    network: "Any",
                    key: stringToU8a(erc20ContractAddress),
                  },
                },
              ],
            },
          },
          fungible: amountTransferred,
        },
      ],
      beneficiary: CHARLETH_ADDRESS,
    };

    // Build the xcm message
    const xcmMessage = new XcmFragment(config)
      .withdraw_asset()
      .clear_origin()
      .buy_execution()
      .deposit_asset(2n)
      .as_v2();

    // Mock the reception of the xcm message
    await injectHrmpMessage(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);
    await context.createBlock();

    // Search for Fail event
    const records = (await context.polkadotApi.query.system.events()) as any;
    const events = records.filter(
      ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
    );

    // Check the error is TooExpensive
    expect(events).to.have.lengthOf(1);
    expect(events[0].toHuman().event.data.error).equals("TooExpensive");

    // Charleth should not receive ERC20 tokens due to failed execution
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [CHARLETH_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(0n, { bitLength: 256 }));
  });
});

describeDevMoonbeam("Mock XCM - Trap ERC20", (context) => {
  let erc20Contract: Contract;
  let erc20ContractAddress: string;

  before("Should deploy erc20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "NewToken", "NTK");
    erc20Contract = contract;
    erc20ContractAddress = contractAddress;
  });

  it("Should not trap any ERC20", async function () {
    this.timeout(20_000);
    const paraId = 888;
    const paraSovereign = sovereignAccountOfSibling(context, paraId);
    const amountTransferred = 1_000_000n;

    // Get pallet index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Balances"
    ).index;
    const erc20XcmPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => pallet.name === "Erc20XcmBridge"
    ).index;

    // Send some native tokens to the sovereign account of paraId (to pay fees)
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        value: 1_000_000_000_000_000_000,
        to: paraSovereign,
        data: "0x",
      })
    );
    expectEVMResult(result.events, "Succeed");

    // Send some erc20 tokens to the sovereign account of paraId
    const { result: result2 } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: erc20ContractAddress,
        data: ERC20_INTERFACE.encodeFunctionData("transfer", [paraSovereign, amountTransferred]),
      })
    );
    expectEVMResult(result2.events, "Succeed");

    // Check the sovereign account has reveived erc20 tokens
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [paraSovereign]),
        })
      ).result
    ).equals(bnToHex(amountTransferred, { bitLength: 256 }));

    const feeAssetAmount = 1_000_000_000_000_000n;

    // Create xcm message to send ERC20 tokens to Charleth
    const config: XcmFragmentConfig = {
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: feeAssetAmount,
        },
        {
          multilocation: {
            parents: 0,
            interior: {
              X2: [
                {
                  PalletInstance: erc20XcmPalletIndex,
                },
                {
                  AccountKey20: {
                    network: "Any",
                    key: stringToU8a(erc20ContractAddress),
                  },
                },
              ],
            },
          },
          fungible: amountTransferred,
        },
      ],
      beneficiary: CHARLETH_ADDRESS,
    };

    // Build the xcm message without deposit_asset()
    // This is to trap all the assets present in the holding register
    const xcmMessage = new XcmFragment(config)
      .withdraw_asset()
      .clear_origin()
      .buy_execution()
      .as_v2();

    // Mock the reception of the xcm message
    await injectHrmpMessage(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);
    await context.createBlock();

    const chargedWeight = await weightMessage(
      context,
      context.polkadotApi.createType("XcmVersionedXcm", xcmMessage) as any
    );
    // We are charging chargedWeight
    // chargedWeight * 50000 = chargedFee
    const chargedFee = chargedWeight * 50000n;

    const amountOfTrappedAssets = feeAssetAmount - chargedFee;
    const claimConfig = {
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: amountOfTrappedAssets,
        },
      ],
      beneficiary: paraSovereign,
    };
    // Check non-erc20 can be claimed
    const xcmMessageToClaimAssets = new XcmFragment(claimConfig)
      .claim_asset()
      .buy_execution()
      .deposit_asset()
      .as_v2();

    // Mock the reception of the xcm message
    await injectHrmpMessage(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessageToClaimAssets,
    } as RawXcmMessage);
    await context.createBlock();

    // Search for AssetsClaimed event
    const records = (await context.polkadotApi.query.system.events()) as any;
    const events = records.filter(
      ({ event }) => event.section == "polkadotXcm" && event.method == "AssetsClaimed"
    );
    expect(events).to.have.lengthOf(1);

    const chargedWeightForClaim = await weightMessage(
      context,
      context.polkadotApi.createType("XcmVersionedXcm", xcmMessageToClaimAssets) as any
    );
    // We are charging chargedWeightForClaim
    // chargedWeightForClaim * 50000 = chargedFeeForClaim
    const chargedFeeForClaim = chargedWeightForClaim * 50000n;

    // Check the balance is correct
    expect(BigInt(await context.web3.eth.getBalance(paraSovereign, 5))).to.equal(
      BigInt(await context.web3.eth.getBalance(paraSovereign, 4)) +
        (amountOfTrappedAssets - chargedFeeForClaim)
    );

    // Mock again the reception of the initial xcm message
    await injectHrmpMessage(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);
    await context.createBlock();

    const failedClaimConfig: XcmFragmentConfig = {
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X2: [
                {
                  PalletInstance: erc20XcmPalletIndex,
                },
                {
                  AccountKey20: {
                    network: "Any",
                    key: stringToU8a(erc20ContractAddress),
                  },
                },
              ],
            },
          },
          fungible: amountTransferred,
        },
      ],
      beneficiary: paraSovereign,
    };

    // Check erc20 cannot be claimed
    const xcmMessageFailedClaim = new XcmFragment(failedClaimConfig)
      .claim_asset()
      .buy_execution()
      .deposit_asset()
      .as_v2();

    await injectHrmpMessage(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessageFailedClaim,
    } as RawXcmMessage);
    await context.createBlock();

    // Search for UnknownClaim error
    const records2 = (await context.polkadotApi.query.system.events()) as any;
    const events2 = records2.filter(
      ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
    );
    expect(events2).to.have.lengthOf(1);
    expect(events2[0].toHuman().event.data.error).equals("UnknownClaim");

    // Check the sovereign account has the same initial amount of ERC20 tokens
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [paraSovereign]),
        })
      ).result
    ).equals(bnToHex(amountTransferred, { bitLength: 256 }));
  });
});
