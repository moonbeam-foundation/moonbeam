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
  id: "D022880",
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
        // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
        // And we need relay tokens for issuing a transaction to be executed in the relay

        const { contractAddress } = await registerAndFundAsset(
          context, 
          {
            id: 42259045809535163221576417993425387648n,
            location: RELAY_SOURCE_LOCATION,
            metadata: relayAssetMetadata,
            relativePrice: 1n
          },
          100000000000000n,
          ALITH_ADDRESS,
          true
        )
        
        const beforeBalance = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "balanceOf",
          args: [ALITH_ADDRESS],
        })

        const beforeSupply = await context.readContract!({
          contractName: "ERC20Instance",
          contractAddress: contractAddress,
          functionName: "totalSupply",
        })

        expect(beforeSupply).to.equal(100000000000000n);
        expect(beforeBalance).to.equal(100000000000000n);
        

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

        // We have used 1000 units to pay for the fees in the relay, so balance and supply should
        // have changed
        const afterAssetBalance = await context
          .polkadotJs()
          .query.assets.account(assetId.toU8a(), ALITH_ADDRESS);

        const expectedBalance = 100000000000000n - 1000n - 1n;
        expect(afterAssetBalance.unwrap().balance.toBigInt()).to.equal(expectedBalance);

        const AfterAssetDetails = await context.polkadotJs().query.assets.asset(assetId.toU8a());

        expect(AfterAssetDetails.unwrap().supply.toBigInt()).to.equal(expectedBalance);

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
