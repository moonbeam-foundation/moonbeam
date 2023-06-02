import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { getCompiled } from "@moonwall/util";
import { ethers } from "ethers";

describeSuite({
  id: "D1401",
  title: "Ethers.js",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should get correct network ids",
      test: async function () {
        expect((await context.ethersSigner().provider!.getNetwork()).chainId).to.equal(1281n);
      },
    });

    it({
      id: "T02",
      title: "should be deployable",
      test: async function () {
        const contractData = getCompiled("MultiplyBy7");
        const contractFactory = new ethers.ContractFactory(
          contractData.contract.abi as ethers.InterfaceAbi,
          contractData.byteCode,
          context.ethersSigner()
        );

        const contract = await contractFactory.deploy({
          gasLimit: 1_000_000,
          gasPrice: 10_000_000_000,
        });
        await context.createBlock();

        log("Contract address: ", await contract.getAddress());
        expect((await contract.getAddress()).length).toBeGreaterThan(3);
        expect(await context.ethersSigner().provider?.getCode(await contract.getAddress())).to.be
          .string;
      },
    });

    it({
      id: "T03",
      title: "should be callable",
      test: async function () {
        const contractData = getCompiled("MultiplyBy7");
        const contractFactory = new ethers.ContractFactory(
          contractData.contract.abi as ethers.InterfaceAbi,
          contractData.byteCode,
          context.ethersSigner()
        );

        const deployed = await contractFactory.deploy({
          gasLimit: 1_000_000,
          gasPrice: 10_000_000_000,
          nonce: await context.ethersSigner().getNonce(),
        });
        await context.createBlock();

        // @ts-expect-error
        const contractCallResult = await deployed.multiply(3, {
          gasLimit: 1_000_000,
          gasPrice: 10_000_000_000,
        });

        await context.createBlock();
        expect(contractCallResult.toString()).to.equal("21");

        // Instantiate contract from address
        const contractFromAddress = new ethers.Contract(
          await deployed.getAddress(),
          contractData.contract.abi as ethers.InterfaceAbi,
          context.ethersSigner()
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
