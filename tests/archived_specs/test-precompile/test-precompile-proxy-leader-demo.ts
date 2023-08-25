import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { ethers } from "ethers";
import Web3 from "web3";
import { Contract } from "web3-eth-contract";
import {
  ALITH_ADDRESS,
  baltathar,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  dorothy,
  DOROTHY_ADDRESS,
  DOROTHY_PRIVATE_KEY,
} from "../../util/accounts";
import { GLMR, MIN_GLMR_DELEGATOR } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { expectOk } from "../../util/expect";
import { instantFastTrack } from "../../util/governance";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
  TRANSACTION_TEMPLATE,
} from "../../util/transactions";

const LEADER_CONTRACT_JSON = getCompiled("ProxyLeaderDemo");
const LEADER_INTERFACE = new ethers.utils.Interface(LEADER_CONTRACT_JSON.contract.abi);

const setupPoolWithParticipants = async (context: DevTestContext) => {
  const { contract, rawTx } = await createContract(context, "ProxyLeaderDemo", {
    ...ALITH_TRANSACTION_TEMPLATE,
    gas: 5_000_000,
  });
  await expectOk(context.createBlock(rawTx));

  // Adds participants
  for (const [privateKey, from] of [
    [BALTATHAR_PRIVATE_KEY, BALTATHAR_ADDRESS],
    [CHARLETH_PRIVATE_KEY, CHARLETH_ADDRESS],
    [DOROTHY_PRIVATE_KEY, DOROTHY_ADDRESS],
  ]) {
    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey,
          from,
          to: contract.options.address,
          data: LEADER_INTERFACE.encodeFunctionData("joinPool", []),
          value: Web3.utils.toWei("1", "ether"),
        })
      )
    );
  }
  return contract;
};

describeDevMoonbeam("Proxy Leader Demo - Preparing Participation Pool", (context) => {
  let leaderContract: Contract;
  before("setup contract", async function () {
    leaderContract = await setupPoolWithParticipants(context);
  });

  it("should have a pool of 3 tokens", async function () {
    expect(await leaderContract.methods.pooledAmount().call()).to.equal((3n * GLMR).toString());
  });

  it("should have a balance of 3 tokens", async function () {
    const freeBalance = (
      await context.polkadotApi.query.system.account(leaderContract.options.address)
    ).data.free.toString();
    expect(freeBalance).to.equal((3n * GLMR).toString());
  });

  it("should have 3 participants", async function () {
    expect(await leaderContract.methods.getParticipants().call()).to.deep.equal([
      BALTATHAR_ADDRESS,
      CHARLETH_ADDRESS,
      DOROTHY_ADDRESS,
    ]);
  });
});

describeDevMoonbeam("Proxy Leader Demo - Start Voting", (context) => {
  let leaderContract: Contract;
  before("setup contract", async function () {
    leaderContract = await setupPoolWithParticipants(context);
  });

  it("should be able to start", async function () {
    expect(await leaderContract.methods.isVoting().call()).to.be.false;

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("startVoting", []),
      })
    );

    expectEVMResult(result.events, "Succeed");

    expect(await leaderContract.methods.isVoting().call()).to.be.true;
  });
});

describeDevMoonbeam("Proxy Leader Demo - Vote", (context) => {
  let leaderContract: Contract;
  before("setup contract and start voting", async function () {
    leaderContract = await setupPoolWithParticipants(context);

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("startVoting", []),
      })
    );

    expectEVMResult(result.events, "Succeed");
  });

  it("should not be able to vote if non-participant", async function () {
    expect(await leaderContract.methods.canVote(ALITH_ADDRESS).call()).to.be.false;

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("vote", [CHARLETH_ADDRESS, DOROTHY_ADDRESS]),
      })
    );
    expectEVMResult(result.events, "Revert");
  });

  it("should not be able to vote for non-participant", async function () {
    expect(await leaderContract.methods.canVote(BALTATHAR_ADDRESS).call()).to.be.true;

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("vote", [ALITH_ADDRESS, DOROTHY_ADDRESS]),
      })
    );
    expectEVMResult(result.events, "Revert");
  });

  it("should be able to vote for participant when participant", async function () {
    expect(await leaderContract.methods.canVote(BALTATHAR_ADDRESS).call()).to.be.true;

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("vote", [CHARLETH_ADDRESS, DOROTHY_ADDRESS]),
      })
    );
    expectEVMResult(result.events, "Succeed");

    expect(await leaderContract.methods.canVote(BALTATHAR_ADDRESS).call()).to.be.false;
  });
});

describeDevMoonbeam("Proxy Leader Demo - End Voting", (context) => {
  let leaderContract: Contract;
  before("setup contract and start voting", async function () {
    leaderContract = await setupPoolWithParticipants(context);

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("startVoting", []),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });

  // TODO: rework this test, contract cannot call proxy precompile
  it.skip("should be able to stop", async function () {
    expect(await leaderContract.methods.isVoting().call()).to.be.true;

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("endVoting", []),
      })
    );
    expectEVMResult(result.events, "Succeed");

    expect(await leaderContract.methods.isVoting().call()).to.be.false;
  });
});

