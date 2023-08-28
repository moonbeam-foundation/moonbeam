import "@moonbeam-network/api-augment";
import { nToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import Web3 from "web3";
import { Contract } from "web3-eth-contract";
import {
  baltathar,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  dorothy,
  DOROTHY_ADDRESS,
  DOROTHY_PRIVATE_KEY,
  ethan,
  ETHAN_ADDRESS,
  ETHAN_PRIVATE_KEY,
} from "../../util/accounts";
import { GLMR, MIN_GLMR_STAKING } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectOk } from "../../util/expect";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
  TRANSACTION_TEMPLATE,
} from "../../util/transactions";

const PROXY_STAKING_CONTRACT_JSON = getCompiled("ProxyCallStakingDemo");
const PROXY_STAKING_INTERFACE = new ethers.utils.Interface(
  PROXY_STAKING_CONTRACT_JSON.contract.abi
);

async function setupWithParticipants(context: DevTestContext) {
  const { contract, rawTx } = await createContract(context, "ProxyCallStakingDemo", {
    ...ALITH_TRANSACTION_TEMPLATE,
    gas: 5_000_000,
    value: Web3.utils.toWei("5", "ether"),
  });
  await expectOk(context.createBlock(rawTx));

  // Add participants
  for (const { account, privateKey, address: from } of [
    {
      account: baltathar,
      privateKey: BALTATHAR_PRIVATE_KEY,
      address: BALTATHAR_ADDRESS,
    },
    {
      account: charleth,
      privateKey: CHARLETH_PRIVATE_KEY,
      address: CHARLETH_ADDRESS,
    },
  ]) {
    // pre-condition provide staking proxy to contract
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.proxy
          .addProxy(contract.options.address, "Staking", 0)
          .signAsync(account)
      )
    );

    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey,
          from,
          gas: 5_000_000,
          to: contract.options.address,
          data: PROXY_STAKING_INTERFACE.encodeFunctionData("join", [0]),
        })
      )
    );
  }
  return contract;
}

describeDevMoonbeam("Proxy Call Staking Demo - Participants", (context) => {
  let demoContract: Contract;
  before("setup contract", async function () {
    demoContract = await setupWithParticipants(context);
  });

  it("should have 2 participants", async function () {
    expect(await demoContract.methods.isParticipant(BALTATHAR_ADDRESS).call()).to.be.true;
    expect(await demoContract.methods.isParticipant(CHARLETH_ADDRESS).call()).to.be.true;
  });
});

describeDevMoonbeam("Proxy Call Staking Demo - Register Candidate", (context) => {
  let demoContract: Contract;
  before("setup contract", async function () {
    demoContract = await setupWithParticipants(context);
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey: ETHAN_PRIVATE_KEY,
          from: ETHAN_ADDRESS,
          gas: 5_000_000,
          to: demoContract.options.address,
          data: PROXY_STAKING_INTERFACE.encodeFunctionData("registerCandidate", [0]),
        })
      )
    );
  });

  it("should have 2 participants", async function () {
    expect(await demoContract.methods.isParticipant(BALTATHAR_ADDRESS).call()).to.be.true;
    expect(await demoContract.methods.isParticipant(CHARLETH_ADDRESS).call()).to.be.true;
  });

  it("should have 1 candidate", async function () {
    expect(await demoContract.methods.isCandidate(ETHAN_ADDRESS).call()).to.be.true;
  });

  it("should have delegated all participants to ethan", async function () {
    const delegations = await context.polkadotApi.query.parachainStaking.topDelegations(
      ETHAN_ADDRESS
    );
    expect(delegations.toJSON()).to.deep.equal({
      delegations: [
        {
          owner: BALTATHAR_ADDRESS,
          amount: nToHex(1n * GLMR, { bitLength: 128 }),
        },
        {
          owner: CHARLETH_ADDRESS,
          amount: nToHex(1n * GLMR, { bitLength: 128 }),
        },
      ],
      total: nToHex(2n * GLMR, { bitLength: 128 }),
    });
  });
});

