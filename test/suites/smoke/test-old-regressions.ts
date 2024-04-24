import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { ethers } from "ethers";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "S15",
  title: "Verify regressions which happend in the past",
  foundationMethods: "read_only",
  testCases: async ({ context, it, log }) => {
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      const chainId = (await paraApi.query.ethereumChainId.chainId()).toString();
      log(`Loading test data for chainId ${chainId}.`);
    });

    it({
      id: "C001",
      title: "Verify MOON-2824",
      test: async function () {
        if (paraApi.consts.system.version.specName.toString() !== "moonriver") {
          log("Skipping... (Test specific for moonriver)");
          return; // TODO: replace this with this.skip() when added to vitest
        }
        const CONTRACT_ADDRESS = "0x1b30a3b5744e733d8d2f19f0812e3f79152a8777";
        const ERROR_STARTED_AT_BLOCK = `0x${(1471037).toString(16)}`;
        const abi = [{
          "inputs": [
            {
              "internalType": "address",
              "name": "who",
              "type": "address"
            },
            {
              "internalType": "uint256",
              "name": "n",
              "type": "uint256"
            }
          ],
          "name": "balanceOf",
          "outputs": [
            {
              "internalType": "uint256",
              "name": "",
              "type": "uint256"
            }
          ],
          "stateMutability": "view",
          "type": "function"
        }];
        const calldata = encodeFunctionData({
          abi,
          functionName: "balanceOf",
          args: ["0x30763be2bf075c3fDeA704c5f59A76d011d02943", 2],
        });
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).call(
          {
            to: CONTRACT_ADDRESS,
            data: calldata,
            // The error occurs between runtime 1201 and 1605
            // https://docs.moonbeam.network/builders/build/runtime-upgrades/
            blockTag: ERROR_STARTED_AT_BLOCK,

          },
        );

        expect(result).to.contain("0x");
      },
    });

    it({
      id: "C002",
      title: "Verify MOON-2822",
      test: async function () {
        if (paraApi.consts.system.version.specName.toString() !== "moonbeam") {
          log("Skipping... (Test specific for moonbeam)");
          return; // TODO: replace this with this.skip() when added to vitest
        }
        const CONTRACT_ADDRESS = "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080";
        const abi = [
          {
            "inputs": [],
            "name": "totalSupply",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
          }
        ];
        const calldata = encodeFunctionData({
          abi,
          functionName: "totalSupply",
          args: [],
        });
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).call(
          {
            to: CONTRACT_ADDRESS,
            from: "0xA9f7C749DdCd4E1b86eC539970DEA61a63A6CDD4",
            data: calldata,
            blockTag: "latest",
          },
        );

        expect(result).to.contain("0x");
      },
    });
  },
});
