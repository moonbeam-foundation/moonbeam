import "@moonbeam-network/api-augment";
import { bnToHex, stringToU8a } from "@polkadot/util";

import { expect } from "chai";
import { ethers } from "ethers";
import { Contract } from "web3-eth-contract";

import { ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS } from "../../util/accounts";
import { PRECOMPILE_XTOKENS_ADDRESS } from "../../util/constants";
import { web3EthCall } from "../../util/providers";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
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

describeDevMoonbeam("Mock XCM - Send two local ERC20", (context) => {
  let erc20Contract1: Contract;
  let erc20ContractAddress1: string;

  let erc20Contract2: Contract;
  let erc20ContractAddress2: string;

  before("Should deploy the first ERC20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "First", "FIR");
    erc20Contract1 = contract;
    erc20ContractAddress1 = contractAddress;
  });

  before("Should deploy the second ERC20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "Second", "SEC");
    erc20Contract2 = contract;
    erc20ContractAddress2 = contractAddress;
  });

  it("Should be able to transfer two ERC20 tokens through xtoken precomp", async function () {
    const amountTransferred = 1000n;
    // Destination as multilocation
    const destination = [
      // one parent
      1,
      // This represents X1(AccountKey20(BALTATHAR_ADDRESS, NetworkAny))
      // AccountKey20 variant (03) + the 20 bytes account + Any network variant (00)
      ["0x03" + BALTATHAR_ADDRESS.slice(2) + "00"],
    ];

    const currency1 = [erc20ContractAddress1, amountTransferred];
    const currency2 = [erc20ContractAddress2, amountTransferred];

    const data = XTOKENS_INTERFACE.encodeFunctionData(
      // action
      "transferMultiCurrencies",
      [
        // addresses of the multiassets
        [currency1, currency2],
        // index fee
        1n,
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
    expect(BigInt(await context.web3.eth.getBalance(ALITH_ADDRESS, 3))).to.equal(
      BigInt(await context.web3.eth.getBalance(ALITH_ADDRESS, 2)) - fees
    );

    // Erc20 tokens of the first contract should have been spent
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress1,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [ALITH_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(ERC20_TOTAL_SUPPLY - amountTransferred, { bitLength: 256 }));

    // Erc20 tokens of the second contract should have been spent
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress2,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [ALITH_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(ERC20_TOTAL_SUPPLY - amountTransferred, { bitLength: 256 }));
  });
});

describeDevMoonbeam("Mock XCM - Receive two ERC20", (context) => {
  let erc20Contract1: Contract;
  let erc20ContractAddress1: string;

  let erc20Contract2: Contract;
  let erc20ContractAddress2: string;

  before("Should deploy the first ERC20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "FirstToken", "FTK");
    erc20Contract1 = contract;
    erc20ContractAddress1 = contractAddress;
  });

  before("Should deploy the second ERC20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "SecondToken", "STK");
    erc20Contract2 = contract;
    erc20ContractAddress2 = contractAddress;
  });

  it("Should be able to transfer two ERC20 through incoming XCM meesage", async function () {
    this.timeout(20_000);
    const paraId = 888;
    const paraSovereign = sovereignAccountOfSibling(context, paraId);
    const amountTransferredOf1 = 1_000_000n;
    const amountTransferredOf2 = 2_000_000n;

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

    // Send some erc20 tokens (of first contract) to the sovereign account of paraId
    const { result: result2 } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: erc20ContractAddress1,
        data: ERC20_INTERFACE.encodeFunctionData("transfer", [paraSovereign, amountTransferredOf1]),
      })
    );
    expectEVMResult(result2.events, "Succeed");

    // Check the sovereign account has reveived ERC20 tokens (of first contract)
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress1,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [paraSovereign]),
        })
      ).result
    ).equals(bnToHex(amountTransferredOf1, { bitLength: 256 }));

    // Send some ERC20 tokens (of second contract) to the sovereign account of paraId
    const { result: result3 } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: erc20ContractAddress2,
        data: ERC20_INTERFACE.encodeFunctionData("transfer", [paraSovereign, amountTransferredOf2]),
      })
    );
    expectEVMResult(result3.events, "Succeed");

    // Check the sovereign account has reveived ERC20 tokens (of second contract)
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress2,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [paraSovereign]),
        })
      ).result
    ).equals(bnToHex(amountTransferredOf2, { bitLength: 256 }));

    // Create the xcm message to send ERC20s to Charleth
    const config: XcmFragmentConfig = {
      assets: [
        {
          multilocation: {
            parents: 0,
            interior: {
              X1: { PalletInstance: balancesPalletIndex },
            },
          },
          fungible: 1_000_000_000_000_000n,
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
                    key: erc20ContractAddress2,
                  },
                },
              ],
            },
          },
          fungible: amountTransferredOf2,
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
                    key: erc20ContractAddress1,
                  },
                },
              ],
            },
          },
          fungible: amountTransferredOf1,
        },
      ],
      beneficiary: CHARLETH_ADDRESS,
    };

    // Build the xcm message
    const xcmMessage = new XcmFragment(config)
      .withdraw_asset()
      .clear_origin()
      .buy_execution()
      .deposit_asset(3n)
      .as_v2();

    // Mock the reception of the xcm message
    await injectHrmpMessage(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);
    await context.createBlock();

    // Erc20 tokens (of first contract) should have been received in Charleth's address
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress1,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [CHARLETH_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(amountTransferredOf1, { bitLength: 256 }));

    // Erc20 tokens (of second contract) should have been received in Charleth's address
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress2,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [CHARLETH_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(amountTransferredOf2, { bitLength: 256 }));
  });
});
