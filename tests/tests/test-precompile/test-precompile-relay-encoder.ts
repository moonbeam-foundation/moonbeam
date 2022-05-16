import { expect } from "chai";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { customWeb3Request } from "../../util/providers";
import { ethers } from "ethers";
import { getCompiled } from "../../util/contracts";

import { GENESIS_ACCOUNT } from "../../util/constants";

const ADDRESS_RELAY_ENCODER = "0x0000000000000000000000000000000000000805";
const ALICE_HEX = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";

const BOB_HEX = "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";

const GAS_PRICE = "0x" + (1_000_000_000).toString(16);

describeDevMoonbeamAllEthTxTypes("Precompiles - relay-encoder", (context) => {
  let iFace;
  let contractData;
  before("Deploy contract", async () => {
    contractData = await getCompiled("RelayEncoderInstance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
  });
  it("allows to get encoding of bond stake call", async function () {
    const data = iFace.encodeFunctionData("encode_bond", [ALICE_HEX, 100, 0x02]);
    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: data,
      },
    ]);
    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000026" +
        "060000d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a5" +
        "6da27d9101020000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of bond_more stake call", async function () {
    const data = iFace.encodeFunctionData("encode_bond_extra", [100]);

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000004" +
        "0601910100000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of unbond stake call", async function () {
    const data = iFace.encodeFunctionData("encode_unbond", [100]);

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000004" +
        "0602910100000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of chill stake call", async function () {
    const data = iFace.encodeFunctionData("encode_chill", []);

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000002" +
        "0606000000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of withdraw_unbonded stake call", async function () {
    const data = iFace.encodeFunctionData("encode_withdraw_unbonded", [100]);
    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000006" +
        "0603640000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of validate stake call", async function () {
    const data = iFace.encodeFunctionData("encode_validate", [100000000, false]);

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
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000007" +
        "06040284d7170000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of nominate stake call", async function () {
    const data = iFace.encodeFunctionData("encode_nominate", [[ALICE_HEX, BOB_HEX]]);

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000045" +
        "06050800d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7" +
        "a56da27d008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa" +
        "4794f26a48000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of set_payee stake call", async function () {
    const data = iFace.encodeFunctionData("encode_set_payee", [0x02]);

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000003" +
        "0607020000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of set_controller stake call", async function () {
    const data = iFace.encodeFunctionData("encode_set_controller", [ALICE_HEX]);

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000023" +
        "060800d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a5" +
        "6da27d0000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of rebond stake call", async function () {
    const data = iFace.encodeFunctionData("encode_rebond", [100]);

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_RELAY_ENCODER,
        data: data,
      },
    ]);

    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000004" +
        "0613910100000000000000000000000000000000000000000000000000000000"
    );
  });
});
