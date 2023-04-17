import "@moonbeam-network/api-augment";
import { bnToHex } from "@polkadot/util";

import { expect } from "chai";
import { ethers } from "ethers";
import { Contract } from "web3-eth-contract";

import { ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS } from "../../util/accounts";
import { PRECOMPILE_XTOKENS_ADDRESS } from "../../util/constants";
import { web3EthCall } from "../../util/providers";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";
import { expectEVMResult } from "../../util/eth-transactions";
import {
  injectHrmpMessage,
  RawXcmMessage,
  sovereignAccountOfSibling,
  XcmFragment,
} from "../../util/xcm";

const ERC20_CONTRACT = getCompiled("ERC20WithInitialSupply");
const ERC20_INTERFACE = new ethers.utils.Interface(ERC20_CONTRACT.contract.abi);
const ERC20_TOTAL_SUPPLY = 1_000_000_000n;
const XTOKENS_CONTRACT = getCompiled("XtokensInstance");
const XTOKENS_INTERFACE = new ethers.utils.Interface(XTOKENS_CONTRACT.contract.abi);

async function getBalance(context: DevTestContext, blockHeight: number, address: string) {
  const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockHeight);
  const account = await context.polkadotApi.query.system.account.at(blockHash, address);
  return account.data.free.toBigInt();
}

const setupErc20Contract = async (context: DevTestContext, name: string, symbol: string) => {
  const { contract, contractAddress, rawTx } = await createContract(
    context,
    "ERC20WithInitialSupply",
    {
      ...ALITH_TRANSACTION_TEMPLATE,
      gas: 5_000_000,
    },
    [name, symbol, ALITH_ADDRESS, ERC20_TOTAL_SUPPLY]
  );
  const { result } = await context.createBlock(rawTx);
  expectEVMResult(result.events, "Succeed");
  return { contract, contractAddress };
};

describeDevMoonbeam("Mock XCM - Send local erc20", (context) => {
  let erc20Contract: Contract;
  let erc20ContractAddress: string;

  before("Should deploy erc20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "MyToken", "TKN");
    erc20Contract = contract;
    erc20ContractAddress = contractAddress;
  });

  it("Should be able to transfer ERC20 token throught xcm with xtoken precomp", async function () {
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
    expect(await getBalance(context, 2, ALITH_ADDRESS)).to.equal(
      (await getBalance(context, 1, ALITH_ADDRESS)) - fees
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

describeDevMoonbeam("Mock XCM - Receice back erc20", (context) => {
  let erc20Contract: Contract;
  let erc20ContractAddress: string;

  before("Should deploy erc20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "MyToken", "TKN");
    erc20Contract = contract;
    erc20ContractAddress = contractAddress;
  });

  it("Should be able to transfer ERC20 token throught incoming XCM message", async function () {
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
    const config = {
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

describeDevMoonbeam("Mock XCM - Send two ERC20", (context) => {
  let erc20Contract1: Contract;
  let erc20ContractAddress1: string;

  let erc20Contract2: Contract;
  let erc20ContractAddress2: string;

  before("Should deploy the first erc20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "FirstToken", "FTK");
    erc20Contract1 = contract;
    erc20ContractAddress1 = contractAddress;
  });

  before("Should deploy the second erc20 contract", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context, "SecondToken", "STK");
    erc20Contract2 = contract;
    erc20ContractAddress2 = contractAddress;
  });

  it("Should be able to transfer two ERC20 tokens throught incoming XCM meesage", async function () {
    this.timeout(20_000);
    const paraId = 888;
    const paraSovereign = sovereignAccountOfSibling(context, paraId);
    const amountTransferredOf1 = 1_000_000n;
    const amountTransferredOf2 = 1_000_000n;

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

    // Check the sovereign account has reveived erc20 tokens
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress1,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [paraSovereign]),
        })
      ).result
    ).equals(bnToHex(amountTransferredOf1, { bitLength: 256 }));

    // Send some erc20 tokens (of second contract) to the sovereign account of paraId
    const { result: result3 } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: erc20ContractAddress2,
        data: ERC20_INTERFACE.encodeFunctionData("transfer", [paraSovereign, amountTransferredOf2]),
      })
    );
    expectEVMResult(result3.events, "Succeed");

    // Check the sovereign account has reveived erc20 tokens
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress2,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [paraSovereign]),
        })
      ).result
    ).equals(bnToHex(amountTransferredOf2, { bitLength: 256 }));

    // Create the first xcm message to send the first ERC20 to Charleth
    const config = {
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
                    key: erc20ContractAddress1,
                  },
                },
              ],
            },
          },
          fungible: amountTransferredOf1,
        }
      ],
      beneficiary: CHARLETH_ADDRESS,
    };

    // Build the first xcm message
    const xcmMessage = new XcmFragment(config)
      .withdraw_asset()
      .clear_origin()
      .buy_execution()
      .deposit_asset(2n)
      .as_v2();
    
    // Mock the reception of the first xcm message
    await injectHrmpMessage(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);
    await context.createBlock();

    // Create another xcm message to send the second ERC20 to Baltathar
    const config2 = {
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
        }
      ],
      beneficiary: BALTATHAR_ADDRESS,
    };

    // Build the second xcm message
    const xcmMessage2 = new XcmFragment(config2)
      .withdraw_asset()
      .clear_origin()
      .buy_execution()
      .deposit_asset(2n)
      .as_v2();
    
    // Mock the reception of the second xcm message
    await injectHrmpMessage(context, paraId, {
      type: "XcmVersionedXcm",
      payload: xcmMessage2,
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

    // Erc20 tokens (of second contract) should have been received in Baltathar's address
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress2,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [BALTATHAR_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(amountTransferredOf2, { bitLength: 256 }));
  })
});

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
    const config = {
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
                    key: erc20ContractAddress,
                  },
                },
              ],
            },
          },
          fungible: amountTransferred,
        }
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

    const records = (await context.polkadotApi.query.system.events()) as any;
    const events = records.filter(
      ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
    );
    expect(events).to.have.lengthOf(1);

    // Charleth should not receive ERC20 tokens due to failed execution
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [CHARLETH_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(0n, { bitLength: 256 }));
  })
});
