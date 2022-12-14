import "@moonbeam-network/api-augment";

import { stringToU8a } from "@polkadot/util";
import { blake2AsHex } from "@polkadot/util-crypto";
import { expect } from "chai";
import Web3 from "web3";
import { Account } from "web3-core";

import {
  alith,
  ALITH_GENESIS_FREE_BALANCE,
  ALITH_GENESIS_TRANSFERABLE_BALANCE,
  baltathar,
  generateKeyringPair,
  goliath,
} from "../../util/accounts";
import { verifyLatestBlockFees } from "../../util/block";
import { DEFAULT_GENESIS_BALANCE, GLMR, VOTE_AMOUNT } from "../../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import { instantFastTrack, notePreimage } from "../../util/governance";

// Relay addresses for crowdloan tests
export const RELAYCHAIN_ARBITRARY_ADDRESS_1: string =
  "0x1111111111111111111111111111111111111111111111111111111111111111";
export const RELAYCHAIN_ARBITRARY_ADDRESS_2: string =
  "0x2222222222222222222222222222222222222222222222222222222222222222";

// 5 blocks per minute, 4 weeks
export const VESTING_PERIOD = 201600n;

async function calculate_vested_amount(
  totalReward: bigint,
  initialPayment: bigint,
  numberOfBlocks: bigint
) {
  const amountToVest = totalReward - initialPayment;
  // On average a parachain only gets a candidate into every other relay chain block.
  // In the dev service, where the relay block number is mocked, we get exactly two relay blocks.
  const elapsedRelayBlocks = numberOfBlocks * 2n;
  const amountForBlocks = (amountToVest * elapsedRelayBlocks) / VESTING_PERIOD;
  const shouldHaveVested = initialPayment + amountForBlocks;
  return shouldHaveVested;
}

// Return the unwrapped accountsPayable or null otherwise
export const getAccountPayable = async (
  context: DevTestContext,
  address: string
): Promise<{
  totalReward: any;
  claimedReward: any;
  contributedRelayAddresses: any;
} | null> => {
  const accountsPayable = await context.polkadotApi.query.crowdloanRewards.accountsPayable(address);
  return accountsPayable.unwrapOr(null);
};

