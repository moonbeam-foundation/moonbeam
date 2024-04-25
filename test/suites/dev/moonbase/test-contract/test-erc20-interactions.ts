import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  GLMR,
} from "@moonwall/util";

describeSuite({
  id: "D010611",
  title: "ERC20 interactionss",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let contract: `0x${string}`;

    beforeEach(async function () {
      const { contractAddress } = await context.deployContract!("ERC20Sample");
      contract = contractAddress;
    });

    it({
      id: "T01",
      title: "Should get the greeter message from the contract",
      test: async function () {
        const response = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "greeter",
        });

        expect(response).toEqual("Hello, ERC20!");
      },
    });

    it({
      id: "T02",
      title: "Should mint as expected",
      test: async function () {
        const tx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "mint",
          args: [BALTATHAR_ADDRESS, 10n * GLMR],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(tx);

        const bal = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [BALTATHAR_ADDRESS],
        });

        expect(result?.successful).toBe(true);
        expect(bal).toEqual(10n * GLMR);
      },
    });

    it({
      id: "T03",
      title: "Should burn as expected",
      test: async function () {
        const amount = 10n * GLMR;

        const balBefore = (await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [ALITH_ADDRESS],
        })) as bigint;

        const tx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "burn",
          args: [amount],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(tx);

        const balAfter = (await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [ALITH_ADDRESS],
        })) as bigint;

        expect(result?.successful).toBe(true);
        expect(balBefore - balAfter).toEqual(amount);
      },
    });

    it({
      id: "T04",
      title: "Should approve as expected",
      test: async function () {
        const tx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "approve",
          args: [BALTATHAR_ADDRESS, 7n * GLMR],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(tx);

        const approval = (await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "allowance",
          args: [ALITH_ADDRESS, BALTATHAR_ADDRESS],
        })) as bigint;

        expect(result?.successful).toBe(true);
        expect(approval).toEqual(7n * GLMR);
      },
    });

    it({
      id: "T05",
      title: "Should transfer as expected",
      test: async function () {
        const tx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "transfer",
          args: [BALTATHAR_ADDRESS, 10n * GLMR],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(tx);

        const bal = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [BALTATHAR_ADDRESS],
        });

        expect(result?.successful).toBe(true);
        expect(bal).toEqual(10n * GLMR);
      },
    });

    it({
      id: "T06",
      title: "Should transferFrom as expected",
      test: async function () {
        await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "approve",
          args: [CHARLETH_ADDRESS, 3n * GLMR],
        });
        await context.createBlock();

        await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "transferFrom",
          args: [ALITH_ADDRESS, BALTATHAR_ADDRESS, 3n * GLMR],
          privateKey: CHARLETH_PRIVATE_KEY,
        });
        await context.createBlock();

        const bal = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [BALTATHAR_ADDRESS],
        });

        expect(bal).toEqual(3n * GLMR);
      },
    });

    // TODO mint via XCM

    // TODO burn via XCM

    // TODO transferFrom via XCM
  },
});
