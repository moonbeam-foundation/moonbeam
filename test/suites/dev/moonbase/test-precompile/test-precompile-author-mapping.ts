import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  DEFAULT_GENESIS_MAPPING,
  PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
  createViemTransaction,
  generateKeyringPair,
} from "@moonwall/util";
import { u8aToHex } from "@polkadot/util";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D012918",
  title: "Precompiles - author mapping",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "allows to add association",
      test: async function () {
        const mappingAccount = generateKeyringPair("sr25519");
        const { abi } = fetchCompiledContract("AuthorMapping");

        const { result } = await context.createBlock(
          createViemTransaction(context, {
            to: PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
            data: encodeFunctionData({
              abi,
              functionName: "addAssociation",
              args: [u8aToHex(mappingAccount.publicKey)],
            }),
          })
        );

        const receipt = await context
          .viem("public")
          .getTransactionReceipt({ hash: result?.hash as `0x${string}` });
        expect(receipt.status).to.equal("success");

        const mapping = await context
          .polkadotJs()
          .query.authorMapping.mappingWithDeposit(u8aToHex(mappingAccount.publicKey));
        expect(mapping.unwrap().account.toString()).to.eq(ALITH_ADDRESS);
        expect(mapping.unwrap().deposit.toBigInt()).to.eq(DEFAULT_GENESIS_MAPPING);
      },
    });
  },
});
