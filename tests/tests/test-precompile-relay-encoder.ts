import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";
import { getCompiled } from "../util/contracts";
import { createContract, createContractExecution, GENESIS_TRANSACTION } from "../util/transactions";


import {
  GENESIS_ACCOUNT,
  ALITH,
  BALTATHAR,
  ALITH_PRIV_KEY,
  CHARLETH,
  BALTATHAR_PRIV_KEY,
} from "../util/constants";

import { createTransaction } from "../util/transactions";

const ADDRESS_RELAY_ENCODER = "0x0000000000000000000000000000000000000805";
const ALICE = "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const BOB = "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";
const SELECTORS = {
  bond: "31627376",
  bond_more: "49def326",
  chill: "bc4b2187",
  unbond: "2cd61217",
  withdrawUnbonded: "2d220331",
  validate: "3a0d803a",
  nominate: "a7cb124b",
  setPayee: "9801b147",
  setController: "7a8f48c2",
  rebond: "add6b3bf",
};

const GAS_PRICE = "0x" + (1_000_000_000).toString(16);

describeDevMoonbeam("Precompiles - relay-encoder", (context) => {
  let contract;
  before("Deploy contract", async () => {
    const { contract, rawTx } = await createContract(context.web3, "RelayStakeEncoder");
  });
  it("allows to get encoding of bond stake call", async function () {
    // 100 units
    const amount = `64`.padStart(64, "0");
    // Offset relative of the bytes object
    const offset = `20`.padStart(64, "0");
    // length of the bytes object
    const length = `01`.padStart(64, "0");
    // controller enum (position 2)
    const bytes = `02`.padEnd(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.bond}${ALICE}${amount}${offset}${length}${bytes}`,
      },
    ]);
    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000026" +
        "070000d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a5" +
        "6da27d9101020000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of bond_more stake call", async function () {
    // 100 units
    const amount = `64`.padStart(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.bond_more}${amount}`,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000004" +
        "0701910100000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of unbond stake call", async function () {

    // 100 units
    const amount = `64`.padStart(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.unbond}${amount}`,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000004" +
        "0702910100000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of chill stake call", async function () {
    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.chill}`,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000002" +
        "0706000000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of withdraw_unbonded stake call", async function () {
    const slashingSpans = `64`.padStart(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.withdrawUnbonded}${slashingSpans}`,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000006" +
        "0703640000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of validate stake call", async function () {
    // this is parts per billion. we are going to set it to 10%, i.e., 100000000
    const comission = `5F5E100`.padStart(64, "0");
    // this is for the blocked boolean. We set it to false
    const blocked = `0`.padStart(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.validate}${comission}${blocked}`,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000007" +
        "07040284d7170000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of nominate stake call", async function () {
    // we need to construct the input, which is an array of addresses
    // Offset relative of the vector object
    const offset = `20`.padStart(64, "0");
    // length of the vec. we will use a 2 length array, alice and bob
    const length = `02`.padStart(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.nominate}${offset}${length}${ALICE}${BOB}`,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000045" +
        "07050800d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7" +
        "a56da27d008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa" +
        "4794f26a48000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of set_payee stake call", async function () {
    // Offset relative of the bytes object
    const offset = `20`.padStart(64, "0");
    // length of the bytes object
    const length = `01`.padStart(64, "0");
    // controller enum (position 2)
    const bytes = `02`.padEnd(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.setPayee}${offset}${length}${bytes}`,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000003" +
        "0707020000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of set_controller stake call", async function () {
    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.setController}${ALICE}`,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000023" +
        "070800d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a5" +
        "6da27d0000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of rebond stake call", async function () {
    // 100 units
    const amount = `64`.padStart(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: `0x${SELECTORS.rebond}${amount}`,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000004" +
        "0713910100000000000000000000000000000000000000000000000000000000"
    );
  });
});
