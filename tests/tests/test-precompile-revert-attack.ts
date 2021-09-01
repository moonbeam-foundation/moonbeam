// Moon-808
// What happens if one calls
// function score_a_free_nomination() public payable{

//     // We nominate our target collator with all the tokens provided
//     staking.nominate(target, msg.value);
//     revert("By reverting this transaction, we return the eth to the caller");
// }
// Would the nomination pass in subtrate but get the eth back in the evm?
// We have to make sure that's not possible

import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { GENESIS_ACCOUNT } from "../util/constants";
import { getCompiled } from "../util/contracts";
import { createContract, createContractExecution } from "../util/transactions";

describeDevMoonbeam("Estimate Gas - Contract creation", (context) => {
  it("should return contract creation gas cost", async function () {
    // Check initial balance
    const initialBalance = await context.web3.eth.getBalance(GENESIS_ACCOUNT);
    // Deploy atatck contract
    const { contract, rawTx } = await createContract(context.web3, "StakingNominationAttaker");
    await context.createBlock({ transactions: [rawTx] });

    await context.createBlock({
      transactions: [
        await createContractExecution(context.web3, {
          contract,
          contractCall: contract.methods.score_a_free_nomination(),
        }),
      ],
    });

    // balance should still be the same
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT)).to.eq(initialBalance);
  });
});
