import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import { ALITH_SESSION_ADDRESS, BALTATHAR_SESSION_ADDRESS } from "../../util/accounts";
import { PRECOMPILE_RELAY_ENCODER_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

const RELAY_ENCODER_CONTRACT = getCompiled("RelayEncoderInstance");
const RELAY_ENCODER_INTERFACE = new ethers.utils.Interface(RELAY_ENCODER_CONTRACT.contract.abi);

describeDevMoonbeamAllEthTxTypes("Precompiles - relay-encoder", (context) => {
  it("allows to get encoding of bond stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeBond", [100, 0x02]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000005" +
        "0600910102000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of bond_more stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeBondExtra", [100]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000004" +
        "0601910100000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of unbond stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeUnbond", [100]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000004" +
        "0602910100000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of chill stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeChill", []),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000002" +
        "0606000000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of withdraw_unbonded stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeWithdrawUnbonded", [100]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000006" +
        "0603640000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of validate stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeValidate", [100000000, false]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000007" +
        "06040284d7170000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of nominate stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeNominate", [
            [ALITH_SESSION_ADDRESS, BALTATHAR_SESSION_ADDRESS],
          ]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000045" +
        "06050800d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7" +
        "a56da27d008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa" +
        "4794f26a48000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of set_payee stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeSetPayee", [0x02]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000003" +
        "0607020000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of set_controller stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeSetController", []),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000002" +
        "0608000000000000000000000000000000000000000000000000000000000000"
    );
  });

  it("allows to get encoding of rebond stake call", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: PRECOMPILE_RELAY_ENCODER_ADDRESS,
          data: RELAY_ENCODER_INTERFACE.encodeFunctionData("encodeRebond", [100]),
        })
      ).result
    ).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000004" +
        "0613910100000000000000000000000000000000000000000000000000000000"
    );
  });
});
