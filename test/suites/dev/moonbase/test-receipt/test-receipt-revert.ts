import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { getAddress } from "viem";

describeSuite({
  id: "D013201",
  title: "Receipt - Revert",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should generate a receipt",
      test: async function () {
        const { hash } = await context.deployContract!("FailingConstructor", { gas: 300000n });
        const receipt = await context.viem().getTransactionReceipt({ hash });

        expect(receipt.status).toBe("reverted");
        expect(receipt.blockNumber).toBe(1n);
        expect(getAddress(receipt.contractAddress!)).toBe(
          getAddress("0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3")
        );
        expect(receipt.cumulativeGasUsed).toBe(54605n);
        expect(getAddress(receipt.from!)).toBe(
          getAddress("0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac")
        );
        expect(receipt.gasUsed).toBe(54605n);
        expect(receipt.logs).toStrictEqual([]);
        expect(receipt.transactionHash).toBe(hash);
        expect(receipt.to).toBe(null);
        expect(receipt.transactionIndex).toBe(0);
      },
    });
  },
});
