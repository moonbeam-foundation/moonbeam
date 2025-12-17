import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { fromBytes } from "viem";
import {
  verifyLatestBlockFees,
  expectEVMResult,
  registerXcmTransactorAndContract,
  registerAndFundAsset,
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
} from "../../../../helpers";

const ADDRESS_RELAY_ASSETS = "0xffffffff1fcacbd218edc0eba20fc2308c778080";

describeSuite({
  id: "D022770",
  title: "Precompiles - xcm transactor",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to issue transfer xcm transactor with currency Id",
      test: async function () {
        const initialBalance = 100000000000000n;

        const { contractAddress } = await registerAndFundAsset(
          context,
          {
            id: 42259045809535163221576417993425387648n,
            location: RELAY_SOURCE_LOCATION,
            metadata: relayAssetMetadata,
            relativePrice: 1000000000000000000n,
          },
          initialBalance,
          ALITH_ADDRESS,
          true
        );

        const beforeBalance = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "balanceOf",
          args: [ALITH_ADDRESS],
        });

        const beforeSupply = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "totalSupply",
        });

        expect(beforeSupply).to.equal(initialBalance);
        expect(beforeBalance).to.equal(initialBalance);

        const transactor = 0;
        const index = 0;
        // Destination as currency Id address
        const asset = contractAddress;
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const weight = 1000;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmTransactorV1",
          functionName: "transactThroughDerivative",
          args: [transactor, index, asset, weight, transact_call],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const afterBalance = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "balanceOf",
          args: [ALITH_ADDRESS],
        });

        const afterSupply = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "totalSupply",
        });

        // Fee calculation via pallet-xcm-weight-trader:
        // - V1 precompile doesn't pass explicit feeAmount, so fee is computed from weight
        // - transact_extra_weight = 1 (from setTransactInfo)
        // - total_weight = weight + transact_extra_weight = 1001
        // - native_fee = WeightToFee(total_weight) = total_weight * WEIGHT_FEE = 1001 * 12_500
        // - With relative_price = 1e18: fee = native_fee * 1e18 / 1e18 = native_fee
        const WEIGHT_FEE = 12_500n;
        const transact_extra_weight = 1n;
        const totalWeight = BigInt(weight) + transact_extra_weight;
        const expectedFee = totalWeight * WEIGHT_FEE;
        const expectedBalance = initialBalance - expectedFee;

        expect(afterBalance).to.equal(expectedBalance);
        expect(afterSupply).to.equal(expectedBalance);

        // Fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
