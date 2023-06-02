import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { Contract, ethers, WebSocketProvider } from "ethers";
import { ALITH_SESSION_ADDRESS } from "@moonwall/util";
import { hexToU8a } from "@polkadot/util";
import { ApiPromise } from "@polkadot/api";
import { getCompiled } from "../../helpers/contracts.js";

describeSuite({
  id: "S1750",
  title: "Relay chain Module:Method indices should match our encoding",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let relayEncoder: Contract;
    let xcmTransactorV1: Contract;
    let xcmTransactorV2: Contract;
    let relayApi: ApiPromise;

    beforeAll(async function () {
      relayApi = context.polkadotJs({ apiName: "relay" });
      const RELAY_ENCODER_PRECOMPILE = "0x0000000000000000000000000000000000000805";
      const XCM_TRANSACTOR_V1_PRECOMPILE = "0x0000000000000000000000000000000000000806";
      const XCM_TRANSACTOR_V2_PRECOMPILE = "0x000000000000000000000000000000000000080D";

      const RELAY_ENCODER_CONTRACT_JSON = getCompiled(
        "compiled/precompiles/relay-encoder/RelayEncoder"
      );
      const RELAY_ENCODER_INTERFACE = RELAY_ENCODER_CONTRACT_JSON.contract.abi;

      const XCM_TRANSACTOR_V1_JSON = getCompiled(
        "compiled/precompiles/xcm-transactor/src/v1/XcmTransactorV1"
      );
      const XCM_TRANSACTOR_V1_INTERFACE = XCM_TRANSACTOR_V1_JSON.contract.abi;

      const XCM_TRANSACTOR_V2_JSON = getCompiled(
        "compiled/precompiles/xcm-transactor/src/v2/XcmTransactorV2"
      );
      const XCM_TRANSACTOR_V2_INTERFACE = XCM_TRANSACTOR_V2_JSON.contract.abi;

      relayEncoder = new ethers.Contract(
        RELAY_ENCODER_PRECOMPILE,
        RELAY_ENCODER_INTERFACE,
        context.ethersSigner().provider as WebSocketProvider
      );

      xcmTransactorV1 = new ethers.Contract(
        XCM_TRANSACTOR_V1_PRECOMPILE,
        XCM_TRANSACTOR_V1_INTERFACE,
        context.ethersSigner().provider as WebSocketProvider
      );

      xcmTransactorV2 = new ethers.Contract(
        XCM_TRANSACTOR_V2_PRECOMPILE,
        XCM_TRANSACTOR_V2_INTERFACE,
        context.ethersSigner().provider as WebSocketProvider
      );
    });

    it({
      id: "C100",
      title: "should have matching indices for HRMP.InitOpenChannel",
      minRtVersion: 2100,
      test: async function () {
        const callHex = relayApi.tx.hrmp.hrmpInitOpenChannel(2000, 1000, 102400).method.toHex();
        const resp = await relayEncoder.encodeHrmpInitOpenChannel(2000, 1000, 102400);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C200",
      title: "should have matching indices for HRMP.AcceptOpenChannel",
      minRtVersion: 2100,
      test: async function () {
        const callHex = relayApi.tx.hrmp.hrmpAcceptOpenChannel(2001).method.toHex();
        const resp = await relayEncoder.encodeHrmpAcceptOpenChannel(2001);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C300",
      title: "should have matching indices for HRMP.CloseChannel",
      minRtVersion: 2100,
      test: async function () {
        const callHex = relayApi.tx.hrmp
          .hrmpCloseChannel({ sender: 2000, recipient: 2001 })
          .method.toHex();
        const resp = await relayEncoder.encodeHrmpCloseChannel(2000, 2001);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C400",
      title: "should have matching indices for Staking.Bond",
      test: async function () {
        const callHex = relayApi.tx.staking
          .bond(ALITH_SESSION_ADDRESS, 10000000000, "Staked")
          .method.toHex();
        const resp = await relayEncoder.encodeBond(
          ALITH_SESSION_ADDRESS,
          10000000000,
          hexToU8a("0x00")
        );
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C500",
      title: "should have matching indices for Staking.BondExtra",
      test: async function () {
        const callHex = relayApi.tx.staking.bondExtra(10000000000).method.toHex();
        const resp = await relayEncoder.encodeBondExtra(10000000000);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C600",
      title: "should have matching indices for Staking.Chill",
      test: async function () {
        const callHex = relayApi.tx.staking.chill().method.toHex();
        const resp = await relayEncoder.encodeChill();
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C700",
      title: "should have matching indices for Staking.Nominate",
      test: async function () {
        const callHex = relayApi.tx.staking.nominate([ALITH_SESSION_ADDRESS]).method.toHex();
        const resp = await relayEncoder.encodeNominate([ALITH_SESSION_ADDRESS]);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C800",
      title: "should have matching indices for Staking.Rebond",
      test: async function () {
        const callHex = relayApi.tx.staking.rebond(1000).method.toHex();
        const resp = await relayEncoder.encodeRebond(1000);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C900",
      title: "should have matching indices for Staking.SetController",
      test: async function () {
        const callHex = relayApi.tx.staking.setController(ALITH_SESSION_ADDRESS).method.toHex();
        const resp = await relayEncoder.encodeSetController(ALITH_SESSION_ADDRESS);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C1000",
      title: "should have matching indices for Staking.SetPayee",
      test: async function () {
        const callHex = relayApi.tx.staking.setPayee("Staked").method.toHex();
        const resp = await relayEncoder.encodeSetPayee(hexToU8a("0x00"));
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C1100",
      title: "should have matching indices for Staking.Unbond",
      test: async function () {
        const callHex = relayApi.tx.staking.unbond(1000).method.toHex();
        const resp = await relayEncoder.encodeUnbond(1000);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C1200",
      title: "should have matching indices for Staking.Validate",
      test: async function () {
        const callHex = relayApi.tx.staking
          .validate({
            commission: 0,
            blocked: false,
          })
          .method.toHex();
        const resp = await relayEncoder.encodeValidate(0, false);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C1300",
      title: "should have matching indices for Staking.WithdrawUnbonded",
      test: async function () {
        const callHex = relayApi.tx.staking.withdrawUnbonded(10).method.toHex();
        const resp = await relayEncoder.encodeWithdrawUnbonded(10);
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C1400",
      title: "should have matching indices for Utility.asDerivative in V1",
      minRtVersion: 2100,
      test: async function () {
        const inputCall = relayApi.tx.balances.transfer(ALITH_SESSION_ADDRESS, 1000);
        const callHex = relayApi.tx.utility.asDerivative(0, inputCall).method.toHex();
        const resp = await xcmTransactorV1.encodeUtilityAsDerivative(
          0,
          0,
          inputCall.method.toU8a()
        );
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });

    it({
      id: "C1500",
      title: "should have matching indices for Utility.asDerivative in V2",
      minRtVersion: 2100,
      test: async function () {
        const chainType = context
          .polkadotJs({ apiName: "para" })
          .consts.system.version.specName.toString();
        if (chainType !== "moonbase") {
          log(`Chain type ${chainType} does not support V2, skipping.`);
          return; // TODO: replace with skip() when added to vitest;
        }

        const inputCall = relayApi.tx.balances.transfer(ALITH_SESSION_ADDRESS, 1000);
        const callHex = relayApi.tx.utility.asDerivative(0, inputCall).method.toHex();
        const resp = await xcmTransactorV2.encodeUtilityAsDerivative(
          0,
          0,
          inputCall.method.toU8a()
        );
        expect(resp, "Mismatched encoding between relaychain and local values").to.equals(callHex);
      },
    });
  },
});
