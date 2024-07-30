import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import { u128 } from "@polkadot/types-codec";
import { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { Abi, decodeAbiParameters, encodeFunctionData } from "viem";
import { mockOldAssetBalance } from "../../../../helpers";

describeSuite({
  id: "D012803",
  title: "Precompiles - Assets-ERC20 Wasm",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let erc20Abi: Abi;
    let assetId: u128;
    let contractInstanceAddress: `0x${string}`;

    const ADDRESS_ERC20 = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
    const ASSET_ID = 42259045809535163221576417993425387648n;

    beforeAll(async () => {
      //TODO: Update test case to use xcm mocker
      // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
      // And we need relay tokens for issuing a transaction to be executed in the relay
      const balance = 100000000000000n;
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

      await mockOldAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        ALITH_ADDRESS,
        true
      );

      const { abi, contractAddress } = await deployCreateCompiledContract(context, "ERC20Instance");
      erc20Abi = abi;
      contractInstanceAddress = contractAddress;
    });

    it({
      id: "T01",
      title: "allows to call name",
      test: async function () {
        const result = (
          await context.viem().call({
            to: ADDRESS_ERC20,
            data: encodeFunctionData({ abi: erc20Abi, functionName: "name" }),
          })
        ).data;

        expect(
          decodeAbiParameters(
            (erc20Abi.find((f: any) => f.name === "name") as any).outputs,
            result as `0x${string}`
          )[0]
        ).equals("DOT");
      },
    });
    it({
      id: "T02",
      title: "allows to call name via wrapper",
      test: async function () {
        const result = (
          await context.viem().call({
            to: contractInstanceAddress,
            data: encodeFunctionData({ abi: erc20Abi, functionName: "name" }),
          })
        ).data;

        expect(
          decodeAbiParameters(
            (erc20Abi.find((f: any) => f.name === "name") as any).outputs,
            result as `0x${string}`
          )[0]
        ).equals("DOT");
      },
    });

    it({
      id: "T03",
      title: "allows to call symbol",
      test: async function () {
        const data = await context.viem().readContract({
          address: ADDRESS_ERC20,
          abi: erc20Abi,
          functionName: "symbol",
        });
        expect(data).equals("DOT");
      },
    });

    it({
      id: "T04",
      title: "allows to call symbol via wrapper",
      test: async function () {
        const data = await context.viem().readContract({
          address: contractInstanceAddress,
          abi: erc20Abi,
          functionName: "symbol",
        });
        expect(data).equals("DOT");
      },
    });

    it({
      id: "T05",
      title: "allows to call decimals",
      test: async function () {
        const data = await context.viem().readContract({
          address: ADDRESS_ERC20,
          abi: erc20Abi,
          functionName: "decimals",
        });
        expect(data).equals(12);
      },
    });

    it({
      id: "T06",
      title: "allows to call decimals via wrapper",
      test: async function () {
        const data = await context.viem().readContract({
          address: contractInstanceAddress,
          abi: erc20Abi,
          functionName: "decimals",
        });
        expect(data).equals(12);
      },
    });

    it({
      id: "T06",
      title: "allows to call getBalance",
      test: async function () {
        const data = await context.viem().readContract({
          address: ADDRESS_ERC20,
          abi: erc20Abi,
          functionName: "balanceOf",
          args: [ALITH_ADDRESS],
        });
        expect(data).equals(100000000000000n);
      },
    });

    it({
      id: "T07",
      title: "allows to call getBalance via wrapper",
      test: async function () {
        const data = await context.viem().readContract({
          address: contractInstanceAddress,
          abi: erc20Abi,
          functionName: "balanceOf",
          args: [ALITH_ADDRESS],
        });
        expect(data).equals(100000000000000n);
      },
    });

    it({
      id: "T08",
      title: "allows to call totalSupply",
      test: async function () {
        const data = await context.viem().readContract({
          address: ADDRESS_ERC20,
          abi: erc20Abi,
          functionName: "totalSupply",
        });
        expect(data).equals(100000000000000n);
      },
    });

    it({
      id: "T09",
      title: "allows to call totalSupply via wrapper",
      test: async function () {
        const data = await context.viem().readContract({
          address: contractInstanceAddress,
          abi: erc20Abi,
          functionName: "totalSupply",
        });
        expect(data).equals(100000000000000n);
      },
    });
  },
});
