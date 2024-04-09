import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  DEFAULT_GENESIS_MAPPING,
  KeyringPair,
  PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
  createViemTransaction,
  generateKeyringPair,
} from "@moonwall/util";
import { u8aToHex } from "@polkadot/util";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D012820",
  title: "Precompiles - author mapping",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let mappingAccount: KeyringPair;

    beforeAll(async () => {
      mappingAccount = generateKeyringPair("sr25519");
      // Add association

      await context.createBlock(
        context.polkadotJs().tx.authorMapping.addAssociation(mappingAccount.publicKey)
      );

      // Verify association was added
      const mapping = await context
        .polkadotJs()
        .query.authorMapping.mappingWithDeposit(u8aToHex(mappingAccount.publicKey));
      expect(mapping.unwrap().account.toString()).to.eq(ALITH_ADDRESS);
      expect(mapping.unwrap().deposit.toBigInt()).to.eq(DEFAULT_GENESIS_MAPPING);
    });

    it({
      id: "T01",
      title: "allows to update association",
      test: async function () {
        const { abi } = fetchCompiledContract("AuthorMapping");
        const { result } = await context.createBlock(
          createViemTransaction(context, {
            to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
            data: encodeFunctionData({
              abi,
              functionName: "clearAssociation",
              args: [u8aToHex(mappingAccount.publicKey)],
            }),
          })
        );

        const receipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: result?.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        // Verify we removed the association
        expect(
          (
            await context
              .polkadotJs()
              .query.authorMapping.mappingWithDeposit(u8aToHex(mappingAccount.publicKey))
          ).isNone
        ).to.be.true;
      },
    });
  },
});