describeDevMoonbeam("Crowdloan", (context) => {
  it("should check initial state", async function () {
    // check that genesis is not registered
    const isPayable = await getAccountPayable(context, alith.address);
    expect(isPayable).to.equal(null);
  });

  it("should be able to register the genesis account for reward", async function () {
    // should be able to register the genesis account for reward
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
        ])
      )
    );

    await verifyLatestBlockFees(context, 3_000_000n);

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    expect((await getAccountPayable(context, alith.address)).totalReward.toBigInt()).to.equal(
      3_000_000n * GLMR
    );
    const isInitialized = await context.polkadotApi.query.crowdloanRewards.initialized();
    expect(isInitialized.toHuman()).to.be.true;
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  it("should be able to make a first claim", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const rewardInfo = await getAccountPayable(context, alith.address);
    const claimed = await calculate_vested_amount(
      rewardInfo.totalReward.toBigInt(),
      rewardInfo.claimedReward.toBigInt(),
      2n
    );
    // construct a transaction
    const transfer = context.polkadotApi.tx.crowdloanRewards.claim();
    await transfer.signAndSend(alith);
    const details = await context.polkadotApi.rpc.payment.queryFeeDetails(transfer.toHex());
    const claimFee =
      details.inclusionFee.unwrap().baseFee.toBigInt() +
      details.inclusionFee.unwrap().lenFee.toBigInt() +
      details.inclusionFee.unwrap().adjustedWeightFee.toBigInt();

    await context.createBlock();

    expect((await getAccountPayable(context, alith.address)).claimedReward.toBigInt()).to.equal(
      claimed
    );

    // check balances
    expect(
      BigInt(await context.web3.eth.getBalance(alith.address)) - ALITH_GENESIS_TRANSFERABLE_BALANCE
    ).to.equal(claimed - claimFee); // reduce the claim fee part;
    const account = await context.polkadotApi.query.system.account(alith.address);
    expect(account.data.free.toBigInt() - ALITH_GENESIS_FREE_BALANCE).to.equal(claimed - claimFee);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  it("should show me the money after 5 blocks, after first claim was called", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const rewardInfo = await getAccountPayable(context, alith.address);

    // construct a transaction
    const transfer = context.polkadotApi.tx.crowdloanRewards.claim();
    await transfer.signAndSend(alith);

    await context.createBlock();

    // should show me the money after 5 blocks
    await context.createBlock();
    await context.createBlock();

    const claimed = await calculate_vested_amount(
      rewardInfo.totalReward.toBigInt(),
      rewardInfo.claimedReward.toBigInt(),
      5n
    );

    await context.polkadotApi.tx.crowdloanRewards.claim().signAndSend(alith);
    await context.createBlock();
    const isPayable4 = await getAccountPayable(context, alith.address);
    expect(isPayable4.claimedReward.toBigInt()).to.equal(claimed);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  it("should make first claim 5 blocks after initialization called", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const rewardInfo = await getAccountPayable(context, alith.address);
    await context.createBlock();

    // should show me the money after 5 blocks
    await context.createBlock();
    await context.createBlock();
    const claimed = await calculate_vested_amount(
      rewardInfo.totalReward.toBigInt(),
      rewardInfo.claimedReward.toBigInt(),
      5n
    );

    await context.polkadotApi.tx.crowdloanRewards.claim().signAndSend(alith);
    await context.createBlock();

    const isPayable4 = await getAccountPayable(context, alith.address);
    expect(isPayable4.claimedReward.toBigInt()).to.equal(claimed);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  it("should not be able to call initializeRewardVec another time", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    // should not be able to call initializeRewardVec another time
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, baltathar.address, 1000n * GLMR],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();
    expect(await getAccountPayable(context, baltathar.address)).to.equal(null);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  it("should be able to register the genesis account - with small amount", async function () {
    // initializeRewardVec
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    expect((await getAccountPayable(context, alith.address)).totalReward.toBigInt()).to.equal(
      3_000_000n * GLMR
    );

    const rewardInfo = await getAccountPayable(context, alith.address);
    const claimed = await calculate_vested_amount(
      rewardInfo.totalReward.toBigInt(),
      rewardInfo.claimedReward.toBigInt(),
      2n
    );
    // claim
    await context.polkadotApi.tx.crowdloanRewards.claim().signAndSend(alith);
    await context.createBlock();

    const isPayable4 = await getAccountPayable(context, alith.address);
    expect(isPayable4.claimedReward.toBigInt()).to.equal(claimed);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  let numberOfAccounts: number;
  let largInput: [string, string, bigint][];

  before("Setup genesis account for substrate", async () => {
    numberOfAccounts = context.polkadotApi.consts.crowdloanRewards.maxInitContributors.toNumber();
  });

  it("should be able to register many accounts : " + numberOfAccounts, async function () {
    // should create a bunch of test eth accounts
    this.timeout(30000);
    const web3 = new Web3();
    // We need to make sure the rewards match the account funds. 3M GLMR/ number of accounts
    const accounts = new Array(numberOfAccounts).fill(0).map((_) => web3.eth.accounts.create());
    largInput = accounts.map((acc: Account, i: number) => {
      return [
        acc.address + "111111111111111111111111",
        acc.address,
        (3_000_000n * GLMR) / BigInt(numberOfAccounts),
      ];
    });
    expect(largInput.length).to.eq(numberOfAccounts);
    expect(largInput[0][1] !== largInput[numberOfAccounts - 1][1]).to.eq(true);

    // should be able to register many accounts
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(largInput))
      .signAndSend(alith);

    await context.createBlock();

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const rewardPerContributor = (3_000_000n * GLMR) / BigInt(numberOfAccounts);
    await Promise.all(
      largInput.map(async (input) => {
        expect((await getAccountPayable(context, input[1])).totalReward.toBigInt()).to.equal(
          rewardPerContributor
        );
      })
    );
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  let numberOfAccounts: number;
  let largInput: [string, string, bigint][];

  before("Setup genesis account for substrate", async () => {
    // We shouldnt be able to register as many accounts unless we do it in batches
    numberOfAccounts = Number(
      await context.polkadotApi.consts.crowdloanRewards.maxInitContributors
    );
  });

  it("should be able to register many accounts - batch : " + numberOfAccounts, async function () {
    // should create a bunch of test eth accounts
    this.timeout(20000);
    const web3 = new Web3();
    const accounts = new Array(numberOfAccounts).fill(0).map((_, i) => web3.eth.accounts.create());
    largInput = accounts.map((acc: Account, i: number) => {
      return [
        acc.address + "111111111111111111111111",
        acc.address,
        (3_000_000n * GLMR) / BigInt(numberOfAccounts),
      ];
    });
    expect(largInput.length).to.eq(numberOfAccounts);
    expect(largInput[0][1] !== largInput[numberOfAccounts - 1][1]).to.eq(true);

    // should be able to register many accounts
    await context.createBlock(
      context.polkadotApi.tx.utility
        .batch([
          await context.polkadotApi.tx.sudo.sudo(
            context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
              largInput.slice(0, Math.floor(numberOfAccounts / 3))
            )
          ),
          await context.polkadotApi.tx.sudo.sudo(
            context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
              largInput.slice(
                Math.floor(numberOfAccounts / 3),
                Math.floor((numberOfAccounts * 2) / 3)
              )
            )
          ),
          await context.polkadotApi.tx.sudo.sudo(
            context.polkadotApi.tx.crowdloanRewards.initializeRewardVec(
              largInput.slice(Math.floor((numberOfAccounts * 2) / 3), numberOfAccounts)
            )
          ),
        ])
        .signAsync(alith)
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    await Promise.all(
      largInput.map(async (input) => {
        expect((await getAccountPayable(context, input[1])).totalReward.toBigInt()).to.equal(
          (3_000_000n * GLMR) / BigInt(numberOfAccounts)
        );
      })
    );
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  before("Setup genesis account for substrate", async () => {});

  it("should be able to initialize through democracy", async function () {
    const calls = [];
    // We are gonna put the initialization and completion in a batch_all utility call
    calls.push(
      context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
        [RELAYCHAIN_ARBITRARY_ADDRESS_1, goliath.address, 1_500_000n * GLMR],
        [RELAYCHAIN_ARBITRARY_ADDRESS_2, null, 1_500_000n * GLMR],
      ])
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();
    calls.push(
      context.polkadotApi.tx.crowdloanRewards.completeInitialization(
        initBlock.toBigInt() + VESTING_PERIOD
      )
    );

    // Here we build the utility call
    const proposal = context.polkadotApi.tx.utility.batchAll(calls);

    const encodedHash = await instantFastTrack(context, proposal);

    // vote
    await context.createBlock(
      context.polkadotApi.tx.democracy.vote(0, {
        Standard: { balance: VOTE_AMOUNT, vote: { aye: true, conviction: 1 } },
      })
    );

    // referendumInfoOf
    const referendumInfoOf = (
      await context.polkadotApi.query.democracy.referendumInfoOf(0)
    ).unwrap() as any;
    const onGoing = referendumInfoOf.asOngoing;

    const blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    for (let i = 0; i < onGoing.end.toNumber() - blockNumber + 1; i++) {
      await context.createBlock();
    }

    const isInitialized = await context.polkadotApi.query.crowdloanRewards.initialized();

    expect(isInitialized.toHuman()).to.be.true;

    // Get reward info of associated
    const reward_info_associated = await getAccountPayable(context, goliath.address);

    // Get reward info of unassociated
    const reward_info_unassociated = (
      await context.polkadotApi.query.crowdloanRewards.unassociatedContributions(
        RELAYCHAIN_ARBITRARY_ADDRESS_2
      )
    ).unwrap();

    // Check payments
    expect(reward_info_associated.totalReward.toBigInt()).to.equal(1_500_000n * GLMR);
    expect(reward_info_associated.claimedReward.toBigInt()).to.equal(450_000n * GLMR);
    expect(reward_info_unassociated.totalReward.toBigInt()).to.equal(1_500_000n * GLMR);
    expect(reward_info_unassociated.claimedReward.toBigInt()).to.equal(0n);

    // check balances
    const account = await context.polkadotApi.query.system.account(goliath.address);
    expect(account.data.free.toBigInt() - DEFAULT_GENESIS_BALANCE).to.equal(
      reward_info_associated.claimedReward.toBigInt()
    );
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  before("Setup genesis account for substrate", async () => {});

  it("should be able to burn the dust", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 1_500_000n * GLMR],
          [RELAYCHAIN_ARBITRARY_ADDRESS_2, null, 1_499_999_999_999_999_999_999_999n],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();
    const previousIssuance = await context.polkadotApi.query.balances.totalIssuance();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const issuance = await context.polkadotApi.query.balances.totalIssuance();

    const isInitialized = await context.polkadotApi.query.crowdloanRewards.initialized();

    expect(isInitialized.toJSON()).to.be.true;

    // We should have burnt 1
    expect(issuance.toBigInt()).to.eq(previousIssuance.toBigInt() - 1n);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  const relayAccount = generateKeyringPair("ed25519");
  const toAssociateAccount = generateKeyringPair();

  it("should be able to associate identity", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 1_500_000n * GLMR],
          [relayAccount.addressRaw, null, 1_500_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const isInitialized = await context.polkadotApi.query.crowdloanRewards.initialized();

    expect(isInitialized.toJSON()).to.be.true;

    // relayAccount should be in the unassociated contributions
    expect(
      (
        await context.polkadotApi.query.crowdloanRewards.unassociatedContributions(
          relayAccount.addressRaw
        )
      )
        .unwrap()
        .totalReward.toBigInt()
    ).to.equal(1_500_000n * GLMR);

    // toAssociateAccount should not be in accounts payable
    expect(await getAccountPayable(context, toAssociateAccount.address)).to.be.null;

    const message = new Uint8Array([
      ...stringToU8a("<Bytes>"),
      ...stringToU8a("moonbase-"),
      ...toAssociateAccount.addressRaw,
      ...stringToU8a("</Bytes>"),
    ]);
    // Construct the signature
    const signature = { Ed25519: relayAccount.sign(message) };

    // Associate the identity
    await context.createBlock(
      context.polkadotApi.tx.crowdloanRewards.associateNativeIdentity(
        toAssociateAccount.address,
        relayAccount.addressRaw,
        signature
      )
    );

    // relayAccount should no longer be in the unassociated contributions
    expect(
      (
        await context.polkadotApi.query.crowdloanRewards.unassociatedContributions(
          relayAccount.addressRaw
        )
      ).toJSON()
    ).to.be.null;

    // toAssociateAccount should now be in accounts payable
    const rewardInfo = await getAccountPayable(context, toAssociateAccount.address);

    expect(rewardInfo.totalReward.toBigInt()).to.equal(1_500_000n * GLMR);
    expect(rewardInfo.claimedReward.toBigInt()).to.equal(450_000n * GLMR);

    // three blocks elapsed
    const claimed = await calculate_vested_amount(
      rewardInfo.totalReward.toBigInt(),
      rewardInfo.claimedReward.toBigInt(),
      3n
    );

    await context.createBlock(
      context.polkadotApi.tx.crowdloanRewards.claim().signAsync(toAssociateAccount)
    );

    // Claimed amount should match
    expect(
      (await getAccountPayable(context, toAssociateAccount.address)).claimedReward.toBigInt()
    ).to.equal(claimed);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  const relayAccount = generateKeyringPair("ed25519");
  const relayAccount2 = generateKeyringPair("ed25519");
  const firstAccount = generateKeyringPair();
  const toAssociateAccount = generateKeyringPair();

  it("should be able to change reward address with relay keys", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [relayAccount.addressRaw, firstAccount.address, 1_500_000n * GLMR],
          [relayAccount2.addressRaw, firstAccount.address, 1_500_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const isInitialized = await context.polkadotApi.query.crowdloanRewards.initialized();

    expect(isInitialized.toJSON()).to.be.true;

    // toAssociateAccount should not be in accounts payable
    expect(await getAccountPayable(context, toAssociateAccount.address)).to.be.null;

    const message = new Uint8Array([
      ...stringToU8a("<Bytes>"),
      ...stringToU8a("moonbase-"),
      ...toAssociateAccount.addressRaw,
      ...firstAccount.addressRaw,
      ...stringToU8a("</Bytes>"),
    ]);

    // Construct the signatures
    const signature1 = { Ed25519: relayAccount.sign(message) };
    const signature2 = { Ed25519: relayAccount2.sign(message) };

    const proofs = [
      [relayAccount.addressRaw, signature1],
      [relayAccount2.addressRaw, signature2],
    ] as any[];
    // Associate the identity
    await context.createBlock(
      context.polkadotApi.tx.crowdloanRewards.changeAssociationWithRelayKeys(
        toAssociateAccount.address,
        firstAccount.address,
        proofs
      )
    );

    // toAssociateAccount should now be in accounts payable
    const rewardInfo = await getAccountPayable(context, toAssociateAccount.address);

    expect(rewardInfo.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  const toUpdateAccount = generateKeyringPair();

  it("should be able to update reward address", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const isInitialized = await context.polkadotApi.query.crowdloanRewards.initialized();

    expect(isInitialized.toJSON()).to.be.true;

    // GENESIS_ACCOUNT should be in accounts pauable
    const rewardInfo = await getAccountPayable(context, alith.address);
    expect(rewardInfo.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
    expect(rewardInfo.claimedReward.toBigInt()).to.equal(900_000n * GLMR);

    // three blocks elapsed
    const claimed = await calculate_vested_amount(
      rewardInfo.totalReward.toBigInt(),
      rewardInfo.claimedReward.toBigInt(),
      2n
    );

    await context.createBlock(context.polkadotApi.tx.crowdloanRewards.claim());

    // Claimed amount should match
    const claimedRewards = (await getAccountPayable(context, alith.address)).claimedReward;
    expect(claimedRewards.toBigInt()).to.equal(claimed);

    // Let's update the reward address
    await context.createBlock(
      context.polkadotApi.tx.crowdloanRewards.updateRewardAddress(toUpdateAccount.address)
    );

    // GENESIS_ACCOUNT should no longer be in accounts payable
    expect(await getAccountPayable(context, alith.address)).to.be.null;

    // toUpdateAccount should be in accounts paYable
    const updateRewardInfo = await getAccountPayable(context, toUpdateAccount.address);
    expect(updateRewardInfo.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
    expect(updateRewardInfo.claimedReward.toBigInt()).to.equal(claimed);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  const proxy = baltathar;

  it("should be able to call crowdloan rewards with non-transfer proxy", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const isInitialized = await context.polkadotApi.query.crowdloanRewards.initialized();
    expect(isInitialized.toJSON()).to.be.true;

    // GENESIS_ACCOUNT should be in accounts pauable
    const rewardInfo = await getAccountPayable(context, alith.address);
    expect(rewardInfo.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
    expect(rewardInfo.claimedReward.toBigInt()).to.equal(900_000n * GLMR);

    // CreateProxy
    await context.createBlock(
      context.polkadotApi.tx.proxy.addProxy(proxy.address, "NonTransfer", 0)
    );

    // three blocks elapsed
    const claimed = await calculate_vested_amount(
      rewardInfo.totalReward.toBigInt(),
      rewardInfo.claimedReward.toBigInt(),
      3n
    );

    // Claim with proxy

    await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(alith.address, null, context.polkadotApi.tx.crowdloanRewards.claim())
        .signAsync(proxy)
    );

    // Claimed amount should match
    const claimedRewards = (await getAccountPayable(context, alith.address)).claimedReward;
    expect(claimedRewards.toBigInt()).to.equal(claimed);
  });
});

describeDevMoonbeam("Crowdloan", (context) => {
  const proxy = baltathar;

  it("should NOT be able to call non-claim extrinsic with non-transfer proxy", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.initializeRewardVec([
          [RELAYCHAIN_ARBITRARY_ADDRESS_1, alith.address, 3_000_000n * GLMR],
        ])
      )
    );

    const initBlock = await context.polkadotApi.query.crowdloanRewards.initRelayBlock();

    // Complete initialization
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.crowdloanRewards.completeInitialization(
          initBlock.toBigInt() + VESTING_PERIOD
        )
      )
    );

    const isInitialized = await context.polkadotApi.query.crowdloanRewards.initialized();
    expect(isInitialized.toJSON()).to.be.true;

    // GENESIS_ACCOUNT should be in accounts pauable
    const rewardInfo = await getAccountPayable(context, alith.address);
    expect(rewardInfo.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
    expect(rewardInfo.claimedReward.toBigInt()).to.equal(900_000n * GLMR);

    // CreateProxy
    await context.createBlock(
      context.polkadotApi.tx.proxy.addProxy(proxy.address, "NonTransfer", 0)
    );

    // Should not be ablte to do this
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(
          alith.address,
          null,
          context.polkadotApi.tx.crowdloanRewards.updateRewardAddress(proxy.address)
        )
        .signAsync(proxy)
    );
    expect(events[1].event.method).to.eq("ProxyExecuted");
    expect(events[1].event.data[0].toString()).to.be.eq(
      `{"err":{"module":{"index":0,"error":"0x05000000"}}}`
    );

    // Genesis account still has the money
    const updatedRewardInfo = await getAccountPayable(context, alith.address);
    expect(updatedRewardInfo.totalReward.toBigInt()).to.equal(3_000_000n * GLMR);
    expect(updatedRewardInfo.claimedReward.toBigInt()).to.equal(900_000n * GLMR);
  });
});
