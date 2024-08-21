import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect, customDevRpcRequest } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { error } from "console";
import { ethers } from "ethers";
import { encodeFunctionData, Hash } from "viem";

// Each case has
// - Contract Address
// - Error started at block
// - Call data

enum Network {
  Moonbeam = "moonbeam",
  Moonriver = "moonriver",
  Moonbase = "moonbase",
}

// Issues/Regressions
class BadBlockRegressionCase {
  issue: string;
  network: Network;
  contractAddress: `0x${string}`;
  blockTag: string;
  callData: `0x${string}`;
}
// MOON-2824
const moon2824: BadBlockRegressionCase = {
  issue: "MOON-2824",
  network: Network.Moonriver,
  contractAddress: "0x1b30a3b5744e733d8d2f19f0812e3f79152a8777",
  blockTag: `0x${(1471037).toString(16)}`,
  callData: encodeFunctionData({
    abi: [
      {
        inputs: [
          {
            internalType: "address",
            name: "who",
            type: "address",
          },
          {
            internalType: "uint256",
            name: "n",
            type: "uint256",
          },
        ],
        name: "balanceOf",
        outputs: [
          {
            internalType: "uint256",
            name: "",
            type: "uint256",
          },
        ],
        stateMutability: "view",
        type: "function",
      },
    ],
    functionName: "balanceOf",
    args: ["0x30763be2bf075c3fDeA704c5f59A76d011d02943", 2n],
  }),
};

// MOON-2822
const moon2822: BadBlockRegressionCase = {
  issue: "MOON-2822",
  network: Network.Moonbeam,
  contractAddress: "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080",
  blockTag: "latest",
  callData: encodeFunctionData({
    abi: [
      {
        inputs: [],
        name: "totalSupply",
        outputs: [],
        stateMutability: "nonpayable",
        type: "function",
      },
    ],
    functionName: "totalSupply",
    args: [],
  }),
};

// Group all cases
const cases = [moon2824, moon2822];

describeSuite({
  id: "S15",
  title: "Verify regressions which happened in the past by reading historical state",
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
      title: "Verify all bad block regression cases",
      test: async function () {
        for (const testCase of cases) {
          if (testCase.network != (paraApi.consts.system.version.specName.toString() as Network)) {
            log(`Skipping... (Issue ${testCase.issue} specific for ${testCase.network})`);
            continue;
          }

          const result = await (context.ethers().provider as ethers.JsonRpcProvider).call({
            to: testCase.contractAddress,
            data: testCase.callData,
            blockTag: testCase.blockTag,
          });

          log(`Result for ${testCase.issue} at block ${testCase.blockTag}: ${result}`);
          expect(result).toBe("0x");
        }
      },
    });
  },
});
