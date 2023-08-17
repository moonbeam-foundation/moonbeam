import "@moonbeam-network/api-augment";
import { bnToHex } from "@polkadot/util";

import { expect } from "chai";
import { ethers } from "ethers";
import { Contract } from "web3-eth-contract";

import { ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS } from "../util/accounts";
import { getCompiled } from "../util/contracts";
import { expectEVMResult } from "../util/eth-transactions";
import { customWeb3Request, web3EthCall } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createTransaction,
  setupErc20Contract,
} from "../util/transactions";
import {
  injectHrmpMessage,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
  RawXcmMessage,
  XcmFragment,
  XcmFragmentConfig,
} from "../util/xcm";

const ERC20_CONTRACT = getCompiled("ERC20WithInitialSupply");
const ERC20_INTERFACE = new ethers.utils.Interface(ERC20_CONTRACT.contract.abi);

describeDevMoonbeam("Trace ERC20 xcm", (context) => {
  let erc20Contract: Contract;
  let erc20ContractAddress: string;
  let transactionHash: string;

  before("should receive ERC20 via XCM", async function () {
    // Deploy ERC20 contract
    const { contract, contractAddress } = await setupErc20Contract(context, "MyToken", "TKN");
    erc20Contract = contract;
    erc20ContractAddress = contractAddress;

    // Define remote parachain
    const paraId = 888;
    const paraSovereign = sovereignAccountOfSibling(context, paraId);

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
    const amountTransferred = 1_000_000n;
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
      .deposit_asset_v3(2n)
      .as_v3();

    // Mock the reception of the xcm message
    await injectHrmpMessageAndSeal(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Retrieve the stored ethereum transaction hash
    transactionHash = (await context.web3.eth.getBlock("latest")).transactions[0];

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

  it("should trace ERC20 xcm transaction with debug_traceTransaction", async function () {
    const receipt = await context.web3.eth.getTransactionReceipt(transactionHash);
    const trace = await customWeb3Request(context.web3, "debug_traceTransaction", [
      transactionHash,
      { tracer: "callTracer" },
    ]);
    // We traced the transaction, and the traced gas used should be greater* than or equal to the
    // one recorded in the ethereum transaction receipt.
    // *gasUsed on tracing does not take into account gas refund.
    console.log(receipt);
    expect(receipt.gasUsed).to.be.at.most(
      context.web3.utils.hexToNumber(trace.result.gasUsed) as number
    );
  });
});

describeDevMoonbeam("Trace ERC20 xcm with regular ethereum transactions", (context) => {
  let erc20Contract: Contract;
  let erc20ContractAddress: string;
  const paraId = 888;
  let paraSovereign: string;
  let transactionsHashes: string[];

  before("should builmd a block with ERC20-XCM and ethereum transactions", async function () {
    // Deploy ERC20 contract
    const { contract, contractAddress } = await setupErc20Contract(context, "MyToken", "TKN");
    erc20Contract = contract;
    erc20ContractAddress = contractAddress;

    // Compute remote parachain sovereign account
    paraSovereign = sovereignAccountOfSibling(context, paraId);

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
    const amountTransferred = 1_000_000n;
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
      .deposit_asset_v3(2n)
      .as_v3();

    // Mock the reception of the xcm message
    await injectHrmpMessage(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Most important part: create block with "regular" ethereum transactions
    await context.createBlock([
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: erc20ContractAddress,
        data: ERC20_INTERFACE.encodeFunctionData("transfer", [BALTATHAR_ADDRESS, 1]),
      }),
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: erc20ContractAddress,
        data: ERC20_INTERFACE.encodeFunctionData("transfer", [BALTATHAR_ADDRESS, 2]),
      }),
    ]);

    // Retrieve the stored ethereum transactions hashes
    //transactionsHashes = (await context.web3.eth.getBlock("latest")).transactions;

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

  it.only("should trace all ethereum transactions with debug_traceBlockByNumber", async function () {
    const number = await context.web3.eth.getBlockNumber();
    const trace = (
      await customWeb3Request(context.web3, "debug_traceBlockByNumber", [
        number.toString(),
        { tracer: "callTracer" },
      ])
    ).result;
    // 3 ethereum transactions: erc20-xcm transfer + regular ethereum transaction
    expect(trace.length).to.eq(3);
    // 1st transaction is erc20-xcm transfer.
    // - `From` is the sovereign account of the remote chain
    // - `To` is the erc20 contract address.
    expect(trace[0].from).to.eq(paraSovereign.toLowerCase());
    expect(trace[0].to).to.eq(erc20ContractAddress.toLowerCase());
    expect(trace[0].type).to.eq("CALL");
    // 2nd transaction is regular erc20 transfer.
    // - `From` is Alith's adddress.
    // - `To` is the erc20 contract address.
    expect(trace[1].from).to.eq(ALITH_ADDRESS.toLowerCase());
    expect(trace[1].to).to.eq(erc20ContractAddress.toLowerCase());
    expect(trace[1].type).be.eq("CALL");
    // 3rd transaction is regular erc20 transfer.
    // - `From` is Alith's adddress.
    // - `To` is the erc20 contract address.
    expect(trace[2].from).to.eq(ALITH_ADDRESS.toLowerCase());
    expect(trace[2].to).to.eq(erc20ContractAddress.toLowerCase());
    expect(trace[2].type).be.eq("CALL");
  });
});
