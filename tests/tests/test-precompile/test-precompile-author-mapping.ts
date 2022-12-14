import "@moonbeam-network/api-augment";

import { u8aToHex } from "@polkadot/util";
import { KeyringPair } from "@substrate/txwrapper-core";
import { expect } from "chai";
import { ethers } from "ethers";

import { alith, generateKeyringPair } from "../../util/accounts";
import { DEFAULT_GENESIS_MAPPING, PRECOMPILE_AUTHOR_MAPPING_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";

const AUTHOR_MAPPING_CONTRACT = getCompiled("AuthorMapping");
const AUTHOR_MAPPING_INTERFACE = new ethers.utils.Interface(AUTHOR_MAPPING_CONTRACT.contract.abi);

describeDevMoonbeamAllEthTxTypes("Precompiles - author mapping", (context) => {
  it("allows to add association", async function () {
    const mappingAccount = generateKeyringPair("sr25519");
    const { rawTx } = await createContract(context, "AuthorMapping");
    await context.createBlock(rawTx);

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
        data: AUTHOR_MAPPING_INTERFACE.encodeFunctionData("addAssociation", [
          mappingAccount.publicKey,
        ]),
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);

    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      u8aToHex(mappingAccount.publicKey)
    );
    expect(mapping.unwrap().account.toString()).to.eq(alith.address);
    expect(mapping.unwrap().deposit.toBigInt()).to.eq(DEFAULT_GENESIS_MAPPING);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - author mapping", (context) => {
  let firstMappingAccount: KeyringPair;
  let secondMappingAccount: KeyringPair;
  before("First add association", async () => {
    firstMappingAccount = generateKeyringPair("sr25519");
    secondMappingAccount = generateKeyringPair("sr25519");
    // Add association
    await context.createBlock(
      context.polkadotApi.tx.authorMapping.addAssociation(firstMappingAccount.publicKey)
    );

    // Verify association was added
    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      u8aToHex(firstMappingAccount.publicKey)
    );
    expect(mapping.unwrap().account.toString()).to.eq(alith.address);
    expect(mapping.unwrap().deposit.toBigInt()).to.eq(DEFAULT_GENESIS_MAPPING);
  });

  it("allows to update association", async function () {
    const { rawTx } = await createContract(context, "AuthorMapping");
    await context.createBlock(rawTx);

    const tx = await createTransaction(context, {
      to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
      data: AUTHOR_MAPPING_INTERFACE.encodeFunctionData("updateAssociation", [
        firstMappingAccount.publicKey,
        secondMappingAccount.publicKey,
      ]),
    });

    const { result } = await context.createBlock(tx);
    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);

    // Verify we updated firstMappingAccount for secondMappingAccount
    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      u8aToHex(secondMappingAccount.publicKey)
    );
    expect(mapping.unwrap().account.toString()).to.eq(alith.address);
    expect(mapping.unwrap().deposit.toBigInt()).to.eq(DEFAULT_GENESIS_MAPPING);
    expect(
      (
        await context.polkadotApi.query.authorMapping.mappingWithDeposit(
          u8aToHex(firstMappingAccount.publicKey)
        )
      ).isNone
    ).to.be.true;
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - author mapping", (context) => {
  let mappingAccount: KeyringPair;
  before("First add association", async () => {
    mappingAccount = generateKeyringPair("sr25519");
    // Add association

    await context.createBlock(
      context.polkadotApi.tx.authorMapping.addAssociation(mappingAccount.publicKey)
    );

    // Verify association was added
    const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(
      u8aToHex(mappingAccount.publicKey)
    );
    expect(mapping.unwrap().account.toString()).to.eq(alith.address);
    expect(mapping.unwrap().deposit.toBigInt()).to.eq(DEFAULT_GENESIS_MAPPING);
  });
  it("allows to clear association", async function () {
    const { rawTx } = await createContract(context, "AuthorMapping");
    await context.createBlock(rawTx);

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
        data: AUTHOR_MAPPING_INTERFACE.encodeFunctionData("clearAssociation", [
          mappingAccount.publicKey,
        ]),
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);

    // Verify we removed the association
    expect(
      (
        await context.polkadotApi.query.authorMapping.mappingWithDeposit(
          u8aToHex(mappingAccount.publicKey)
        )
      ).isNone
    ).to.be.true;
  });
});
