// Moon-808
// What happens if one calls
// function score_a_free_delegation() public payable{

//     // We delegate our target collator with all the tokens provided
//     staking.delegate(target, msg.value);
//     revert("By reverting this transaction, we return the eth to the caller");
// }
// Would the delegation pass in substrate but get the eth back in the evm?
// We have to make sure that's not possible

import { expect } from "chai";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

import { GENESIS_ACCOUNT, MIN_GLMR_STAKING } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import {
  createContract,
  createContractExecution,
  GENESIS_TRANSACTION,
} from "../../util/transactions";
import { numberToHex } from "@polkadot/util";

describeDevMoonbeamAllEthTxTypes(
  "Precompiles - test revert attack on state modifier",
  (context) => {
    it("should return contract creation gas cost", async function () {
      // Check initial balance
      const initialBalance = await context.web3.eth.getBalance(GENESIS_ACCOUNT);
      // Deploy atatck contract
      const { contract, rawTx } = await createContract(context, "StakingDelegationAttaker");
      await context.createBlock({ transactions: [rawTx] });

      // call the payable function, which should revert
      const block = await context.createBlock({
        transactions: [
          await createContractExecution(
            context,
            {
              contract,
              contractCall: contract.methods.score_a_free_delegation(),
            },
            {
              ...GENESIS_TRANSACTION,
              value: numberToHex(Number(MIN_GLMR_STAKING)),
            }
          ),
        ],
      });

      // TX should be included but fail
      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
      expect(receipt.status).to.eq(false);

      // Delegation shouldn't have passed
      const nominatorsAfter = await context.polkadotApi.query.parachainStaking.delegatorState(
        GENESIS_ACCOUNT
      );
      expect(nominatorsAfter.toHuman()).to.eq(null);

      // balance dif should only be tx fee, not MIN_GLMR_STAKING
      expect(
        Number(initialBalance) - Number(await context.web3.eth.getBalance(GENESIS_ACCOUNT)) <
          Number(MIN_GLMR_STAKING)
      ).to.eq(true);
    });
  }
);
