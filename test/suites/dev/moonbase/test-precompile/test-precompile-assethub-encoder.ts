import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_SESSION_ADDRESS, BALTATHAR_SESSION_ADDRESS } from "@moonwall/util";

// TODO: Migrate from readContract to readPrecompile once Moonwall supports AssetHubEncoder precompile

describeSuite({
  id: "D022890",
  title: "Precompiles - assethub-encoder",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const ASSETHUB_ENCODER_ADDRESS = "0x000000000000000000000000000000000000081B";

    it({
      id: "T01",
      title: "allows to get encoding of bond stake call",
      test: async function () {
        // AssetHub Westend uses pallet index 80 for staking (not 6 like relay)
        // Expected: 0x50 (80) + 0x00 (bond call index) + encoded args
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeBond",
          args: [100, "0x02"],
        });
        expect(result).toBe("0x5000910102");
      },
    });

    it({
      id: "T02",
      title: "allows to get encoding of bond_extra stake call",
      test: async function () {
        // AssetHub pallet 80, call 1 (bond_extra)
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeBondExtra",
          args: [100],
        });
        expect(result).to.equal("0x50019101");
      },
    });

    it({
      id: "T03",
      title: "allows to get encoding of unbond stake call",
      test: async function () {
        // AssetHub pallet 80, call 2 (unbond)
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeUnbond",
          args: [100],
        });
        expect(result).to.equal("0x50029101");
      },
    });

    it({
      id: "T04",
      title: "allows to get encoding of chill stake call",
      test: async function () {
        // AssetHub pallet 80, call 6 (chill)
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeChill",
          args: [],
        });
        expect(result).to.equal("0x5006");
      },
    });

    it({
      id: "T05",
      title: "allows to get encoding of withdraw_unbonded stake call",
      test: async function () {
        // AssetHub pallet 80, call 3 (withdraw_unbonded)
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeWithdrawUnbonded",
          args: [100],
        });
        expect(result).to.equal("0x500364000000");
      },
    });

    it({
      id: "T06",
      title: "allows to get encoding of validate stake call",
      test: async function () {
        // AssetHub pallet 80, call 4 (validate)
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeValidate",
          args: [100000000, false],
        });
        expect(result).to.equal("0x50040284d71700");
      },
    });

    it({
      id: "T07",
      title: "allows to get encoding of nominate stake call",
      test: async function () {
        // AssetHub pallet 80, call 5 (nominate)
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeNominate",
          args: [[ALITH_SESSION_ADDRESS, BALTATHAR_SESSION_ADDRESS]],
        });
        expect(result).to.equal(
          "0x50050800d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7" +
            "a56da27d008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa" +
            "4794f26a48"
        );
      },
    });

    it({
      id: "T08",
      title: "allows to get encoding of set_payee stake call",
      test: async function () {
        // AssetHub pallet 80, call 7 (set_payee)
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeSetPayee",
          args: ["0x02"],
        });
        expect(result).to.equal("0x500702");
      },
    });

    it({
      id: "T09",
      title: "allows to get encoding of set_controller stake call",
      test: async function () {
        // AssetHub pallet 80, call 8 (set_controller)
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeSetController",
          args: [],
        });
        expect(result).to.equal("0x5008");
      },
    });

    it({
      id: "T10",
      title: "allows to get encoding of rebond stake call",
      test: async function () {
        // AssetHub pallet 80, call 19 (rebond)
        const result = await context.readContract!({
          contractAddress: ASSETHUB_ENCODER_ADDRESS,
          contractName: "AssetHubEncoderInstance",
          functionName: "encodeRebond",
          args: [100],
        });
        expect(result).to.equal("0x50139101");
      },
    });
  },
});
