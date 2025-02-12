import { DevModeContext, expect } from "@moonwall/cli";
import { injectHrmpMessageAndSeal, RawXcmMessage, XcmFragment } from "../../../../helpers";

export async function expectEvent(context: DevModeContext, blockHash: `0x${string}`, eventName: string) {
    const apiAt = await context.polkadotJs().at(blockHash);
    const events = await apiAt.query.system.events();
    const event = events.find(({ event: { method } }) => method.toString() === eventName)!.event;
    expect(event).to.exist;
    return event;
}

export const getPalletIndex = async (name: string, context: DevModeContext) => {
    const metadata = await context.polkadotJs().rpc.state.getMetadata();
    return metadata.asLatest.pallets
        .find(({ name: palletName }) => palletName.toString() === name)!
        .index.toNumber();
};

export const getForeignAssetDetails = async (assetId: number, context: DevModeContext) => {
    const createdForeignAsset = (
        await context.polkadotJs().query.evmForeignAssets.assetsById(assetId)
    ).toJSON();
    const assetDetails = {
        parents: createdForeignAsset!["parents"],
        interior: {
        X3: [
            { Parachain: createdForeignAsset!["interior"]["x3"][0]["parachain"] },
            { PalletInstance: createdForeignAsset!["interior"]["x3"][1]["palletInstance"] },
            { GeneralIndex: createdForeignAsset!["interior"]["x3"][2]["generalIndex"] },
        ],
        },
    };
    return assetDetails;
};

export const sendCallAsPara = async (
    call: any,
    paraId: number,
    fungible: bigint = 10_000_000_000_000_000_000n, // Default 10 GLMR
    context: DevModeContext
) => {
    const encodedCall = call.method.toHex();
    const balancesPalletIndex = await getPalletIndex("Balances", context);

    const xcmMessage = new XcmFragment({
        assets: [
        {
            multilocation: {
            parents: 0,
            interior: {
                X1: { PalletInstance: balancesPalletIndex },
            },
            },
            fungible: fungible
        },
        ],
        weight_limit: {
        refTime: 40_000_000_000n,
        proofSize: 120_000n,
        },
    })
        .withdraw_asset()
        .buy_execution()
        .push_any({
        Transact: {
            originKind: "Xcm",
            requireWeightAtMost: {
            refTime: 20_089_165_000n,
            proofSize: 80_000n,
            },
            call: {
            encoded: encodedCall,
            },
        },
        })
        .as_v4();

    // Send an XCM and create block to execute it
    const block = await injectHrmpMessageAndSeal(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
    } as RawXcmMessage);

    return block;
}