import "@moonbeam-network/api-augment";
import { expectEVMResult } from "../../util/eth-transactions";
import { Contract } from "web3-eth-contract";
import testInputs from "../../util/artefacts/modexp.json";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import {
  createContract,
  createContractExecution,
  createTransaction,
} from "../../util/transactions";
import { ALITH_PRIVATE_KEY, alith, ALITH_ADDRESS } from "../../util/accounts";
import { EXTRINSIC_GAS_LIMIT } from "../../util/constants";
import { customWeb3Request } from "../../util/providers";
import { expect } from "chai";
import { hexToU8a, u8aToHex } from "@polkadot/util";

const MODEXP_PRECOMPILE_ADDRESS = "0x0000000000000000000000000000000000000005";

describeDevMoonbeam("Precompiles - modexp", (context) => {
  let hasherContract: Contract;

  before(async function () {
    const { contract, rawTx } = await createContract(context, "HasherChecker");
    await context.createBlock(rawTx);
    hasherContract = contract;
  });

  it("should be accessible from a smart contract", async function () {
    const { result } = await context.createBlock(
      createContractExecution(context, {
        contract: hasherContract,
        contractCall: hasherContract.methods.modExpChecker(),
      })
    );

    expectEVMResult(result.events, "Succeed");
  });

  it("EIP example 1 - calculation", async function () {
    const tx = await createContractExecution(context, {
      contract: hasherContract,
      contractCall: hasherContract.methods.modExpVerify(
        "3",
        "115792089237316195423570985008687907853269984665640564039457584007908834671662",
        "115792089237316195423570985008687907853269984665640564039457584007908834671663"
      ),
    });
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);
    await context.createBlock();

    expect(await hasherContract.methods.getResult().call(), "Incorrect modexp result").to.be.equals(
      "1"
    );
  });

  it("EIP example 1 - gas", async function () {
    const expectedModExpGasCost = 1360;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000001" + // base length
      "0000000000000000000000000000000000000000000000000000000000000020" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000020" + // modulus length
      "03" + // base
      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e" + // exponent
      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"; // modulus
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();

    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("EIP example 2", async function () {
    const tx = await createContractExecution(context, {
      contract: hasherContract,
      contractCall: hasherContract.methods.modExpVerify(
        "0",
        "115792089237316195423570985008687907853269984665640564039457584007908834671662",
        "115792089237316195423570985008687907853269984665640564039457584007908834671663"
      ),
    });
    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx]);
    await context.createBlock();
    expect(await hasherContract.methods.getResult().call(), "Incorrect modexp result").to.be.equals(
      "0"
    );
  });

  it("EIP example 2 - gas", async function () {
    const expectedModExpGasCost = 1360;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000000" + // base length
      "0000000000000000000000000000000000000000000000000000000000000020" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000020" + // modulus length
      // base length is zero so value is inferred zero
      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e" + // exponent
      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"; // modulus
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-1-square - gas", async function () {
    const expectedModExpGasCost = 200;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000040" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000040" + // modulus length
      testInputs["nagydani-1-square"].base +
      testInputs["nagydani-1-square"].exponent +
      testInputs["nagydani-1-square"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-1-qube - gas", async function () {
    const expectedModExpGasCost = 200;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000040" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000040" + // modulus length
      testInputs["nagydani-1-square"].base +
      testInputs["nagydani-1-qube"].exponent +
      testInputs["nagydani-1-qube"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-1-pow0x10001 - gas", async function () {
    const expectedModExpGasCost = 341;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000040" + // base length
      "0000000000000000000000000000000000000000000000000000000000000003" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000040" + // modulus length
      testInputs["nagydani-1-pow0x10001"].base +
      testInputs["nagydani-1-pow0x10001"].exponent +
      testInputs["nagydani-1-pow0x10001"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-2-square - gas", async function () {
    const expectedModExpGasCost = 200;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000080" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000080" + // modulus length
      testInputs["nagydani-2-square"].base +
      testInputs["nagydani-2-square"].exponent +
      testInputs["nagydani-2-square"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-2-qube - gas", async function () {
    const expectedModExpGasCost = 200;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000080" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000080" + // modulus length
      testInputs["nagydani-2-qube"].base +
      testInputs["nagydani-2-qube"].exponent +
      testInputs["nagydani-2-qube"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-2-pow0x10001 - gas", async function () {
    const expectedModExpGasCost = 1365;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000080" + // base length
      "0000000000000000000000000000000000000000000000000000000000000003" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000080" + // modulus length
      testInputs["nagydani-2-pow0x10001"].base +
      testInputs["nagydani-2-pow0x10001"].exponent +
      testInputs["nagydani-2-pow0x10001"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-3-square - gas", async function () {
    const expectedModExpGasCost = 341;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000100" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000100" + // modulus length
      testInputs["nagydani-3-square"].base +
      testInputs["nagydani-3-square"].exponent +
      testInputs["nagydani-3-square"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-3-qube - gas", async function () {
    const expectedModExpGasCost = 341;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000100" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000100" + // modulus length
      testInputs["nagydani-3-qube"].base +
      testInputs["nagydani-3-qube"].exponent +
      testInputs["nagydani-3-qube"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-3-pow0x10001 - gas", async function () {
    const expectedModExpGasCost = 5461;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000100" + // base length
      "0000000000000000000000000000000000000000000000000000000000000003" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000100" + // modulus length
      testInputs["nagydani-3-pow0x10001"].base +
      testInputs["nagydani-3-pow0x10001"].exponent +
      testInputs["nagydani-3-pow0x10001"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-4-square - gas", async function () {
    const expectedModExpGasCost = 1365;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000200" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000200" + // modulus length
      testInputs["nagydani-4-square"].base +
      testInputs["nagydani-4-square"].exponent +
      testInputs["nagydani-4-square"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-4-qube - gas", async function () {
    const expectedModExpGasCost = 1365;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000200" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000200" + // modulus length
      testInputs["nagydani-4-qube"].base +
      testInputs["nagydani-4-qube"].exponent +
      testInputs["nagydani-4-qube"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-4-pow0x10001 - gas", async function () {
    const expectedModExpGasCost = 21845;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000200" + // base length
      "0000000000000000000000000000000000000000000000000000000000000003" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000200" + // modulus length
      testInputs["nagydani-4-pow0x10001"].base +
      testInputs["nagydani-4-pow0x10001"].exponent +
      testInputs["nagydani-4-pow0x10001"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-5-square - gas", async function () {
    const expectedModExpGasCost = 5461;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000400" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000400" + // modulus length
      testInputs["nagydani-5-square"].base +
      testInputs["nagydani-5-square"].exponent +
      testInputs["nagydani-5-square"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-5-qube - gas", async function () {
    const expectedModExpGasCost = 5461;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000400" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000400" + // modulus length
      testInputs["nagydani-5-qube"].base +
      testInputs["nagydani-5-qube"].exponent +
      testInputs["nagydani-5-qube"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("nagydani-5-pow0x10001 - gas", async function () {
    const expectedModExpGasCost = 87381;
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000400" + // base length
      "0000000000000000000000000000000000000000000000000000000000000003" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000400" + // modulus length
      testInputs["nagydani-5-pow0x10001"].base +
      testInputs["nagydani-5-pow0x10001"].exponent +
      testInputs["nagydani-5-pow0x10001"].modulus;
    const byteArray = hexToU8a(inputData);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("Exponent > 32", async function () {
    // We multiply by a factor of 20 for an even mod.
    // See https://github.com/paritytech/frontier/pull/1017
    const expectedModExpGasCost = 7104 * 20;
    const byteArray = new Uint8Array([
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x26, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x60, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x10, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0x02, 0x0, 0x0, 0xb3, 0x0, 0x0, 0x02,
      0x0, 0x0, 0x7a, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff, 0xfb, 0x0, 0x0, 0x0, 0x0, 0x04,
      0x26, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 96, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x10, 0x0, 0x0, 0x0, 0xff, 0xff,
      0xff, 0x02, 0x0, 0x0, 0xb3, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
      0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xff, 0xff, 0xff, 0xff, 0xf9,
    ]);
    const inputData = u8aToHex(byteArray);
    const inputLength = byteArray.length;
    const numZeroBytes = byteArray.filter((a) => a == 0).length;
    const numNonZeroBytes = inputLength - numZeroBytes;
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
    const modExpGas = receipt.gasUsed - numNonZeroBytes * 16 - numZeroBytes * 4 - 21000;
    expect(modExpGas, "ModExp gas pricing mismatch").to.equal(expectedModExpGasCost);
  });

  it("should pad input when too short", async function () {
    const inputData =
      "0000000000000000000000000000000000000000000000000000000000000001" + // base length
      "0000000000000000000000000000000000000000000000000000000000000001" + // exponent length
      "0000000000000000000000000000000000000000000000000000000000000002" + // modulus length
      "05" + // base
      "03" + // exponent
      "01"; // modulus

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: alith.address,
        to: MODEXP_PRECOMPILE_ADDRESS,
        gas: EXTRINSIC_GAS_LIMIT.toString(),
        value: "0x00",
        data: "0x" + inputData,
      },
      ALITH_PRIVATE_KEY
    );

    const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    await context.createBlock();

    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.be.true;
  });
});
