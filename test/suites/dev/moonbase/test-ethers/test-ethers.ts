import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "moonwall";
import { ethers } from "ethers";
import { waitFor } from "../../../../helpers";

describeSuite({
  id: "D021301",
  title: "Ethers.js",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should get correct network ids",
      test: async function () {
        expect((await context.ethers().provider!.getNetwork()).chainId).to.equal(1281n);
      },
    });

    it({
      id: "T02",
      title: "should be deployable",
      test: async function () {
        const { abi, bytecode } = fetchCompiledContract("MultiplyBy7");
        const contractFactory = new ethers.ContractFactory(
          abi as ethers.InterfaceAbi,
          bytecode,
          context.ethers()
        );

        const contract = await contractFactory.deploy({
          gasLimit: 1_000_000,
          gasPrice: 10_000_000_000,
        });
        await context.createBlock();

        log(`Contract address: ${await contract.getAddress()}`);
        expect((await contract.getAddress()).length).toBeGreaterThan(3);
        expect(await context.ethers().provider?.getCode(await contract.getAddress())).to.be.string;
      },
    });

    it({
      id: "T03",
      title: "should be callable",
      test: async function () {
        const contractData = fetchCompiledContract("MultiplyBy7");
        const contractFactory = new ethers.ContractFactory(
          contractData.abi as ethers.InterfaceAbi,
          contractData.bytecode,
          context.ethers()
        );

        const deployed = await contractFactory.deploy({
          gasLimit: 1_000_000,
          gasPrice: 10_000_000_000,
          nonce: await context.ethers().getNonce(),
        });
        await context.createBlock();

        // The eth RPC "latest" view can briefly lag the freshly sealed block, so
        // the deployed bytecode may not be visible yet. Wait for it before making
        // the eth_call below, otherwise the call returns "0x" (BAD_DATA).
        const deployedAddress = await deployed.getAddress();
        await waitFor(
          async () => (await context.ethers().provider!.getCode(deployedAddress)) !== "0x"
        );

        // @ts-expect-error It doesn't know what functions are available
        const contractCallResult = await deployed.multiply(3, {
          gasLimit: 1_000_000,
          gasPrice: 10_000_000_000,
        });

        await context.createBlock();
        expect(contractCallResult.toString()).to.equal("21");

        // Instantiate contract from address
        const contractFromAddress = new ethers.Contract(
          await deployed.getAddress(),
          contractData.abi as ethers.InterfaceAbi,
          context.ethers()
        );
        await context.createBlock();
        expect(
          (
            await contractFromAddress.multiply(3, { gasLimit: 1_000_000, gasPrice: 10_000_000_000 })
          ).toString()
        ).to.equal("21");
      },
    });
  },
});
