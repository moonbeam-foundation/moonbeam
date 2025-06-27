import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, ALITH_PRIVATE_KEY } from "@moonwall/util";
import { fromBytes } from "viem";
import {
  verifyLatestBlockFees,
  expectEVMResult,
  registerXcmTransactorDerivativeIndex,
  registerAndFundAsset,
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
  PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS,
  PARA_1000_SOURCE_LOCATION,
} from "../../../../helpers";

describeSuite({
  id: "D022879",
  title: "Precompiles - xcm transactor V3",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let xcmTransactorCaller;
    const ALITH_TRANSACTOR_INDEX = 0;
    const SM_TRANSACTOR_INDEX = 1;

    beforeAll(async () => {
      xcmTransactorCaller = await context.deployContract!("XcmTransactorCaller");
      console.log("XcmTransactorCaller deployed at", xcmTransactorCaller.contractAddress);

      // Register account at index 0
      await registerXcmTransactorDerivativeIndex(context);
      expect(
        await context.readContract!({
          contractAddress: PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS,
          contractName: "XcmTransactorV3",
          functionName: "indexToAccount",
          args: [ALITH_TRANSACTOR_INDEX],
        })
      ).toBe(ALITH_ADDRESS);

      // Register account at index 1
      await context.createBlock(
        context
          .polkadotJs()
          .tx.sudo.sudo(
            context
              .polkadotJs()
              .tx.xcmTransactor.register(xcmTransactorCaller.contractAddress, SM_TRANSACTOR_INDEX)
          )
      );
      expect(
        (
          await context.readContract!({
            contractAddress: PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS,
            contractName: "XcmTransactorV3",
            functionName: "indexToAccount",
            args: [SM_TRANSACTOR_INDEX],
          })
        ).toLowerCase()
      ).toBe(xcmTransactorCaller.contractAddress.toLowerCase());
    });

    it({
      id: "T01",
      title: "allows to issue transfer xcm transactor with currency Id - weights v2 - refund",
      test: async function () {
        const { contractAddress } = await registerAndFundAsset(
          context,
          {
            id: 1n,
            location: RELAY_SOURCE_LOCATION,
            metadata: relayAssetMetadata,
            relativePrice: 1n,
          },
          100000000000000n,
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

        expect(beforeBalance).to.equal(100000000000000n);
        expect(beforeSupply).to.equal(100000000000000n);

        const transactor = 0;
        const asset = contractAddress;
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const transactWeight = { refTime: 1000, proofSize: 1000 };
        const overallWeight = { refTime: 2000, proofSize: 2000 };
        const feeAmount = 1000;
        const refund = true;

        const rawTx = await context.writeContract!({
          contractAddress: PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS,
          contractName: "XcmTransactorV3",
          functionName: "transactThroughDerivative",
          args: [
            transactor,
            ALITH_TRANSACTOR_INDEX,
            asset,
            transactWeight,
            transact_call,
            feeAmount,
            overallWeight,
            refund,
          ],
          gas: 500_000n,
          rawTxOnly: true,
          privateKey: ALITH_PRIVATE_KEY,
        });

        const result = await context.createBlock(rawTx);
        expectEVMResult(result.result!.events, "Succeed");

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

        // We have used 1000 units to pay for the fees in the relay  (plus 1 transact_extra_weight),
        // so balance and supply should have changed
        const expectedBalance = 100000000000000n - 1000n;
        expect(afterBalance).to.equal(expectedBalance);
        expect(afterSupply).to.equal(expectedBalance);

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });

    it({
      id: "T02",
      title: "allows to issue transfer xcm transactor with currency Id - weights v2 - refund",
      test: async function () {
        const { contractAddress } = await registerAndFundAsset(
          context,
          {
            id: 2n,
            location: PARA_1000_SOURCE_LOCATION,
            metadata: {
              name: "PARA1000",
              symbol: "PARA1000",
              decimals: 12n,
              isFrozen: false,
            },
            relativePrice: 1n,
          },
          100000000000000n,
          xcmTransactorCaller.contractAddress,
          true
        );

        const beforeBalance = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "balanceOf",
          args: [xcmTransactorCaller.contractAddress],
        });

        const beforeSupply = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "totalSupply",
        });

        expect(beforeBalance).to.equal(100000000000000n);
        expect(beforeSupply).to.equal(100000000000000n);

        const transactor = 0;
        const asset = contractAddress;
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const transactWeight = { refTime: 1000, proofSize: 1000 };
        const overallWeight = { refTime: 2000, proofSize: 2000 };
        const feeAmount = 1000;
        const refund = true;

        const rawTx = await context.writeContract!({
          contractAddress: xcmTransactorCaller.contractAddress,
          contractName: "XcmTransactorCaller",
          functionName: "transactThroughDerivativeV3",
          args: [
            transactor,
            SM_TRANSACTOR_INDEX,
            asset,
            transactWeight,
            transact_call,
            feeAmount,
            overallWeight,
            refund,
          ],
          gas: 500_000n,
          rawTxOnly: true,
          privateKey: ALITH_PRIVATE_KEY,
        });

        const result = await context.createBlock(rawTx);
        expectEVMResult(result.result!.events, "Succeed");

        const afterBalance = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "balanceOf",
          args: [xcmTransactorCaller.contractAddress],
        });

        const afterSupply = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "totalSupply",
        });

        // We have used 1000 units to pay for the fees in the relay  (plus 1 transact_extra_weight),
        // so balance and supply should have changed
        const expectedBalance = 100000000000000n - 1000n;
        expect(afterBalance).to.equal(expectedBalance);
        expect(afterSupply).to.equal(expectedBalance);

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
