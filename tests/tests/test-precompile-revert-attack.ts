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

describeDevMoonbeam("Estimate Gas - Contract creation", (context) => {
  it("should return contract creation gas cost", async function () {
    const contract = await getCompiled("TestContract");
    expect(
      await context.web3.eth.estimateGas({
        from: GENESIS_ACCOUNT,
        data: contract.byteCode,
      })
    ).to.equal(149143);
  });
});
