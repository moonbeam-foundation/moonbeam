import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { ethers } from "ethers";
import { getCompiled } from "../../util/contracts";
import { createContract, createTransaction } from "../../util/transactions";
import { Keyring } from "@polkadot/api";
import { randomAsHex } from "@polkadot/util-crypto";
import { u8aToHex } from "@polkadot/util";

import { DEFAULT_GENESIS_MAPPING, PRECOMPILE_AUTHOR_MAPPING_ADDRESS } from "../../util/constants";
import { alith, ALITH_PRIVATE_KEY } from "../../util/accounts";

async function getMappingInfo(
  context,
  authorId: string
): Promise<{ account: string; deposit: BigInt }> {
  const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(authorId);
  if (mapping.isSome) {
    return {
      account: mapping.unwrap().account.toString(),
      deposit: mapping.unwrap().deposit.toBigInt(),
    };
  }
  return null;
}

describeDevMoonbeamAllEthTxTypes("Precompiles - author mapping", (context) => {
  it("allows to add association", async function () {
    const contractData = await getCompiled("AuthorMapping");
    const iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context, "AuthorMapping");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });
    const seed = randomAsHex(32);

    const mappingKeyRing = new Keyring({ type: "sr25519" });
    // add the account
    let mappingAccount = await mappingKeyRing.addFromUri(seed, null, "sr25519");

    const data = iFace.encodeFunctionData(
      // action
      "add_association",
      [mappingAccount.publicKey]
    );

    const base_fee = await context.web3.eth.getGasPrice();

    const tx = await createTransaction(context, {
      from: alith.address,
      privateKey: ALITH_PRIVATE_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: base_fee,
      to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
      data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    expect(receipt.status).to.equal(true);

    const registerInfo = await getMappingInfo(context, u8aToHex(mappingAccount.publicKey));
    expect(await registerInfo.account).to.eq(alith.address);
    expect(await registerInfo.deposit).to.eq(DEFAULT_GENESIS_MAPPING);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - author mapping", (context) => {
  let firstMappingAccount, secondMappingAccount;
  before("First add association", async () => {
    // We will work with genesis account

    // lets generate 2 seeds for 2 sr25519 addresses
    const seed = randomAsHex(32);
    const seed2 = randomAsHex(32);

    const mappingKeyRing = new Keyring({ type: "sr25519" });
    // accounts
    firstMappingAccount = await mappingKeyRing.addFromUri(seed, null, "sr25519");
    secondMappingAccount = await mappingKeyRing.addFromUri(seed2, null, "sr25519");

    // Add association
    await context.polkadotApi.tx.authorMapping
      .addAssociation(firstMappingAccount.publicKey)
      .signAndSend(alith);
    await context.createBlock();

    // Verify association was added
    const registerInfo = await getMappingInfo(context, u8aToHex(firstMappingAccount.publicKey));
    expect(await registerInfo.account).to.eq(alith.address);
    expect(await registerInfo.deposit).to.eq(DEFAULT_GENESIS_MAPPING);
  });
  it("allows to update association", async function () {
    const contractData = await getCompiled("AuthorMapping");
    const iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context, "AuthorMapping");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });

    const data = iFace.encodeFunctionData(
      // action
      "update_association",
      [firstMappingAccount.publicKey, secondMappingAccount.publicKey]
    );

    const base_fee = await context.web3.eth.getGasPrice();

    const tx = await createTransaction(context, {
      from: alith.address,
      privateKey: ALITH_PRIVATE_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: base_fee,
      to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
      data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    expect(receipt.status).to.equal(true);

    // Verify we updated firstMappingAccount for secondMappingAccount
    const secondRegisterInfo = await getMappingInfo(
      context,
      u8aToHex(secondMappingAccount.publicKey)
    );
    expect(await secondRegisterInfo.account).to.eq(alith.address);
    expect(await secondRegisterInfo.deposit).to.eq(DEFAULT_GENESIS_MAPPING);

    const firstRegisterInfo = await getMappingInfo(
      context,
      u8aToHex(firstMappingAccount.publicKey)
    );
    expect(firstRegisterInfo).to.eq(null);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - author mapping", (context) => {
  let mappingAccount;
  before("First add association", async () => {
    const seed = randomAsHex(32);
    const mappingKeyRing = new Keyring({ type: "sr25519" });
    // account
    mappingAccount = await mappingKeyRing.addFromUri(seed, null, "sr25519");

    // Add association
    await context.polkadotApi.tx.authorMapping
      .addAssociation(mappingAccount.publicKey)
      .signAndSend(alith);
    await context.createBlock();

    // Verify association was added
    const registerInfo = await getMappingInfo(context, u8aToHex(mappingAccount.publicKey));
    expect(await registerInfo.account).to.eq(alith.address);
    expect(await registerInfo.deposit).to.eq(DEFAULT_GENESIS_MAPPING);
  });
  it("allows to clear association", async function () {
    const contractData = await getCompiled("AuthorMapping");
    const iFace = new ethers.utils.Interface(contractData.contract.abi);
    const { contract, rawTx } = await createContract(context, "AuthorMapping");
    const address = contract.options.address;
    await context.createBlock({ transactions: [rawTx] });

    const data = iFace.encodeFunctionData(
      // action
      "clear_association",
      [mappingAccount.publicKey]
    );

    const base_fee = await context.web3.eth.getGasPrice();

    const tx = await createTransaction(context, {
      from: alith.address,
      privateKey: ALITH_PRIVATE_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: base_fee,
      to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
      data,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    expect(receipt.status).to.equal(true);

    // Verify we removed the association
    const firstRegisterInfo = await getMappingInfo(context, u8aToHex(mappingAccount.publicKey));
    expect(firstRegisterInfo).to.eq(null);
  });
});
