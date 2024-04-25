import { beforeAll, describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "D010611",
  title: "ERC20 interactionss",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let contract: `0x${string}`;

    beforeAll(async function () {
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

    // TODO add test for minting

    // TODO add test for burning

    // TODO add test for approval

    // TODO add test for transfer

    // TODO add test for transferFrom

    // TODO mint via XCM

    // TODO burn via XCM

    // TODO transferFrom via XCM
  },
});
