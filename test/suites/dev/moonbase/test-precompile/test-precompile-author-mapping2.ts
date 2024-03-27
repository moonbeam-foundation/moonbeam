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
  id: "D012919",
  title: "Precompiles - author mapping",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let firstMappingAccount: KeyringPair;
    let secondMappingAccount: KeyringPair;

    beforeAll(async () => {
      firstMappingAccount = generateKeyringPair("sr25519");
      secondMappingAccount = generateKeyringPair("sr25519");
      // Add association
      await context.createBlock(
        context.polkadotJs().tx.authorMapping.addAssociation(firstMappingAccount.publicKey)
      );

      // Verify association was added
      const mapping = await context
        .polkadotJs()
        .query.authorMapping.mappingWithDeposit(u8aToHex(firstMappingAccount.publicKey));
      expect(mapping.unwrap().account.toString()).to.eq(ALITH_ADDRESS);
      expect(mapping.unwrap().deposit.toBigInt()).to.eq(DEFAULT_GENESIS_MAPPING);
    });

    it({
      id: "T01",
      title: "allows to update association",
      test: async function () {
        const { abi } = fetchCompiledContract("AuthorMapping");
        const tx = await createViemTransaction(context, {
          to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
          data: encodeFunctionData({
            abi,
            functionName: "updateAssociation",
            args: [
              u8aToHex(firstMappingAccount.publicKey),
              u8aToHex(secondMappingAccount.publicKey),
            ],
          }),
        });

        const { result } = await context.createBlock(tx);

        const receipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: result?.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        // Verify we updated firstMappingAccount for secondMappingAccount
        const mapping = await context
          .polkadotJs()
          .query.authorMapping.mappingWithDeposit(u8aToHex(secondMappingAccount.publicKey));
        expect(mapping.unwrap().account.toString()).to.eq(ALITH_ADDRESS);
        expect(mapping.unwrap().deposit.toBigInt()).to.eq(DEFAULT_GENESIS_MAPPING);
        expect(
          (
            await context
              .polkadotJs()
              .query.authorMapping.mappingWithDeposit(u8aToHex(firstMappingAccount.publicKey))
          ).isNone
        ).to.be.true;
      },
    });
  },
});
