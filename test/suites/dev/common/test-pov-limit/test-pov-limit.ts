import { beforeAll, deployCreateCompiledContract, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { Abi, concatHex, encodeDeployData, encodeFunctionData } from "viem";
import { deployHeavyContracts, expectEVMResult, getBlockDetails, HeavyContract } from "../../../../helpers";
import { alith, ALITH_ADDRESS, ALITH_PRIVATE_KEY, baltathar, BALTATHAR_ADDRESS, BALTATHAR_PRIVATE_KEY, CHARLETH_ADDRESS, CHARLETH_PRIVATE_KEY, createEthersTransaction, createRawTransfer, DOROTHY_ADDRESS, DOROTHY_PRIVATE_KEY, ETHAN_ADDRESS, ETHAN_PRIVATE_KEY, FAITH_ADDRESS, FAITH_PRIVATE_KEY } from "@moonwall/util";
import { SignatureOptions } from "@polkadot/types/types";
import { Wallet } from "ethers";
import { sendTransaction } from "viem/zksync";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D0121111",
  title: "PoV Limit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {

    function genRandomEthAccount() {
      let randomEthAccount = "0x";
      for (let j = 0; j < 40; j++) {
        randomEthAccount += Math.floor(Math.random() * 16).toString(16);
      }
      return randomEthAccount as `0x${string}`;
    }

    // return an array with [length] number of balance transfers to random accounts
    async function createEthTx(length: number) {
      const data: string[] = [];

      let alithNonce = await context
        .viem()
        .getTransactionCount({ address: alith.address as `0x${string}` });

      for (let i = 0; i < length; i++) {
        const randomEthAccount = genRandomEthAccount();
        data.push(await createRawTransfer(context, randomEthAccount, 100, {
          nonce: alithNonce++,
        }));
      }
      return data;
    }

    function tagInstr(tag: number): `0x${string}` {
      const padded = tag.toString(16).padStart(8, "0"); // 4-byte big-endian
      return `0x63${padded}50` as `0x${string}`;
    }

    async function createEthTxContracts(length: number) {
      const compiled = fetchCompiledContract("MultiplyBy7Fat");
      const padding = "00".repeat(24_000);              // STOPs to bloat size

      let alithNonce = await context.viem().getTransactionCount({
        address: alith.address as `0x${string}`,
      });
      let baltatharNonce = await context.viem().getTransactionCount({
        address: baltathar.address as `0x${string}`,
      });

      for (let i = 0; i < length; i++) {
        // 1️⃣ unique runtime tag for this instance
        const fatBytecode = concatHex([
          compiled.bytecode,
          tagInstr(i),              // 6 bytes: PUSH4 i ; POP
          `0x${padding}`,           // keep total < 24 576 bytes
        ]) as `0x${string}`;
        const abi = compiled.abi;

        // 2️⃣ encode constructor call-data
        const callData = encodeDeployData({
          abi: abi,
          bytecode: fatBytecode,
          args: [i],
        }) as `0x${string}`;

        // 3️⃣ send tx
        const signer = new Wallet((i % 2 === 0) ? ALITH_PRIVATE_KEY : BALTATHAR_PRIVATE_KEY, context.ethers().provider);
        await signer.sendTransaction({
          data: callData,
          nonce: (i % 2 === 0) ? alithNonce++ : baltatharNonce++,
          gasLimit: 5_000_000n,     // big enough for ~20 kB init-code
        });

        await new Promise(resolve => setTimeout(resolve, 1));
      }
    }

    async function createPolkadotTx(length: number) {
      const data: any[] = [];

      // Get the current nonce for alith
      const nonce = await context.polkadotJs().query.system.account(alith.address);
      let currentNonce = nonce.nonce.toNumber();

      // Get the current block hash for signing
      const blockHash = await context.polkadotJs().rpc.chain.getBlockHash();
      const genesisHash = await context.polkadotJs().rpc.chain.getBlockHash(0);
      const runtimeVersion = await context.polkadotJs().rpc.state.getRuntimeVersion();

      for (let i = 0; i < length; i++) {

        // data.push(
        context.polkadotJs().tx.balances.transferKeepAlive(genRandomEthAccount(), 100).signAndSend(
          alith,
          {
            nonce: currentNonce++,
          }
        )
        // );
        await new Promise(resolve => setTimeout(resolve, 1));

        if (i % 1000 === 0) {
          await new Promise(resolve => setTimeout(resolve, 100));
        }
      }

      return data;
    }


    async function heavyContractsMethod() {
      const { contractAddress, abi } = await deployCreateCompiledContract(context, "CallForwarder");

      const MAX_CONTRACTS = 40;

      const contracts = await deployHeavyContracts(context, 6000, 6000 + MAX_CONTRACTS);

      const callData = encodeFunctionData({
        abi: abi,
        functionName: "callRange",
        args: [contracts[0].account, contracts[MAX_CONTRACTS].account],
      });

      const accounts = [
        [ALITH_PRIVATE_KEY, ALITH_ADDRESS],
        [BALTATHAR_PRIVATE_KEY, BALTATHAR_ADDRESS],
        [CHARLETH_PRIVATE_KEY, CHARLETH_ADDRESS],
        [DOROTHY_PRIVATE_KEY, DOROTHY_ADDRESS],
        [ETHAN_PRIVATE_KEY, ETHAN_ADDRESS],
        [FAITH_PRIVATE_KEY, FAITH_ADDRESS],
      ];

      let nonce = (await context.polkadotJs().query.system.account(BALTATHAR_ADDRESS)).nonce.toNumber();

      for (let i = 0; i < 10; i++) {
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
        // await createPolkadotTx(1);

        // await createEthTxContracts(200);
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
      }
    });

  },
});
