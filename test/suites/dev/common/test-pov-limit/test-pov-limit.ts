import { deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { encodeFunctionData } from "viem";
import { deployHeavyContracts, getBlockDetails } from "../../../../helpers";
import { baltathar, BALTATHAR_ADDRESS } from "@moonwall/util";
import { Wallet } from "ethers";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D0121111",
  title: "PoV Limit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    async function heavyContractsMethod() {
      const { contractAddress, abi } = await deployCreateCompiledContract(context, "CallForwarder");

      const MAX_CONTRACTS = 40;

      const contracts = await deployHeavyContracts(context, 6000, 6000 + MAX_CONTRACTS);

      const callData = encodeFunctionData({
        abi: abi,
        functionName: "callRange",
        args: [contracts[0].account, contracts[MAX_CONTRACTS].account],
      });

      const accounts: [string, string][] = [];

      let nonce = (await context.polkadotJs().query.system.account(BALTATHAR_ADDRESS)).nonce.toNumber();

      for (let i = 0; i < 15; i++) {
        const randomPrivateKey = generatePrivateKey();
        const randomAddress = privateKeyToAccount(randomPrivateKey as `0x${string}`).address;
        accounts.push([randomPrivateKey as `0x${string}`, randomAddress]);

        context.polkadotJs().tx.balances.transferKeepAlive(randomAddress, 1_000_000_000_000_000_000n * 1000n).signAndSend(
          baltathar,
          {
            nonce: nonce++,
          }
        );
      }

      await context.createBlock();


      for (const [privateKey, address] of accounts) {

        const gasEstimate = await context.viem().estimateGas({
          account: address,
          to: contractAddress,
          value: 0n,
          data: callData,
        });

        const signer = new Wallet(privateKey, context.ethers().provider);

        const rawSigned = await signer.sendTransaction({
          to: contractAddress,
          data: callData,
          gasLimit: gasEstimate,
        });

        await new Promise(resolve => setTimeout(resolve, 1));
      }

    }

    it({
      id: "T01",
      title: "Test PoV Limit",
      test: async function () {
        await heavyContractsMethod();

        const res = await context.createBlock();

        const blockDetails = await getBlockDetails(context.polkadotJs(), res.block.hash);

        console.log(`Number of added extrinsics: ${blockDetails.txWithEvents.length}`);

        const blockWeight = await context.polkadotJs().query.system.blockWeight();

        const proofSize = blockWeight.normal.proofSize.toBigInt() +
          blockWeight.operational.proofSize.toBigInt() +
          blockWeight.mandatory.proofSize.toBigInt();

        const fullPov = 10 * 1024 * 1024;
        const floatPov = parseFloat(proofSize.toString());
        console.log(`Proof size: ${proofSize} bytes (${(floatPov / fullPov) * 100}% of FullPov 10MB)`);
        console.log(`Proof size: ${floatPov / 1024} KB`);
        console.log(`Proof size: ${floatPov / 1024 / 1024} MB`);

        // 75% of 10MB
        const limit = 10 * 1024 * 1024 * 0.75;
        expect(floatPov).toBeGreaterThanOrEqual(limit);
      }
    });

  },
});
