import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import { alith } from "@moonwall/util";
import { PRECOMPILE_ZKAUTH_VERIFIER } from "../../../../helpers";

describeSuite({
  id: "D012893",
  title: "Precompiles - ZkAuth Verifier",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "Proof verification",
      test: async function () {
        const receipt = u8aToHex(
          new Uint8Array([
            2, 128, 2, 43, 1, 50, 120, 33, 198, 254, 149, 68, 226, 225, 17, 140, 204, 69, 20, 109,
            51, 6, 168, 103, 117, 110, 250, 109, 184, 32, 196, 171, 167, 28, 85, 5, 244, 90, 120,
            165, 223, 25, 154, 108, 104, 122, 13, 232, 213, 14, 86, 19, 132, 209, 3, 193, 25, 213,
            245, 201, 14, 171, 188, 20, 61, 87, 143, 13, 130, 207, 30, 66, 49, 130, 239, 36, 170,
            56, 67, 88, 46, 28, 54, 66, 86, 165, 95, 68, 198, 216, 117, 26, 60, 223, 118, 135, 136,
            12, 136, 32, 23, 65, 122, 191, 152, 22, 242, 244, 200, 125, 118, 171, 39, 237, 221, 111,
            54, 206, 237, 239, 43, 53, 43, 18, 34, 183, 18, 246, 121, 66, 82, 32, 197, 167, 19, 129,
            173, 59, 16, 34, 242, 30, 235, 7, 146, 132, 118, 87, 148, 253, 81, 23, 208, 7, 168, 105,
            210, 52, 109, 145, 151, 150, 81, 6, 121, 61, 178, 93, 11, 164, 87, 126, 219, 168, 114,
            138, 195, 52, 220, 254, 122, 146, 136, 48, 3, 212, 72, 75, 128, 147, 141, 67, 218, 154,
            60, 46, 220, 205, 203, 69, 96, 242, 206, 27, 86, 2, 223, 100, 121, 144, 202, 185, 119,
            43, 89, 171, 139, 197, 216, 117, 193, 198, 116, 232, 116, 247, 177, 18, 174, 114, 105,
            25, 177, 73, 197, 193, 153, 220, 185, 220, 55, 126, 49, 54, 69, 48, 178, 207, 58, 130,
            2, 134, 78, 60, 202, 24, 79, 241, 245, 0, 0, 128, 236, 128, 1, 170, 250, 213, 155, 4,
            174, 236, 161, 132, 15, 140, 173, 206, 233, 7, 239, 144, 137, 146, 13, 236, 243, 157,
            148, 11, 129, 196, 248, 250, 14, 176, 134, 243, 215, 2, 238, 254, 237, 243, 9, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 32, 0, 228, 11, 84, 2, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 177,
            158, 148, 199, 9, 232, 199, 167, 255, 2, 168, 139, 240, 189, 2, 173, 254, 225, 194, 7,
            152, 247, 174, 196, 6, 196, 171, 223, 158, 9, 141, 241, 146, 220, 10, 170, 237, 133,
            231, 5, 32, 0, 228, 11, 84, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0,
            2, 0, 0, 0, 3, 0, 0, 0, 177, 158, 148, 199, 9, 232, 199, 167, 255, 2, 168, 139, 240,
            189, 2, 173, 254, 225, 194, 7, 152, 247, 174, 196, 6, 196, 171, 223, 158, 9, 141, 241,
            146, 220, 10, 170, 237, 133, 231, 5,
          ])
        );
        const imageId = [
          715585636, 3586935525, 3274293606, 2872050810, 564159597, 2621011314, 3667725176,
          1510137221,
        ];
        // Set ImageId
        const imageIdStorageKey = u8aToHex(
          u8aConcat(xxhashAsU8a("zkAuth", 128), xxhashAsU8a("ImageId", 128))
        );
        const imageIdStorageValue = u8aToHex(new Uint8Array(new Uint32Array(imageId).buffer));
        const tx = context
          .polkadotJs()
          .tx.system.setStorage([[imageIdStorageKey, imageIdStorageValue]]);
        console.log([imageIdStorageKey, imageIdStorageValue]);
        await context.polkadotJs().tx.sudo.sudo(tx).signAndSend(alith);
        await context.createBlock();

        expect(
          await context.readContract!({
            contractAddress: PRECOMPILE_ZKAUTH_VERIFIER,
            contractName: "ZkAuth",
            functionName: "verifyProof",
            args: [receipt],
            gas: "estimate",
          })
        ).toBe("0x010203");
      },
    });
  },
});
