import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, fetchCompiledContract, expect, deployCreateCompiledContract } from "@moonwall/cli";
import {
    ALITH_ADDRESS,
    BALTATHAR_ADDRESS,
    BALTATHAR_PRIVATE_KEY,
    ALITH_PRIVATE_KEY,
    CHARLETH_ADDRESS,
    createViemTransaction,
    sendRawTransaction,
    alith,
    createEthersTransaction
} from "@moonwall/util";
import { u128 } from "@polkadot/types-codec";
import { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { fromBytes, encodeFunctionData } from "viem";
import {
  verifyLatestBlockFees,
  expectEVMResult,
  registerXcmTransactorAndContract,
  mockAssetBalance,
  extractRevertReason
} from "../../../../helpers";

const PRECOMPILE_PALLET_XCM_ADDRESS: `0x${string}` = "0x000000000000000000000000000000000000081A";

describeSuite({
  id: "D012900",
  title: "Precompiles - PalletXcm",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let assetId: u128;
    const ASSET_ID = 42259045809535163221576417993425387648n;
    beforeAll(async () => {
      //await registerXcmTransactorAndContract(context);

      const balance = 200000000000000n;
      const assetBalance: PalletAssetsAssetAccount = context
        .polkadotJs()
        .createType("PalletAssetsAssetAccount", {
          balance: balance,
        });
      assetId = context.polkadotJs().createType("u128", ASSET_ID);

      const assetDetails: PalletAssetsAssetDetails = context
        .polkadotJs()
        .createType("PalletAssetsAssetDetails", {
          supply: balance,
        });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        ALITH_ADDRESS,
        true
      );
    });

    it({
      id: "T01",
      title: "allows to call transfer_assets function",
      test: async function () {

        const assetBalance = await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS);

        console.log("ASSET BALANCE:", assetBalance.toHuman())

        const { abi: xcmInterface } = fetchCompiledContract("XCM");
        //const deployResult = await deployCreateCompiledContract(context, "XCM");


        //console.log(xcmInterface[0]["inputs"][0])

        const evmCode = context.polkadotJs().query.evm.accountCodes.key(PRECOMPILE_PALLET_XCM_ADDRESS);
        console.log("EVM CODE: ", evmCode);

        const dest: [number, any[]] = [1, []];
        //const asset: [number, any[]] = [1,[]];

/*         const assetLocation: [number, any[]] = [
            // one parent
            0,
            // X1(PalletInstance)
            // PalletInstance: Selector (04) + palconst instance 1 byte (03)
            ["0x04" + "03"],
        ]; */

        const assetLocation: [number, any[]] = [
            // one parent
            1,
            // X1(PalletInstance)
            // PalletInstance: Selector (04) + palconst instance 1 byte (03)
            [],
        ];

        const destination_address =
          "0101010101010101010101010101010101010101010101010101010101010101";
        // NetworkId::Any
        const destination_network_id = "00";
        const beneficiary: [number, any[]] = [
            0, 
            // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
            ["0x01" + destination_address + destination_network_id]
        ]

        const assetsToSend = [[assetLocation, 100000000000000n]];
        const weight = { refTime: 4000000000, proofSize: 80000 };

        let aliceNonce = (await context.polkadotJs().query.system.account(ALITH_ADDRESS)).nonce.toNumber();

/*         const transferAssetsTx = await createViemTransaction(context, {
            to: PRECOMPILE_PALLET_XCM_ADDRESS,
            gas: 500_000n,
            nonce: aliceNonce++,
            data: encodeFunctionData({
                abi: xcmInterface,
                functionName: "transferAssets",
                args: [dest, beneficiary, assetsToSend, 0, weight],
            }),
        });

        const transferAssetsResult = await sendRawTransaction(context, transferAssetsTx);
             
        await context.createBlock();

        const receipt = await context
            .viem("public")
            .getTransactionReceipt({ hash: transferAssetsResult as `0x${string}` });
        console.log("RECEIPT: ", receipt); */

        // ---------Section 2 -----------
/*         const events = result.result?.events;

        events?.forEach((event) => {
            //console.log(event.toHuman())
            if (event["event"]["section"] == "ethereum") {
                console.log(event["event"]["data"]["exitReason"].toHuman())
            }
        }) */
        //console.log("RESULT: ", result.result?.events); 

/*         const rawTxn = await context.writePrecompile!({
          precompileName: "PalletXcm",
          functionName: "transferAssets",
          args: [dest, beneficiary, assetsToSend, 0, weight],
          gas: 500_000n,
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed"); */

        // 1000 fee for the relay is paid with relay assets
        //await verifyLatestBlockFees(context);

        const rawTxn = await createEthersTransaction(context, {
            to: PRECOMPILE_PALLET_XCM_ADDRESS,
            data: encodeFunctionData({
              abi: xcmInterface,
              args: [dest, beneficiary, assetsToSend, 0, weight],
              functionName: "transferAssets",
            }),
            gasLimit: 13_000_000,
          });
          const result = await context.createBlock(rawTxn);
  
          expectEVMResult(result.result!.events, "Succeed");
          const revertReason = await extractRevertReason(context, result.result!.hash);
          
          const ethereumResult = result.result?.events.find(
            ({ event: { section, method } }) => section == "ethereum" && method == "Executed"
          )!.event.data[3].toHuman();

          console.log("Ethereum Result: ", ethereumResult)

          const assetBalanceAfter = await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS);

          console.log("Result: ", revertReason)
          console.log("Asset Balance after: ", assetBalanceAfter.unwrap().balance.toBigInt());
          //expect(revertReason).to.contain(`NotSuicided:`);


          //----------Section 3-----------
/*           const rawTx = await context.writeContract!({
            contractAddress: PRECOMPILE_PALLET_XCM_ADDRESS,
            contractName: "XCM",
            functionName: "transferAssets",
            args: [dest, beneficiary, assetsToSend, 0n, weight],
            gas: 500_000n,
            rawTxOnly: true,
            privateKey: ALITH_PRIVATE_KEY,
          });
  
          const result = await context.createBlock(rawTx);
          expectEVMResult(result.result!.events, "Succeed"); */
      },
    });
  },
});