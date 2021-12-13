import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { ethers } from "ethers";
import { getCompiled } from "../../util/contracts";
import { createContract, createTransaction } from "../../util/transactions";

import { GAS_PRICE, GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../../util/constants";
import { verifyLatestBlockFees } from "../../util/block";

const ADDRESS_XTOKENS = "0x0000000000000000000000000000000000000804";
const BALANCES_ADDRESS = "0x0000000000000000000000000000000000000802";

async function getBalance(context, blockHeight, address) {
  const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockHeight);
  const account = await context.polkadotApi.query.system.account.at(blockHash, address);
  return account.data.free;
}

describeDevMoonbeam("Precompiles - xtokens", (context) => {
  it("allows to issue transfer xtokens", async function () {
    const contractData = await getCompiled("XtokensInstance");
    const iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context.web3, "XtokensInstance");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });
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
    let destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];
    // 1000 units
    let amountTransferred = 1000;

    // weight
    let weight = 100;

    const data = iFace.encodeFunctionData(
      // action
      "transfer",
      [
        // address of the multiasset, in this case our own balances
        BALANCES_ADDRESS,
        // amount
        amountTransferred,
        // Destination as multilocation
        destination,
        // weight
        weight,
      ]
    );

    const tx = await createTransaction(context.web3, {
      from: GENESIS_ACCOUNT,
      privateKey: GENESIS_ACCOUNT_PRIVATE_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_XTOKENS,
      data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    const fees = BigInt(receipt.gasUsed) * BigInt(GAS_PRICE);

    // our tokens + fees should have been spent
    expect(BigInt(await getBalance(context, 2, GENESIS_ACCOUNT))).to.equal(
      BigInt(await getBalance(context, 1, GENESIS_ACCOUNT)) -
        BigInt(amountTransferred) -
        BigInt(fees)
    );
    await verifyLatestBlockFees(context.polkadotApi, expect, BigInt(amountTransferred));
  });
});

describeDevMoonbeam("Precompiles - xtokens", (context) => {
  it("allows to issue transfer xtokens with fee", async function () {
    const contractData = await getCompiled("XtokensInstance");
    const iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context.web3, "XtokensInstance");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });
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
    let destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];
    // 1000 units
    let amountTransferred = 1000;

    // 100 units
    let fee = 100;

    // weight
    let weight = 100;

    const data = iFace.encodeFunctionData(
      // action
      "transfer_with_fee",
      [
        // address of the multiasset, in this case our own balances
        BALANCES_ADDRESS,
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

    const base_fee = await context.web3.eth.getGasPrice();

    const tx = await createTransaction(context.web3, {
      from: GENESIS_ACCOUNT,
      privateKey: GENESIS_ACCOUNT_PRIVATE_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_XTOKENS,
      data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    const fees = BigInt(receipt.gasUsed) * BigInt(GAS_PRICE);

    // our tokens + fees should have been spent
    expect(BigInt(await getBalance(context, 2, GENESIS_ACCOUNT))).to.equal(
      BigInt(await getBalance(context, 1, GENESIS_ACCOUNT)) -
        BigInt(amountTransferred) -
        BigInt(fee) -
        BigInt(fees)
    );
    await verifyLatestBlockFees(context.polkadotApi, expect, BigInt(amountTransferred+fee));
  });
});

