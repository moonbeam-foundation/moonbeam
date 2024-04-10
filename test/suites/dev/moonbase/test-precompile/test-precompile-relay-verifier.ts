import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { PRECOMPILE_RELAY_DATA_VERIFIER_ADDRESS } from "../../../../helpers";

describeSuite({
  id: "D012872",
  title: "Precompiles - relay-verifier",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const proof = {
      at: "0x1272470f226fc0e955838262e8dd17a7d7bad6563739cc53a3b1744ddf0ea872",
      proof: [
        "0x5f07c875e4cff74148e4628f264b974c8040628949b6ef4600c40000000000000000",
        "0x5f0c9c1284130706f5aea0c8b3d4c54d891501445f02000060020000610200006202000063020000640200" +
          "0065020000660200006702000068020000690200006a0200006b0200006c0200006d0200006e0200006f02" +
          "0000",
        "0x80046480ae6dc31222118597e5aaf1a34a195c282822e9f147bd2113fc611c2d8ad0daaf80106e32398477" +
          "afa87370d207850b5c2fcc9edde886bbbff970f30e870cb016ae806eb4916897c8a0f14604da0b634e5102" +
          "e814e3b3564d52a6a987b3822f35845980ce9727320ca95ab80f36e3f416706757f73bdc4b6a844b541848" +
          "64cf6f4d3783",
        "0x8061008051c5d06fc458e469b187e464073a9b1a27b78bc92f79e7519439b85509aebe67807c2154d55dc4" +
          "efdf670330add5144d07ed6efa4bdc6ffae6f1dd5eaa2f429e3080af579d5ddc5c697d42bfc014076594e6" +
          "6c7b324cfd3017810c4e93e4f6f0ae9e",
        "0x80ffff80a544fa461df3dc9358b0f7f88095a7e37d2037ce25934f9c47956687a94c79d7803413c0780b32" +
          "567fe87b4b5c073c992f0f50118f44f68ee4cea51bc7d1bc125c8000c1699c8f59a00b69d7034f91cad97e" +
          "7637a93e3f54984a01ca08c8dc9f9ad080699e1d4c85f1e4e73590d69882f9188db0445e1f6414dd753d69" +
          "aa4a201ccdfb80e2c14ce9239d367bde39f9625cf2dae689dff77760a6478bb5dc7a28309d95ce809992be" +
          "e3f46c3be2e44aec660c4a3109d71548441dd8bd4f8dcdeda20c6105f88002c9c0b5dbb322abfe7edfbb91" +
          "67049d0824d19cab106c62233d7da53517f8ca80583d87fe18e8d9ed0f9601d98f7614a6f12bdcccbc9e62" +
          "db443b0753fe1320ab800ab44d0802168f45ff9cff687769b6d4664c8ca1bc94b086df19e000f805d33b80" +
          "1802363d7de5b2d26805f5c86c4ad99384fa61184024cf597e2d65614625050580c161755bb505e8bdb112" +
          "5229bad3bc41c2ede4dba0789c0c1fa2eac866bbc6d580f697d83a00387c4123875066a7c74c97b09db562" +
          "d99ce515032da7826564fc2d808ee71cb07ac490d2c01144fde0f85c784a9e45d1eb50e1fc7f71d414e268" +
          "94b78090b075ba89594ceb80523aea74a75d35d16810920b36378e23cb173b408f2749807a57bac6b45c61" +
          "8551ec2afc20378cb9fe2da367249c9fa1975e1c81bd0a641d80a0197196bf1ae5833408f7c6cb410ddaa9" +
          "d524bfb29f6805a365ca353c19e931",
        "0x9e261276cc9d1f8598ea4b6a74b15c2f360080888a8ef6d6b18947204b9d2a2caec570f31bcca8de3d62cb" +
          "304750bfe750e799802530be352ac1dcc99fe5693df3c6445cdf72b2e3ded3ccd8275851b24fdd8d53505f" +
          "0e7b9012096b41c4eb3aaf947f6ea42908010080fc6475d793cf00f4eefb53e649aa37823d402f10863ccd" +
          "12868397067ed24e16",
        "0x9ec365c3cf59d671eb72da0e7a4113c41002505f0e7b9012096b41c4eb3aaf947f6ea429080000685f0f1f" +
          "0515f462cdcf84e0f1d6045dfcbb20c0e413b88d010000",
        "0x9f09d139e01a5eb2256f222e5fc5dbe6b3581580495b645f9c559f6d1b4047d2b84cdd96247886647e03c1" +
          "2d153b00247e17bfd2505f0e7b9012096b41c4eb3aaf947f6ea429080000585f0254e9d55588784fa2a62b" +
          "726696e2b1107002000080595d98af3421f8e2e99d30442ea36735a8047c30975f58d69e9684cfadd26e69" +
          "805e53a3e74921c6bf8c0e1c24d25a60d10fcbb7fa789d6c2263c568ce01c0aee180298a8183623b166f4e" +
          "75de0160dc695e2620f96bb4cc5b34a9467ddb937b0b1c",
      ],
    };

    // Keys that are used in the proof
    const keys = [
      // Timestamp now
      "0xf0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb",
      // Balances total issuance
      "0xc2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80",
      // Treasury approvals
      "0x89d139e01a5eb2256f222e5fc5dbe6b33c9c1284130706f5aea0c8b3d4c54d89",
    ];

    beforeAll(async () => {
      const RELAY_STORAGE_ROOT_KEY = context
        .polkadotJs()
        .query.relayStorageRoots.relayStorageRoot.key(1000)
        .toString();
      await context.createBlock();
      await context.createBlock([
        context
          .polkadotJs()
          .tx.sudo.sudo(
            context
              .polkadotJs()
              .tx.system.setStorage([
                [
                  RELAY_STORAGE_ROOT_KEY,
                  "0x767caa877bcea0d34dd515a202b75efa41bffbc9f814ab59e2c1c96716d4c65d",
                ],
              ])
          ),
      ]);
    }),
      it({
        id: "T01",
        title: "should successfully verify the Timestamp value in the proof",
        test: async function () {
          const readProof = context.polkadotJs().createType("ReadProof", proof);

          expect(
            await context.readContract!({
              contractAddress: PRECOMPILE_RELAY_DATA_VERIFIER_ADDRESS,
              contractName: "RelayDataVerifier",
              functionName: "verifyEntry",
              args: [1000, readProof.toJSON(), keys[0]],
              gas: 100_000n,
            })
          ).toBe("0xc0e413b88d010000"); // 1_708_190_328_000 scale encoded
        },
      });
    it({
      id: "T02",
      title: "should successfully verify the values in the proof (order of values matters)",
      test: async function () {
        const readProof = context.polkadotJs().createType("ReadProof", proof);

        expect(
          await context.readContract!({
            contractAddress: PRECOMPILE_RELAY_DATA_VERIFIER_ADDRESS,
            contractName: "RelayDataVerifier",
            functionName: "verifyEntries",
            args: [1000, readProof.toJSON(), keys],
            gas: 100_000n,
          })
        ).toStrictEqual([
          "0xc0e413b88d010000",
          "0x628949b6ef4600c40000000000000000",
          "0x445f02000060020000610200006202000063020000640200006502000066020000670200006802000069" +
            "0200006a0200006b0200006c0200006d0200006e0200006f020000",
        ]);
      },
    });
    it({
      id: "T03",
      title: "should return the latest relay block number",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: PRECOMPILE_RELAY_DATA_VERIFIER_ADDRESS,
            contractName: "RelayDataVerifier",
            functionName: "latestRelayBlockNumber",
            args: [],
          })
        ).toBe(1002);
      },
    });
  },
});
