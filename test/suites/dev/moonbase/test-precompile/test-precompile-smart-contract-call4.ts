import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, baltathar } from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../../helpers";

describeSuite({
  id: "D012879",
  title: "Smart Contract Precompile Call - Proxy - Real Account",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let testContractAddress: `0x${string}`;
    let testContract2Address: `0x${string}`;
    let multiplyContractAddress: `0x${string}`;

    beforeAll(async function () {
      const { contractAddress: addr1 } = await context.deployContract!(
        "SmartContractPrecompileCallTest"
      );
      testContractAddress = addr1;

      const { contractAddress: addr2 } = await context.deployContract!(
        "SmartContractPrecompileCallTest"
      );
      testContract2Address = addr2;

      const { contractAddress: addr3 } = await context.deployContract!("MultiplyBy7");
      multiplyContractAddress = addr3;

      await context.createBlock(
        context.polkadotJs().tx.proxy.addProxy(testContractAddress, "Any", 0).signAsync(baltathar)
      );

      // Add proxy from a canary smart contract to the test smart contract via setStorage
      const storageKey = context.polkadotJs().query.proxy.proxies.key(testContract2Address);
      const storageValue = context
        .polkadotJs()
        .registry.createType("(Vec<PalletProxyProxyDefinition>,u128)", [
          [
            {
              delegate: testContractAddress,
              proxyType: "Any",
              delay: 0,
            },
          ],
          1002900000000000000n,
        ])
        .toHex();

      await context.createBlock(
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.system.setStorage([[storageKey, storageValue]]))
      );
    });

    it({
      id: "T01",
      title:
        "should revert when caller is a smart contract and real address is \
        smart contract",
      test: async function () {
        const rawTxn = await context.writeContract!({
          contractAddress: testContractAddress,
          contractName: "SmartContractPrecompileCallTest",
          functionName: "callProxy",
          gas: 5_000_000n,
          rawTxOnly: true,
          args: [
            testContract2Address,
            multiplyContractAddress,
            encodeFunctionData({
              abi: fetchCompiledContract("MultiplyBy7").abi,
              functionName: "multiply",
              args: [5],
            }),
          ],
        });

        const { result } = await context.createBlock(rawTxn);

        expectEVMResult(result!.events, "Revert");
        expect(
          async () =>
            await context.writeContract!({
              contractAddress: testContractAddress,
              contractName: "SmartContractPrecompileCallTest",
              functionName: "callProxy",
              args: [
                testContract2Address,
                multiplyContractAddress,
                encodeFunctionData({
                  abi: fetchCompiledContract("MultiplyBy7").abi,
                  functionName: "multiply",
                  args: [5],
                }),
              ],
            })
        ).rejects.toThrowError("real address must be EOA");
      },
    });
  },
});
