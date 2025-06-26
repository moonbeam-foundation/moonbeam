import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { fromBytes } from "viem";
import {
  verifyLatestBlockFees,
  registerXcmTransactorAndContract,
  registerAndFundAsset,
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
} from "../../../../helpers";

describeSuite({
  id: "D022876",
  title: "Precompiles - xcm transactor",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to retrieve index through precompiles",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "XcmTransactorV1",
            functionName: "indexToAccount",
            args: [0],
          })
        ).toBe(ALITH_ADDRESS);
      },
    });

    it({
      id: "T02",
      title: "allows to retrieve transactor info through precompiles old interface",
      test: async function () {
        // Destination as multilocation, one parent
        const asset: [number, any[]] = [1, []];

        expect(
          await context.readPrecompile!({
            precompileName: "XcmTransactorV1",
            functionName: "transactInfo",
            args: [asset],
          })
        ).toEqual([1n, 1000000000000n, 20000000000n]);
      },
    });

    it({
      id: "T03",
      title: "allows to retrieve fee per second through precompiles",
      test: async function () {
        const asset: [number, any[]] = [1, []];

        expect(
          await context.readPrecompile!({
            precompileName: "XcmTransactorV1",
            functionName: "feePerSecond",
            args: [asset],
          })
        ).toBe(1000000000000n);
      },
    });

    it({
      id: "T04",
      title: "allows to retrieve transactor info through precompiles",
      test: async function () {
        const asset: [number, any[]] = [1, []];

        expect(
          await context.readPrecompile!({
            precompileName: "XcmTransactorV1",
            functionName: "transactInfoWithSigned",
            args: [asset],
          })
        ).toStrictEqual([1n, 1n, 20000000000n]);
      },
    });

    it({
      id: "T05",
      title: "allows to issue transfer xcm transactor",
      test: async function () {
        const { contractAddress } = await registerAndFundAsset(
          context,
          {
            id: 42259045809535163221576417993425387648n,
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

        expect(beforeSupply).to.equal(100000000000000n);
        expect(beforeBalance).to.equal(100000000000000n);

        const transactor = 0;
        const index = 0;
        const asset = [1, []];
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const weight = 1000;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmTransactorV1",
          functionName: "transactThroughDerivativeMultilocation",
          args: [transactor, index, asset, weight, transact_call],
          rawTxOnly: true,
        });

        await context.createBlock(rawTxn);

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
        const expectedBalance = 100000000000000n - 1000n - 1n;
        expect(afterBalance).to.equal(expectedBalance);
        expect(afterSupply).to.equal(expectedBalance);

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