// TODO: rework this test, contract cannot call proxy precompile
describeDevMoonbeam("Proxy Leader Demo - Winners", (context) => {
  let leaderContract: Contract;

  before("setup contract and voting results", async function () {
    this.skip();
    leaderContract = await setupPoolWithParticipants(context);

    // start voting
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("startVoting", []),
      })
    );
    expectEVMResult(result.events, "Succeed");

    // baltathar votes
    const { result: resultVote1 } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("vote", [CHARLETH_ADDRESS, DOROTHY_ADDRESS]),
      })
    );
    expectEVMResult(resultVote1.events, "Succeed");

    // charleth votes
    const { result: resultVote2 } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        from: CHARLETH_ADDRESS,
        privateKey: CHARLETH_PRIVATE_KEY,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("vote", [BALTATHAR_ADDRESS, BALTATHAR_ADDRESS]),
      })
    );
    expectEVMResult(resultVote2.events, "Succeed");

    // dorothy votes
    const { result: resultVote3 } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        from: DOROTHY_ADDRESS,
        privateKey: DOROTHY_PRIVATE_KEY,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("vote", [CHARLETH_ADDRESS, DOROTHY_ADDRESS]),
      })
    );
    expectEVMResult(resultVote3.events, "Succeed");

    // end voting
    const { result: resultEnd } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: leaderContract.options.address,
        data: LEADER_INTERFACE.encodeFunctionData("endVoting", []),
      })
    );
    expectEVMResult(resultEnd.events, "Succeed");

    // create a referendum
    await instantFastTrack(context, context.polkadotApi.tx.system.remark("foobar"), {
      votingPeriod: 10,
      delayPeriod: 0,
    });
  });

  it.skip("should proxy charleth as governor", async function () {
    expect(await leaderContract.methods.governor().call()).to.equal(CHARLETH_ADDRESS);
  });

  it.skip("should proxy dorothy as staker", async function () {
    expect(await leaderContract.methods.staker().call()).to.equal(DOROTHY_ADDRESS);
  });

  it.skip("should setup proxy types for contract address", async function () {
    const proxies = await context.polkadotApi.query.proxy.proxies(leaderContract.options.address);
    expect(proxies[0].toJSON()).to.deep.equal([
      {
        delegate: DOROTHY_ADDRESS,
        proxyType: "Staking",
        delay: 0,
      },
      {
        delegate: CHARLETH_ADDRESS,
        proxyType: "Governance",
        delay: 0,
      },
    ]);
  });

  it.skip("should not allow baltathar to stake via proxy", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(
          leaderContract.options.address,
          "Staking",
          context.polkadotApi.tx.parachainStaking.delegate(ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 0, 0)
        )
        .signAsync(baltathar)
    );
    expect(result.successful).to.be.false;
    expect(result.error.name).to.equal("NotProxy");
  });

  it.skip("should allow dorothy to stake via proxy", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(
          leaderContract.options.address,
          "Staking",
          context.polkadotApi.tx.parachainStaking.delegate(ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 0, 0)
        )
        .signAsync(dorothy)
    );

    const delegationEvents = result.events.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.Delegation.is(event.event)) {
        acc.push({
          delegator: event.event.data[0].toString(),
          candidate: event.event.data[2].toString(),
        });
      }
      return acc;
    }, []);

    expect(result.successful).to.be.true;
    expect(delegationEvents).to.deep.equal([
      {
        delegator: leaderContract.options.address,
        candidate: ALITH_ADDRESS,
      },
    ]);
  });

  it.skip("should not allow dorothy to vote via proxy", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(
          leaderContract.options.address,
          "Governance",
          context.polkadotApi.tx.democracy.vote(0, {
            Standard: { balance: 10n * GLMR, vote: { aye: true, conviction: 1 } },
          })
        )
        .signAsync(dorothy)
    );
    expect(result.successful).to.be.false;
    expect(result.error.name).to.equal("NotProxy");
  });

  it.skip("should allow charleth to vote via proxy", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(
          leaderContract.options.address,
          "Governance",
          context.polkadotApi.tx.democracy.vote(0, {
            Standard: { balance: 1n * GLMR, vote: { aye: true, conviction: 1 } },
          })
        )
        .signAsync(charleth)
    );

    const votedEvents = result.events.reduce((acc, event) => {
      if (context.polkadotApi.events.democracy.Voted.is(event.event)) {
        acc.push({
          voter: event.event.data[0].toString(),
          isAye: event.event.data[2].asStandard.vote.isAye,
        });
      }
      return acc;
    }, []);

    expect(result.successful).to.be.true;
    expect(votedEvents).to.deep.equal([
      {
        voter: leaderContract.options.address,
        isAye: true,
      },
    ]);
  });
});