describeDevMoonbeam("Proxy Call Staking Demo - New Participant", (context) => {
  let demoContract: Contract;
  before("setup contract", async function () {
    demoContract = await setupWithParticipants(context);
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey: ETHAN_PRIVATE_KEY,
          from: ETHAN_ADDRESS,
          gas: 5_000_000,
          to: demoContract.options.address,
          data: PROXY_STAKING_INTERFACE.encodeFunctionData("registerCandidate", [0]),
        })
      )
    );

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.proxy
          .addProxy(demoContract.options.address, "Staking", 0)
          .signAsync(dorothy)
      )
    );
    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey: DOROTHY_PRIVATE_KEY,
          from: DOROTHY_ADDRESS,
          gas: 5_000_000,
          to: demoContract.options.address,
          data: PROXY_STAKING_INTERFACE.encodeFunctionData("join", [0]),
        })
      )
    );
  });

  it("should have 3 participants", async function () {
    expect(await demoContract.methods.isParticipant(BALTATHAR_ADDRESS).call()).to.be.true;
    expect(await demoContract.methods.isParticipant(CHARLETH_ADDRESS).call()).to.be.true;
    expect(await demoContract.methods.isParticipant(DOROTHY_ADDRESS).call()).to.be.true;
  });

  it("should have 1 candidate", async function () {
    expect(await demoContract.methods.isCandidate(ETHAN_ADDRESS).call()).to.be.true;
  });

  it("should have delegated all participants including dorothy to ethan", async function () {
    const delegations = await context.polkadotApi.query.parachainStaking.topDelegations(
      ETHAN_ADDRESS
    );
    expect(delegations.toJSON()).to.deep.equal({
      delegations: [
        {
          owner: BALTATHAR_ADDRESS,
          amount: nToHex(1n * GLMR, { bitLength: 128 }),
        },
        {
          owner: CHARLETH_ADDRESS,
          amount: nToHex(1n * GLMR, { bitLength: 128 }),
        },
        {
          owner: DOROTHY_ADDRESS,
          amount: nToHex(1n * GLMR, { bitLength: 128 }),
        },
      ],
      total: nToHex(3n * GLMR, { bitLength: 128 }),
    });
  });
});

describeDevMoonbeam("Proxy Call Staking Demo - Leave Participant", (context) => {
  let demoContract: Contract;
  before("setup contract", async function () {
    demoContract = await setupWithParticipants(context);
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey: ETHAN_PRIVATE_KEY,
          from: ETHAN_ADDRESS,
          gas: 5_000_000,
          to: demoContract.options.address,
          data: PROXY_STAKING_INTERFACE.encodeFunctionData("registerCandidate", [0]),
        })
      )
    );
    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey: CHARLETH_PRIVATE_KEY,
          from: CHARLETH_ADDRESS,
          gas: 5_000_000,
          to: demoContract.options.address,
          data: PROXY_STAKING_INTERFACE.encodeFunctionData("leave"),
        })
      )
    );
  });

  it("should have 1 participant", async function () {
    expect(await demoContract.methods.isParticipant(BALTATHAR_ADDRESS).call()).to.be.true;
    expect(await demoContract.methods.isParticipant(CHARLETH_ADDRESS).call()).to.be.false;
  });

  it("should have 1 candidate", async function () {
    expect(await demoContract.methods.isCandidate(ETHAN_ADDRESS).call()).to.be.true;
  });

  it("should have scheduled leave from charleth to ethan", async function () {
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ETHAN_ADDRESS);
    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();
    const roundDelay = context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber();
    expect(delegationRequests.toJSON()).to.deep.equal([
      {
        delegator: CHARLETH_ADDRESS,
        whenExecutable: currentRound + roundDelay,
        action: {
          revoke: nToHex(1n * GLMR, { bitLength: 128 }),
        },
      },
    ]);
  });
});

describeDevMoonbeam("Proxy Call Staking Demo - Unregister Candidate", (context) => {
  let demoContract: Contract;
  before("setup contract", async function () {
    demoContract = await setupWithParticipants(context);
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1).signAsync(ethan)
      )
    );
    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey: ETHAN_PRIVATE_KEY,
          from: ETHAN_ADDRESS,
          gas: 5_000_000,
          to: demoContract.options.address,
          data: PROXY_STAKING_INTERFACE.encodeFunctionData("registerCandidate", [0]),
        })
      )
    );
    expect(await demoContract.methods.isCandidate(ETHAN_ADDRESS).call()).to.be.true;
    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey: ETHAN_PRIVATE_KEY,
          from: ETHAN_ADDRESS,
          gas: 5_000_000,
          to: demoContract.options.address,
          data: PROXY_STAKING_INTERFACE.encodeFunctionData("unregisterCandidate"),
        })
      )
    );
  });

  it("should have 2 participants", async function () {
    expect(await demoContract.methods.isParticipant(BALTATHAR_ADDRESS).call()).to.be.true;
    expect(await demoContract.methods.isParticipant(CHARLETH_ADDRESS).call()).to.be.true;
  });

  it("should have 0 candidates", async function () {
    expect(await demoContract.methods.isCandidate(ETHAN_ADDRESS).call()).to.be.false;
  });

  it("should have scheduled leave from baltathar and charleth to ethan", async function () {
    const delegationRequests =
      await context.polkadotApi.query.parachainStaking.delegationScheduledRequests(ETHAN_ADDRESS);
    const currentRound = (
      await context.polkadotApi.query.parachainStaking.round()
    ).current.toNumber();
    const roundDelay = context.polkadotApi.consts.parachainStaking.revokeDelegationDelay.toNumber();
    expect(delegationRequests.toJSON()).to.deep.equal([
      {
        delegator: BALTATHAR_ADDRESS,
        whenExecutable: currentRound + roundDelay,
        action: {
          revoke: nToHex(1n * GLMR, { bitLength: 128 }),
        },
      },
      {
        delegator: CHARLETH_ADDRESS,
        whenExecutable: currentRound + roundDelay,
        action: {
          revoke: nToHex(1n * GLMR, { bitLength: 128 }),
        },
      },
    ]);
  });
});
