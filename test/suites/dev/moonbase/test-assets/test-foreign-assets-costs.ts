import "@moonbeam-network/api-augment";
import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect, beforeAll, fetchCompiledContract } from "@moonwall/cli";
import {
  ARBITRARY_ASSET_ID,
  PARA_1000_SOURCE_LOCATION_V4,
  RELAY_SOURCE_LOCATION_V4,
  foreignAssetBalance,
  mockAssetBalance,
  patchLocationV4recursively,
  registerForeignAsset,
  relayAssetMetadata,
} from "../../../../helpers";
import { parseAbi } from "viem";
import exp from "constants";
import { alith, ALITH_ADDRESS, BALTATHAR_ADDRESS } from "@moonwall/util";

describeSuite({
  id: "D010111",
  title: "XCM - Costs of managing foreign assets",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let address: string;
    let assetId: bigint;
    let assetLocation = RELAY_SOURCE_LOCATION_V4;
    let xcmLoc = patchLocationV4recursively(assetLocation);
    let decimals = 18;

    beforeAll(async () => {
    });

    // Cases
    // 1. Cost of creating an asset
    // 2. Cost of changing an asset's location
    // 3. Cost of freezing an asset
    // 4. Cost of unfreezing an asset
    // 6. Size of the created contract
    // 7. Cost of sending an asset
    it({
        id: "T01",
        "title": "Cost of creating an asset",
        test: async function () {

            const createAsset = context.polkadotJs().tx.sudo.sudo(
                context.polkadotJs().tx.evmForeignAssets.createForeignAsset(assetId, xcmLoc, decimals, "test", "test")
            );

            const { weight, proofSize } = await calculateWeight(context, createAsset);

            expect(weight).toMatchInlineSnapshot(`6287470000`);
            expect(proofSize).toMatchInlineSnapshot(`3284782`);
        }
    })

    it({
        id: "T01",
        "title": "Cost of changing an asset location",
        test: async function () {

            const changeLocation = context.polkadotJs().tx.sudo.sudo(
                context.polkadotJs().tx.evmForeignAssets.changeXcmLocation(assetId, xcmLoc)
            );

            const { weight, proofSize } = await calculateWeight(context, changeLocation);

            expect(weight).toMatchInlineSnapshot(`479200000`);
            expect(proofSize).toMatchInlineSnapshot(`9906`);
        }
    })

    it({
        id: "T01",
        "title": "Cost of freezing an asset",
        test: async function () {

            const freezeAsset = context.polkadotJs().tx.sudo.sudo(
                context.polkadotJs().tx.evmForeignAssets.freezeForeignAsset(assetId, false)
            );

            const { weight, proofSize } = await calculateWeight(context, freezeAsset);

            expect(weight).toMatchInlineSnapshot(`4945940000`);
            expect(proofSize).toMatchInlineSnapshot(`3302014`);
        }
    })

    it({
        id: "T01",
        "title": "Cost of unfreezing an asset",
        test: async function () {

            const unfreezeAsset = context.polkadotJs().tx.sudo.sudo(
                context.polkadotJs().tx.evmForeignAssets.freezeForeignAsset(assetId, true)
            );

            const { weight, proofSize } = await calculateWeight(context, unfreezeAsset);

            expect(weight).toMatchInlineSnapshot(`4945940000`);
            expect(proofSize).toMatchInlineSnapshot(`3302014`);
        }
    })

    it({
        id: "T01",
        "title": "Size of the created contract",
        test: async function () {
            const someBalance = 100_000_000_000_000n;
            const assetLocation = RELAY_SOURCE_LOCATION_V4;
            const assetId = 1n;

            // Register the asset
            const {contractAddress, registeredAssetId } = await registerForeignAsset(context, assetId, assetLocation, relayAssetMetadata);
            // Mock asset balance
            await mockAssetBalance(context, someBalance, assetId, alith, ALITH_ADDRESS);

            const newBalance = await foreignAssetBalance(context, assetId, ALITH_ADDRESS);
            expect(newBalance).toBe(someBalance);


            const { request } = await context.viem().simulateContract({
                address: contractAddress as `0x${string}`,
                abi: fetchCompiledContract("ERC20Instance").abi,
                functionName: "transfer",
                args: [BALTATHAR_ADDRESS, someBalance / 2n],
            });

            const hash = await context.viem().writeContract(request);

            await context.createBlock();

            const receipt = await context.viem().getTransactionReceipt({hash});

            console.log(receipt);

            expect(receipt.status).toBe('success');
            expect(receipt.gasUsed).toMatchInlineSnapshot(`154976n`);

            const transferredBalance = await foreignAssetBalance(context, assetId, BALTATHAR_ADDRESS);
            expect(transferredBalance).toBe(someBalance / 2n);

        }
    })
  },
});

async function calculateWeight(context, transaction) {
    const info = await context.polkadotJs().call.transactionPaymentApi.queryInfo(transaction.toU8a(), transaction.encodedLength);
    return {
        weight: info.weight.refTime,
        proofSize: info.weight.proofSize
    }
}