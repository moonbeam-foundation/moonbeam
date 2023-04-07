import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { Contract, ethers } from "ethers";
import { getCompiled } from "../util/contracts";
import { ALITH_ADDRESS, ALITH_SESSION_ADDRESS, BALTATHAR_ADDRESS } from "../util/accounts";
import { hexToU8a, u8aToHex } from "@polkadot/util";
const debug = require("debug")("smoke:relay-indices");

describeSmokeSuite(
  "S1750",
  `Relay chain Module:Method indices should match our encoding`,
  (context, testIt) => {
    let relayEncoder: Contract;
    let xcmTransactorV1: Contract;
    let xcmTransactorV2: Contract;
    let rtVersion: number;

    before(async function () {
      if (process.env.SKIP_RELAY_TESTS) {
        debug(`SKIP_RELAY_TESTS=true, skipping test.`);
        this.skip();
      }

      if (typeof process.env.RELAY_WSS_URL === "undefined" || process.env.RELAY_WSS_URL === "") {
        debug(`RELAY_WSS_URL env var not supplied, skipping test.`);
        this.skip();
      }

      rtVersion = context.polkadotApi.consts.system.version.specVersion.toNumber();

      const RELAY_ENCODER_PRECOMPILE = "0x0000000000000000000000000000000000000805";
      const XCM_TRANSACTOR_V1_PRECOMPILE = "0x0000000000000000000000000000000000000806";
      const XCM_TRANSACTOR_V2_PRECOMPILE = "0x000000000000000000000000000000000000080D";

      const RELAY_ENCODER_CONTRACT_JSON = getCompiled("precompiles/relay-encoder/RelayEncoder");
      const RELAY_ENCODER_INTERFACE = new ethers.utils.Interface(
        RELAY_ENCODER_CONTRACT_JSON.contract.abi
      );

      const XCM_TRANSACTOR_V1_JSON = getCompiled(
        "precompiles/xcm-transactor/src/v1/XcmTransactorV1"
      );
      const XCM_TRANSACTOR_V1_INTERFACE = new ethers.utils.Interface(
        XCM_TRANSACTOR_V1_JSON.contract.abi
      );

      const XCM_TRANSACTOR_V2_JSON = getCompiled(
        "precompiles/xcm-transactor/src/v2/XcmTransactorV2"
      );
      const XCM_TRANSACTOR_V2_INTERFACE = new ethers.utils.Interface(
        XCM_TRANSACTOR_V2_JSON.contract.abi
      );

      relayEncoder = new ethers.Contract(
        RELAY_ENCODER_PRECOMPILE,
        RELAY_ENCODER_INTERFACE,
        context.ethers
      );

      xcmTransactorV1 = new ethers.Contract(
        XCM_TRANSACTOR_V1_PRECOMPILE,
        XCM_TRANSACTOR_V1_INTERFACE,
        context.ethers
      );

      xcmTransactorV2 = new ethers.Contract(
        XCM_TRANSACTOR_V2_PRECOMPILE,
        XCM_TRANSACTOR_V2_INTERFACE,
        context.ethers
      );
    });

    testIt("C100", "should have matching indices for HRMP.InitOpenChannel", async function () {
      if (rtVersion < 2100) {
        debug(`Runtime version is ${rtVersion}, which is less than 2100. Skipping test. `);
        this.skip();
      }
      const callHex = context.relayApi.tx.hrmp
        .hrmpInitOpenChannel(2000, 1000, 102400)
        .method.toHex();
      const resp = await relayEncoder.encodeHrmpInitOpenChannel(2000, 1000, 102400);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C200", "should have matching indices for HRMP.AcceptOpenChannel", async function () {
      if (rtVersion < 2100) {
        debug(`Runtime version is ${rtVersion}, which is less than 2100. Skipping test. `);
        this.skip();
      }
      const callHex = context.relayApi.tx.hrmp.hrmpAcceptOpenChannel(2001).method.toHex();
      const resp = await relayEncoder.encodeHrmpAcceptOpenChannel(2001);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C300", "should have matching indices for HRMP.CloseChannel", async function () {
      if (rtVersion < 2100) {
        debug(`Runtime version is ${rtVersion}, which is less than 2100. Skipping test. `);
        this.skip();
      }
      const callHex = context.relayApi.tx.hrmp
        .hrmpCloseChannel({ sender: 2000, recipient: 2001 })
        .method.toHex();
      const resp = await relayEncoder.encodeHrmpCloseChannel(2000, 2001);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C400", "should have matching indices for Staking.Bond", async function () {
      const callHex = context.relayApi.tx.staking
        .bond(ALITH_SESSION_ADDRESS, 10000000000, "Staked")
        .method.toHex();
      const resp = await relayEncoder.encodeBond(
        ALITH_SESSION_ADDRESS,
        10000000000,
        hexToU8a("0x00")
      );
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C500", "should have matching indices for Staking.BondExtra", async function () {
      const callHex = context.relayApi.tx.staking.bondExtra(10000000000).method.toHex();
      const resp = await relayEncoder.encodeBondExtra(10000000000);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C600", "should have matching indices for Staking.Chill", async function () {
      const callHex = context.relayApi.tx.staking.chill().method.toHex();
      const resp = await relayEncoder.encodeChill();
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C700", "should have matching indices for Staking.Nominate", async function () {
      const callHex = context.relayApi.tx.staking.nominate([ALITH_SESSION_ADDRESS]).method.toHex();
      const resp = await relayEncoder.encodeNominate([ALITH_SESSION_ADDRESS]);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C800", "should have matching indices for Staking.Rebond", async function () {
      const callHex = context.relayApi.tx.staking.rebond(1000).method.toHex();
      const resp = await relayEncoder.encodeRebond(1000);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C900", "should have matching indices for Staking.SetController", async function () {
      const callHex = context.relayApi.tx.staking
        .setController(ALITH_SESSION_ADDRESS)
        .method.toHex();
      const resp = await relayEncoder.encodeSetController(ALITH_SESSION_ADDRESS);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C1000", "should have matching indices for Staking.SetPayee", async function () {
      const callHex = context.relayApi.tx.staking.setPayee("Staked").method.toHex();
      const resp = await relayEncoder.encodeSetPayee(hexToU8a("0x00"));
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C1100", "should have matching indices for Staking.Unbond", async function () {
      const callHex = context.relayApi.tx.staking.unbond(1000).method.toHex();
      const resp = await relayEncoder.encodeUnbond(1000);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C1200", "should have matching indices for Staking.Validate", async function () {
      const callHex = context.relayApi.tx.staking
        .validate({
          commission: 0,
          blocked: false,
        })
        .method.toHex();
      const resp = await relayEncoder.encodeValidate(0, false);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt("C1300", "should have matching indices for Staking.WithdrawUnbonded", async function () {
      const callHex = context.relayApi.tx.staking.withdrawUnbonded(10).method.toHex();
      const resp = await relayEncoder.encodeWithdrawUnbonded(10);
      expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
    });

    testIt(
      "C1400",
      "should have matching indices for Utility.asDerivative in V1",
      async function () {
        if (rtVersion < 2100) {
          debug(`Runtime version is ${rtVersion}, which is less than 2100. Skipping test. `);
          this.skip();
        }
        const inputCall = context.relayApi.tx.balances.transfer(ALITH_SESSION_ADDRESS, 1000);
        const callHex = context.relayApi.tx.utility.asDerivative(0, inputCall).method.toHex();
        const resp = await xcmTransactorV1.encodeUtilityAsDerivative(
          0,
          0,
          inputCall.method.toU8a()
        );
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      }
    );

    testIt(
      "C1500",
      "should have matching indices for Utility.asDerivative in V2",
      async function () {
        if (rtVersion < 2100) {
          debug(`Runtime version is ${rtVersion}, which is less than 2100. Skipping test. `);
          this.skip();
        }

        const chainType = context.polkadotApi.consts.system.version.specName.toString();
        if (chainType !== "moonbase") {
          debug(`Chain type ${chainType} does not support V2, skipping.`);
          this.skip();
        }

        const inputCall = context.relayApi.tx.balances.transfer(ALITH_SESSION_ADDRESS, 1000);
        const callHex = context.relayApi.tx.utility.asDerivative(0, inputCall).method.toHex();
        const resp = await xcmTransactorV2.encodeUtilityAsDerivative(
          0,
          0,
          inputCall.method.toU8a()
        );
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      }
    );
  }
);