describeDevMoonbeam("Precompiles - xtokens", (context) => {
  it("allows to issue transfer_multiasset xtokens", async function () {
    const contractData = await getCompiled("XtokensInstance");
    const iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context.web3, "XtokensInstance");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });
    // Junction::AccountId32
    const destination_enum_selector = "0x01";
    // [0x01; 32]
    const destination_address = "0101010101010101010101010101010101010101010101010101010101010101";
    // NetworkId::Any
    const destination_network_id = "00";

    // Junction::Parachain(0)
    const x2_parachain_asset_enum_selector = "0x00";
    const x2_parachain_id = "00000000";

    // Junction::PalletInstance(3)
    const x2_pallet_instance_enum_selector = "0x04";
    const x2_instance = "03";

    // This represents X3(Parent, Parachain(1000), PalletInstance(3)))

    // This multilocation represents our native token
    let asset = [
      // one parent
      1,
      // X2(Parachain, PalletInstance)
      // Parachain: Parachain selector (00) + parachain id (0) in 4 bytes (00000000)
      // PalletInstance: Selector (04) + pallet instance 1 byte (03)
      [
        x2_parachain_asset_enum_selector + x2_parachain_id,
        x2_pallet_instance_enum_selector + x2_instance,
      ],
    ];
    // This represents X2(Parent, AccountId32([0x01; 32]))
    // We will transfer the tokens the former account in the relay chain
    // However it does not really matter as we are not testing what happens
    // in the relay side of things
    let destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];
    // 1000 units
    let amountTransferred = 1000;

    // weight
    let weight = 100;

    // encode the input with ethers
    const data = iFace.encodeFunctionData(
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

    // create tx
    const tx = await createTransaction(context.web3, {
      from: GENESIS_ACCOUNT,
      privateKey: GENESIS_ACCOUNT_PRIVATE_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_XTOKENS,
      data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    const fees = BigInt(receipt.gasUsed) * BigInt(GAS_PRICE);

    // our tokens + fees should have been spent
    expect(BigInt(await getBalance(context, 2, GENESIS_ACCOUNT))).to.equal(
      BigInt(await getBalance(context, 1, GENESIS_ACCOUNT)) -
        BigInt(amountTransferred) -
        BigInt(fees)
    );
    await verifyLatestBlockFees(context.polkadotApi, expect, BigInt(amountTransferred));
  });
});

describeDevMoonbeam("Precompiles - xtokens", (context) => {
  it("allows to issue transfer_multiasset xtokens with fee", async function () {
    const contractData = await getCompiled("XtokensInstance");
    const iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context.web3, "XtokensInstance");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });
    // Junction::AccountId32
    const destination_enum_selector = "0x01";
    // [0x01; 32]
    const destination_address = "0101010101010101010101010101010101010101010101010101010101010101";
    // NetworkId::Any
    const destination_network_id = "00";

    // Junction::Parachain(0)
    const x2_parachain_asset_enum_selector = "0x00";
    const x2_parachain_id = "00000000";

    // Junction::PalletInstance(3)
    const x2_pallet_instance_enum_selector = "0x04";
    const x2_instance = "03";

    // This represents X3(Parent, Parachain(1000), PalletInstance(3)))

    // This multilocation represents our native token
    let asset = [
      // one parent
      1,
      // X2(Parachain, PalletInstance)
      // Parachain: Parachain selector (00) + parachain id (0) in 4 bytes (00000000)
      // PalletInstance: Selector (04) + pallet instance 1 byte (03)
      [
        x2_parachain_asset_enum_selector + x2_parachain_id,
        x2_pallet_instance_enum_selector + x2_instance,
      ],
    ];
    // This represents X2(Parent, AccountId32([0x01; 32]))
    // We will transfer the tokens the former account in the relay chain
    // However it does not really matter as we are not testing what happens
    // in the relay side of things
    let destination =
      // Destination as multilocation
      [
        // one parent
        1,
        // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
        [destination_enum_selector + destination_address + destination_network_id],
      ];
    // 1000 units
    let amountTransferred = 1000;

    // 100 units
    let fee = 1000;

    // weight
    let weight = 100;

    // encode the input with ethers
    const data = iFace.encodeFunctionData(
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
    const tx = await createTransaction(context.web3, {
      from: GENESIS_ACCOUNT,
      privateKey: GENESIS_ACCOUNT_PRIVATE_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_XTOKENS,
      data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    const fees = BigInt(receipt.gasUsed) * BigInt(GAS_PRICE);

    // our tokens + fees should have been spent
    expect(BigInt(await getBalance(context, 2, GENESIS_ACCOUNT))).to.equal(
      BigInt(await getBalance(context, 1, GENESIS_ACCOUNT)) -
        BigInt(amountTransferred) -
        BigInt(fee) -
        BigInt(fees)
    );
    await verifyLatestBlockFees(context.polkadotApi, expect, BigInt(amountTransferred+fee));
  });
});
