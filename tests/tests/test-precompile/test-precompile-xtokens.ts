import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import { alith } from "../../util/accounts";
import { verifyLatestBlockFees } from "../../util/block";
import {
  MIN_GAS_PRICE,
  PRECOMPILE_NATIVE_ERC20_ADDRESS,
  PRECOMPILE_XTOKENS_ADDRESS,
} from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeamAllEthTxTypes, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";

const XTOKENS_CONTRACT = getCompiled("XtokensInstance");
const XTOKENS_INTERFACE = new ethers.utils.Interface(XTOKENS_CONTRACT.contract.abi);

async function getBalance(context: DevTestContext, blockHeight: number, address: string) {
  const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockHeight);
  const account = await context.polkadotApi.query.system.account.at(blockHash, address);
  return account.data.free.toBigInt();
}

describeDevMoonbeamAllEthTxTypes("Precompiles - xtokens", (context) => {
  it("allows to issue transfer xtokens", async function () {
    const { rawTx } = await createContract(context, "XtokensInstance");
    await context.createBlock(rawTx);
    // Junction::AccountId32
    const destination_enum_selector = "0x01";
    // [0x01; 32]
    const destination_address = "0101010101010101010101010101010101010101010101010101010101010101";
    // NetworkId::Any
    const destination_network_id = "00";

    // This represents X2(Parent, AccountId32([0x01; 32]))
    // We will transfer the tokens the former account in the relay chain
    // However it does not really matter as we are not testing what happens
    // in the relay side of things
    const destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];
    // 1000 units
    const amountTransferred = 1000n;

    // weight
    const weight = 100n;

    const data = XTOKENS_INTERFACE.encodeFunctionData(
      // action
      "transfer",
      [
        // address of the multiasset, in this case our own balances
        PRECOMPILE_NATIVE_ERC20_ADDRESS,
        // amount
        amountTransferred,
        // Destination as multilocation
        destination,
        // weight
        weight,
      ]
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XTOKENS_ADDRESS,
        data,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    const fees = BigInt(receipt.gasUsed) * MIN_GAS_PRICE;

    // our tokens + fees should have been spent
    expect(await getBalance(context, 2, alith.address)).to.equal(
      (await getBalance(context, 1, alith.address)) - amountTransferred - fees
    );
    await verifyLatestBlockFees(context, amountTransferred);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xtokens", (context) => {
  it("allows to issue transfer xtokens with fee", async function () {
    const { rawTx } = await createContract(context, "XtokensInstance");
    await context.createBlock(rawTx);
    // Junction::AccountId32
    const destination_enum_selector = "0x01";
    // [0x01; 32]
    const destination_address = "0101010101010101010101010101010101010101010101010101010101010101";
    // NetworkId::Any
    const destination_network_id = "00";

    // This represents X2(Parent, AccountId32([0x01; 32]))
    // We will transfer the tokens the former account in the relay chain
    // However it does not really matter as we are not testing what happens
    // in the relay side of things
    const destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];
    // 1000 units
    const amountTransferred = 1000n;

    // 100 units
    const fee = 100n;

    // weight
    const weight = 100n;

    const data = XTOKENS_INTERFACE.encodeFunctionData(
      // action
      "transfer_with_fee",
      [
        // address of the multiasset, in this case our own balances
        PRECOMPILE_NATIVE_ERC20_ADDRESS,
        // amount
        amountTransferred,
        // fee
        fee,
        // Destination as multilocation
        destination,
        // weight
        weight,
      ]
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XTOKENS_ADDRESS,
        data,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    const fees = BigInt(receipt.gasUsed) * MIN_GAS_PRICE;

    // our tokens + fees should have been spent
    expect(await getBalance(context, 2, alith.address)).to.equal(
      (await getBalance(context, 1, alith.address)) - amountTransferred - BigInt(fee) - fees
    );
    await verifyLatestBlockFees(context, BigInt(amountTransferred + fee));
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xtokens", (context) => {
  it("allows to issue transfer_multiasset xtokens", async function () {
    const { rawTx } = await createContract(context, "XtokensInstance");
    await context.createBlock(rawTx);
    // Junction::AccountId32
    const destination_enum_selector = "0x01";
    // [0x01; 32]
    const destination_address = "0101010101010101010101010101010101010101010101010101010101010101";
    // NetworkId::Any
    const destination_network_id = "00";

    // Junction::PalletInstance(3)
    const x2_pallet_instance_enum_selector = "0x04";
    const x2_instance = "03";

    // This represents X1(PalletInstance(3)))

    // This multilocation represents our native token
    const asset = [
      // zero parents
      0,
      // X1(PalletInstance)
      // PalletInstance: Selector (04) + palconst instance 1 byte (03)
      [x2_pallet_instance_enum_selector + x2_instance],
    ];
    // This represents X2(Parent, AccountId32([0x01; 32]))
    // We will transfer the tokens the former account in the relay chain
    // However it does not really matter as we are not testing what happens
    // in the relay side of things
    const destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];
    // 1000 units
    const amountTransferred = 1000n;

    // weight
    const weight = 100;

    // encode the input with ethers
    const data = XTOKENS_INTERFACE.encodeFunctionData(
      // action
      "transfer_multiasset",
      [
        asset,
        // amount
        amountTransferred,
        destination,
        // weight
        weight,
      ]
    );

    const base_fee = await context.web3.eth.getGasPrice();

    // create tx
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XTOKENS_ADDRESS,
        data,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    const fees = BigInt(receipt.gasUsed) * MIN_GAS_PRICE;

    // our tokens + fees should have been spent
    expect(await getBalance(context, 2, alith.address)).to.equal(
      (await getBalance(context, 1, alith.address)) - amountTransferred - fees
    );
    await verifyLatestBlockFees(context, amountTransferred);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xtokens", (context) => {
  it("allows to issue transfer_multiasset xtokens with fee", async function () {
    const { rawTx } = await createContract(context, "XtokensInstance");
    await context.createBlock(rawTx);
    // Junction::AccountId32
    const destination_enum_selector = "0x01";
    // [0x01; 32]
    const destination_address = "0101010101010101010101010101010101010101010101010101010101010101";
    // NetworkId::Any
    const destination_network_id = "00";

    // Junction::PalletInstance(3)
    const x2_pallet_instance_enum_selector = "0x04";
    const x2_instance = "03";

    // This represents X1(PalletInstance(3)))

    // This multilocation represents our native token
    const asset = [
      // one parent
      0,
      // X1(PalletInstance)
      // PalletInstance: Selector (04) + palconst instance 1 byte (03)
      [x2_pallet_instance_enum_selector + x2_instance],
    ];
    // This represents X2(Parent, AccountId32([0x01; 32]))
    // We will transfer the tokens the former account in the relay chain
    // However it does not really matter as we are not testing what happens
    // in the relay side of things
    const destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];
    // 1000 units
    const amountTransferred = 1000n;

    // 100 units
    const fee = 100n;

    // weight
    const weight = 100;

    // encode the input with ethers
    const data = XTOKENS_INTERFACE.encodeFunctionData(
      // action
      "transfer_multiasset_with_fee",
      [
        asset,
        // amount
        amountTransferred,
        // fee
        fee,
        destination,
        // weight
        weight,
      ]
    );

    const base_fee = await context.web3.eth.getGasPrice();

    // create tx

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XTOKENS_ADDRESS,
        data,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    const fees = BigInt(receipt.gasUsed) * MIN_GAS_PRICE;

    // our tokens + fees should have been spent
    expect(await getBalance(context, 2, alith.address)).to.equal(
      (await getBalance(context, 1, alith.address)) - amountTransferred - BigInt(fee) - fees
    );
    await verifyLatestBlockFees(context, amountTransferred + fee);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xtokens", (context) => {
  it("allows to issue transfer multicurrencies xtokens", async function () {
    const { rawTx } = await createContract(context, "XtokensInstance");
    await context.createBlock(rawTx);
    // Junction::AccountId32
    const destination_enum_selector = "0x01";
    // [0x01; 32]
    const destination_address = "0101010101010101010101010101010101010101010101010101010101010101";
    // NetworkId::Any
    const destination_network_id = "00";
    // 1000 units
    const amountTransferred = 1000n;
    const currencies = [[PRECOMPILE_NATIVE_ERC20_ADDRESS, amountTransferred]];

    // This represents X2(Parent, AccountId32([0x01; 32]))
    // We will transfer the tokens the former account in the relay chain
    // However it does not really matter as we are not testing what happens
    // in the relay side of things
    const destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];

    // fee_item
    const fee_item = 0;

    // weight
    const weight = 100;

    const data = XTOKENS_INTERFACE.encodeFunctionData(
      // action
      "transfer_multi_currencies",
      [
        // currencies, only one in this case
        currencies,
        // fee_item
        fee_item,
        // Destination as multilocation
        destination,
        // weight
        weight,
      ]
    );
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XTOKENS_ADDRESS,
        data,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    const fees = BigInt(receipt.gasUsed) * MIN_GAS_PRICE;

    // our tokens + fees should have been spent
    expect(await getBalance(context, 2, alith.address)).to.equal(
      (await getBalance(context, 1, alith.address)) - amountTransferred - fees
    );
    await verifyLatestBlockFees(context, amountTransferred);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - xtokens", (context) => {
  it("allows to issue transfer multiassets xtokens", async function () {
    const { rawTx } = await createContract(context, "XtokensInstance");
    await context.createBlock(rawTx);
    // Junction::AccountId32
    const destination_enum_selector = "0x01";
    // [0x01; 32]
    const destination_address = "0101010101010101010101010101010101010101010101010101010101010101";
    // NetworkId::Any
    const destination_network_id = "00";
    // 1000 units
    const amountTransferred = 1000n;

    // Junction::PalletInstance(3)
    const x2_pallet_instance_enum_selector = "0x04";
    const x2_instance = "03";

    // This multilocation represents our native token
    const asset = [
      // one parent
      0,
      // X1(PalletInstance)
      // PalletInstance: Selector (04) + palconst instance 1 byte (03)
      [x2_pallet_instance_enum_selector + x2_instance],
    ];

    const multiassets = [[asset, amountTransferred]];

    // This represents X2(Parent, AccountId32([0x01; 32]))
    // We will transfer the tokens the former account in the relay chain
    // However it does not really matter as we are not testing what happens
    // in the relay side of things
    const destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];

    // fee_item
    const fee_item = 0;

    // weight
    const weight = 100;

    const data = XTOKENS_INTERFACE.encodeFunctionData(
      // action
      "transfer_multi_assets",
      [
        // assets, only one in this case
        multiassets,
        // fee_item
        fee_item,
        // Destination as multilocation
        destination,
        // weight
        weight,
      ]
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XTOKENS_ADDRESS,
        data,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    const fees = BigInt(receipt.gasUsed) * MIN_GAS_PRICE;

    // our tokens + fees should have been spent
    expect(await getBalance(context, 2, alith.address)).to.equal(
      (await getBalance(context, 1, alith.address)) - amountTransferred - fees
    );
    await verifyLatestBlockFees(context, amountTransferred);
  });
});
