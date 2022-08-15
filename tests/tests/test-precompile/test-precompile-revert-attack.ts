// Moon-808
// What happens if one calls
// function score_a_free_delegation() public payable{
import "@moonbeam-network/api-augment";

import { numberToHex } from "@polkadot/util";
import { expect } from "chai";

import { alith } from "../../util/accounts";
import { MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createContractExecution,
} from "../../util/transactions";

//     // We delegate our target collator with all the tokens provided
//     staking.delegate(target, msg.value);
//     revert("By reverting this transaction, we return the eth to the caller");
// }
// Would the delegation pass in substrate but get the eth back in the evm?
// We have to make sure that's not possible

describeDevMoonbeamAllEthTxTypes("Precompiles - Reverting Staking precompile", (context) => {
  it("should not revert the whole transaction cost", async function () {
    // Check initial balance
    const initialBalance = await context.web3.eth.getBalance(alith.address);
    // Deploy attack contract
    const { contract, rawTx } = await createContract(context, "StakingAttacker");
    await context.createBlock(rawTx);

    // call the payable function, which should revert
    const { result } = await context.createBlock(
      createContractExecution(
        context,
        {
          contract,
          contractCall: contract.methods.score_a_free_delegation(),
        },
        {
          ...ALITH_TRANSACTION_TEMPLATE,
          value: numberToHex(Number(MIN_GLMR_STAKING)),
        }
      )
    );

    // TX should be included but fail
    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.eq(false);

    // Delegation shouldn't have passed
    const nominatorsAfter = await context.polkadotApi.query.parachainStaking.delegatorState(
      alith.address
    );
    expect(nominatorsAfter.toHuman()).to.eq(null);

    // balance dif should only be tx fee, not MIN_GLMR_STAKING
    expect(
      Number(initialBalance) - Number(await context.web3.eth.getBalance(alith.address)) <
        Number(MIN_GLMR_STAKING)
    ).to.eq(true);
  });
});
