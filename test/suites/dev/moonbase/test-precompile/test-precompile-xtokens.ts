import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, PRECOMPILES, createViemTransaction } from "@moonwall/util";
import {
  verifyLatestBlockFees,
  expectEVMResult,
  DEFAULT_TXN_MAX_BASE_FEE,
} from "../../../../helpers";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D012802",
  title: "Precompiles - xtokens",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      await context.deployContract!("XTokensInstance");
    });
    it({
      id: "T01",
      title: "allows to issue transfer xtokens",
      test: async function () {
        const destination_enum_selector = "0x01";
        // [0x01; 32]
        const destination_address =
          "0101010101010101010101010101010101010101010101010101010101010101";
        const destination_network_id = "00";

        // This represents X2(Parent, AccountId32([0x01; 32]))
        // We will transfer the tokens the former account in the relay chain
        // However it does not really matter as we are not testing what happens
        // in the relay side of things
        const destination = [
          1,
          // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
          [destination_enum_selector + destination_address + destination_network_id],
        ];
        const amountTransferred = 1000n;
        const weight = 100n;

        const balBefore = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const rawTxn = await context.writePrecompile!({
          precompileName: "Xtokens",
          functionName: "transfer",
          args: [PRECOMPILES.NativeErc20[0], amountTransferred, destination, weight],
          rawTxOnly: true,
          gas: 500_000n,
        });

        const { result } = await context.createBlock(rawTxn);

        const balAfter = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        const gasPrice = receipt.effectiveGasPrice;
        const fees = receipt.gasUsed * gasPrice;
        expectEVMResult(result!.events, "Succeed");
        await verifyLatestBlockFees(context, amountTransferred);
        expect(balBefore - balAfter).to.equal(amountTransferred + fees);
      },
    });

    it({
      id: "T03",
      title: "allows to issue transfer_multiasset xtokens",
      test: async function () {
        const destination_enum_selector = "0x01";
        // [0x01; 32]
        const destination_address =
          "0101010101010101010101010101010101010101010101010101010101010101";
        const destination_network_id = "00";

        // Junction::PalletInstance(3)
        const x2_pallet_instance_enum_selector = "0x04";
        const x2_instance = "03";

        // This represents X1(PalletInstance(3)))

        // This multilocation represents our native token
        const asset = [
          // zero parents
          0,
          // X1(PalletInstance)
          // PalletInstance: Selector (04) + palconst instance 1 byte (03)
          [x2_pallet_instance_enum_selector + x2_instance],
        ];
        // This represents X2(Parent, AccountId32([0x01; 32]))
        // We will transfer the tokens the former account in the relay chain
        // However it does not really matter as we are not testing what happens
        // in the relay side of things
        const destination = [
          1,
          // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
          [destination_enum_selector + destination_address + destination_network_id],
        ];
        const amountTransferred = 1000n;
        const weight = 100;

        const balBefore = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const rawTxn = await context.writePrecompile!({
          precompileName: "Xtokens",
          functionName: "transferMultiasset",
          args: [asset, amountTransferred, destination, weight],
          gas: 1_200_000n,
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);
        const balAfter = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expectEVMResult(result!.events, "Succeed");
        const gasPrice = receipt.effectiveGasPrice;
        const fees = receipt.gasUsed * gasPrice;

        expect(balBefore - balAfter).to.equal(amountTransferred + fees);
        await verifyLatestBlockFees(context, amountTransferred);
      },
    });

    it({
      id: "T05",
      title: "allows to issue transfer multicurrencies xtokens",
      test: async function () {
        const destination_enum_selector = "0x01";
        // [0x01; 32]
        const destination_address =
          "0101010101010101010101010101010101010101010101010101010101010101";
        // NetworkId::Any
        const destination_network_id = "00";
        const amountTransferred = 1000n;
        const currencies = [[PRECOMPILES.NativeErc20[0], amountTransferred]];

        // This represents X2(Parent, AccountId32([0x01; 32]))
        // We will transfer the tokens the former account in the relay chain
        // However it does not really matter as we are not testing what happens
        // in the relay side of things
        const destination = [
          1,
          // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
          [destination_enum_selector + destination_address + destination_network_id],
        ];

        const fee_item = 0;
        const weight = 100;

        const balBefore = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const rawTxn = await context.writePrecompile!({
          precompileName: "Xtokens",
          functionName: "transferMultiCurrencies",
          args: [currencies, fee_item, destination, weight],
          gas: 1_200_000n,
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);
        const balAfter = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });
        expectEVMResult(result!.events, "Succeed");

        const gasPrice = receipt.effectiveGasPrice;
        const fees = receipt.gasUsed * gasPrice;

        expect(balBefore - balAfter).to.equal(amountTransferred + fees);
        await verifyLatestBlockFees(context, amountTransferred);
      },
    });

    it({
      id: "T06",
      title: "allows to issue transfer multiassets xtokens",
      test: async function () {
        const destination_enum_selector = "0x01";
        // [0x01; 32]
        const destination_address =
          "0101010101010101010101010101010101010101010101010101010101010101";
        // NetworkId::Any
        const destination_network_id = "00";
        const amountTransferred = GLMR;

        // Junction::PalletInstance(3)
        const x2_pallet_instance_enum_selector = "0x04";
        const x2_instance = "03";

        // This multilocation represents our native token
        const asset = [
          // one parent
          0,
          // X1(PalletInstance)
          // PalletInstance: Selector (04) + palconst instance 1 byte (03)
          [x2_pallet_instance_enum_selector + x2_instance],
        ];

        const multiassets = [[asset, amountTransferred]];

        // This represents X2(Parent, AccountId32([0x01; 32]))
        // We will transfer the tokens the former account in the relay chain
        // However it does not really matter as we are not testing what happens
        // in the relay side of things
        const destination =
          // Destination as multilocation
          [
            // one parent
            1,
            // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
            [destination_enum_selector + destination_address + destination_network_id],
          ];

        const fee_item = 0;
        const weight = 100;

        const balBefore = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const { abi } = fetchCompiledContract("Xtokens");
        const data = encodeFunctionData({
          abi,
          functionName: "transferMultiAssets",
          args: [multiassets, fee_item, destination, weight],
        });

        const rawTxn = await createViemTransaction(context, {
          to: PRECOMPILES.Xtokens,
          value: 0n,
          data,
          txnType: "legacy",
          gas: 500_000n,
          gasPrice: BigInt(DEFAULT_TXN_MAX_BASE_FEE),
        });

        const { result } = await context.createBlock(rawTxn);
        const balAfter = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expectEVMResult(result!.events, "Succeed");

        const fees = receipt.gasUsed * BigInt(DEFAULT_TXN_MAX_BASE_FEE);
        expect(balBefore - balAfter).to.equal(amountTransferred + fees);
        await verifyLatestBlockFees(context, amountTransferred);
      },
    });
  },
});
