import "@moonbeam-network/api-augment";
import { bnToHex, stringToU8a } from "@polkadot/util";

import { expect } from "chai";
import { ethers } from "ethers";
import { Contract } from "web3-eth-contract";

import { ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS } from "../../util/accounts";
import { PRECOMPILE_XTOKENS_ADDRESS } from "../../util/constants";
import { web3EthCall } from "../../util/providers";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeamAllRuntimes } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createTransaction,
  setupErc20Contract,
  ERC20_TOTAL_SUPPLY,
} from "../../util/transactions";
import { expectEVMResult } from "../../util/eth-transactions";
import {
  injectHrmpMessage,
  RawXcmMessage,
  sovereignAccountOfSibling,
  XcmFragment,
  XcmFragmentConfig,
} from "../../util/xcm";

const ERC20_CONTRACT = getCompiled("ERC20WithInitialSupply");
const ERC20_INTERFACE = new ethers.utils.Interface(ERC20_CONTRACT.contract.abi);
const XTOKENS_CONTRACT = getCompiled("XtokensInstance");
const XTOKENS_INTERFACE = new ethers.utils.Interface(XTOKENS_CONTRACT.contract.abi);

describeDevMoonbeamAllRuntimes("Mock XCM - Send local erc20", (context) => {
  let erc20Contract: Contract;
  let erc20ContractAddress: string;

  before("Should deploy erc20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "MyToken", "TKN");
    erc20Contract = contract;
    erc20ContractAddress = contractAddress;
  });

  it("Should be able to transfer ERC20 token through xcm with xtoken precomp", async function () {
    const amountTransferred = 1000n;
    // Destination as multilocation
    const destination = [
      // one parent
      1,
      // This represents X1(AccountKey20(BALTATHAR_ADDRESS, NetworkAny))
      // AccountKey20 variant (03) + the 20 bytes account + Any network variant (00)
      ["0x03" + BALTATHAR_ADDRESS.slice(2) + "00"],
    ];
    const data = XTOKENS_INTERFACE.encodeFunctionData(
      // action
      "transfer",
      [
        // address of the multiasset
        erc20ContractAddress,
        // amount
        amountTransferred,
        // Destination as multilocation
        destination,
        // weight
        500_000_000n,
      ]
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XTOKENS_ADDRESS,
        data,
      })
    );
    expectEVMResult(result.events, "Succeed");

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    const gasPrice = receipt.effectiveGasPrice;
    const fees = BigInt(receipt.gasUsed) * BigInt(gasPrice);

    // Fees should have been spent
    expect(BigInt(await context.web3.eth.getBalance(ALITH_ADDRESS, 2))).to.equal(
      BigInt(await context.web3.eth.getBalance(ALITH_ADDRESS, 1)) - fees
    );

    // Erc20 tokens should have been spent
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [ALITH_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(ERC20_TOTAL_SUPPLY - amountTransferred, { bitLength: 256 }));
  });
});

describeDevMoonbeamAllRuntimes("Mock XCM - Receive back erc20", (context) => {
  let erc20Contract: Contract;
  let erc20ContractAddress: string;

  before("Should deploy erc20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "MyToken", "TKN");
    erc20Contract = contract;
    erc20ContractAddress = contractAddress;
  });

  it("Should be able to transfer ERC20 token through incoming XCM message", async function () {
    this.timeout(20_000);
    const paraId = 888;
    const paraSovereign = sovereignAccountOfSibling(context, paraId);
    const amountTransferred = 1_000_000n;

    // Get pallet indices
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
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [paraSovereign]),
        })
      ).result
    ).equals(bnToHex(amountTransferred, { bitLength: 256 }));

    // Create the incoming xcm message
    const config: XcmFragmentConfig = {
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: Number(balancesPalletIndex) },
            },
          },
          fungible: 100_000_000_000_000_000n,
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
                    key: erc20ContractAddress,
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

    // Erc20 tokens should have been received
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [CHARLETH_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(amountTransferred, { bitLength: 256 }));
  });
});
