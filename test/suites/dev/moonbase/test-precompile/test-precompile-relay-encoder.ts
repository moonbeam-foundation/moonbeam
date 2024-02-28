import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_SESSION_ADDRESS, BALTATHAR_SESSION_ADDRESS } from "@moonwall/util";

describeSuite({
  id: "D012973",
  title: "Precompiles - relay-encoder",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "allows to get encoding of bond stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeBond",
            args: [100, "0x02"],
          })
        ).toBe("0x0600910102");
      },
    });

    it({
      id: "T02",
      title: "allows to get encoding of bond_more stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeBondExtra",
            args: [100],
          })
        ).to.equal("0x06019101");
      },
    });

    it({
      id: "T03",
      title: "allows to get encoding of unbond stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeUnbond",
            args: [100],
          })
        ).to.equal("0x06029101");
      },
    });

    it({
      id: "T04",
      title: "allows to get encoding of chill stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeChill",
            args: [],
          })
        ).to.equal("0x0606");
      },
    });

    it({
      id: "T05",
      title: "allows to get encoding of withdraw_unbonded stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeWithdrawUnbonded",
            args: [100],
          })
        ).to.equal("0x060364000000");
      },
    });

    it({
      id: "T06",
      title: "allows to get encoding of validate stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeValidate",
            args: [100000000, false],
          })
        ).to.equal("0x06040284d71700");
      },
    });

    it({
      id: "T07",
      title: "allows to get encoding of nominate stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeNominate",
            args: [[ALITH_SESSION_ADDRESS, BALTATHAR_SESSION_ADDRESS]],
          })
        ).to.equal(
          "0x06050800d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7" +
            "a56da27d008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa" +
            "4794f26a48"
        );
      },
    });

    it({
      id: "T08",
      title: "allows to get encoding of set_payee stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeSetPayee",
            args: ["0x02"],
          })
        ).to.equal("0x060702");
      },
    });

    it({
      id: "T09",
      title: "allows to get encoding of set_controller stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeSetController",
            args: [],
          })
        ).to.equal("0x0608");
      },
    });

    it({
      id: "T10",
      title: "allows to get encoding of rebond stake call",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "RelayEncoder",
            functionName: "encodeRebond",
            args: [100],
          })
        ).to.equal("0x06139101");
      },
    });
  },
});
