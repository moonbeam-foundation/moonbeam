import "@moonbeam-network/api-augment";
import { alith, describeSuite, expect } from "moonwall";
import type { SignerOptions } from "@polkadot/api/types";
import { merkleizeMetadata } from "@polkadot-api/merkleize-metadata";
import { u8aToHex } from "@polkadot/util";
import type { ApiPromise } from "@polkadot/api";

async function metadataHash(api: ApiPromise) {
  const m = await api.call.metadata.metadataAtVersion(15);
  const { specName, specVersion } = api.runtimeVersion;
  const merkleizedMetadata = merkleizeMetadata(m.toHex(), {
    base58Prefix: api.consts.system.ss58Prefix.toNumber(),
    decimals: api.registry.chainDecimals[0],
    specName: specName.toString(),
    specVersion: specVersion.toNumber(),
    tokenSymbol: api.registry.chainTokens[0],
  });

  return u8aToHex(merkleizedMetadata.digest());
}

describeSuite({
  id: "D010601",
  title: "Test transaction with metadata hash",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "Should fail with an invalid metadata hash",
      test: async function () {
        const withMetadataOpts: Partial<SignerOptions> = {
          mode: 1,
          metadataHash: "0x" + "00".repeat(32),
        };

        let errorMsg = "";
        try {
          await context.polkadotJs().tx.system.remark("0x00").signAndSend(alith, withMetadataOpts);
        } catch (e: any) {
          errorMsg = e.message;
        }

        expect(errorMsg).to.be.eq("1010: Invalid Transaction: Transaction has a bad signature");
      },
    });

    it({
      id: "T02",
      title: "Should succeed with a valid metadata hash",
      test: async function () {
        const withMetadataOpts = {
          mode: 1,
          metadataHash: await metadataHash(context.polkadotJs()),
        };

        await context.polkadotJs().tx.system.remark("0x00").signAndSend(alith, withMetadataOpts);
      },
    });
  },
});
