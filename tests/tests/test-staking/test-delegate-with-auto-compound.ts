import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect, use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";
import { MIN_GLMR_STAKING, MIN_GLMR_DELEGATOR } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, baltathar, charleth, ethan } from "../../util/accounts";
import { expectOk } from "../../util/expect";

chaiUse(chaiAsPromised);

describeDevMoonbeam("Staking - Delegate With Auto-Compound - bond less than min", (context) => {
  it("should fail", async () => {
    const minDelegatorStk = context.polkadotApi.consts.parachainStaking.minDelegatorStk;
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegateWithAutoCompound(alith.address, minDelegatorStk.subn(10), 50, 0, 0, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("DelegatorBondBelowMin");
  });
});

describeDevMoonbeam("Staking - Delegate With Auto-Compound - candidate not exists", (context) => {
  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("CandidateDNE");
  });
});

describeDevMoonbeam(
  "Staking - Delegate With Auto-Compound - candidate not exists and self",
  (context) => {
    it("should fail", async () => {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(ethan.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
          .signAsync(ethan)
      );
      expect(block.result.successful).to.be.false;
      expect(block.result.error.name).to.equal("CandidateDNE");
    });
  }
);

describeDevMoonbeam("Staking - Delegate With Auto-Compound - already a candidate", (context) => {
  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
        .signAsync(alith)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("CandidateExists");
  });
});

describeDevMoonbeam("Staking - Delegate With Auto-Compound - already delegated", (context) => {
  before("should delegate", async () => {
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
          .signAsync(ethan)
      )
    );
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 1)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("AlreadyDelegatedCandidate");
  });
});

describeDevMoonbeam(
  "Staking - Delegate With Auto-Compound - wrong candidate delegation hint",
  (context) => {
    before("setup candidates alith & baltathar, and delegators ethan & charleth", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
            .signAsync(charleth),
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
            .signAsync(ethan),
        ])
      );
    });

    it("should fail", async () => {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 1)
          .signAsync(ethan)
      );
      expect(block.result.successful).to.be.false;
      expect(block.result.error.name).to.equal("TooLowCandidateDelegationCountToDelegate");
    });
  }
);

describeDevMoonbeam(
  "Staking - Delegate With Auto-Compound - wrong candidate auto-compounding delegation hint",
  (context) => {
    before("setup candidates alith & baltathar, and delegators ethan & charleth", async () => {
      await expectOk(
        context.createBlock([
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 50, 0, 0)
            .signAsync(charleth),
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
            .signAsync(ethan),
        ])
      );
    });

    it("should fail", async () => {
      const block = await context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 1, 0, 1)
          .signAsync(ethan)
      );
      expect(block.result.successful).to.be.false;
      expect(block.result.error.name).to.equal(
        "TooLowCandidateAutoCompoundingDelegationCountToDelegate"
      );
    });
  }
);

describeDevMoonbeam("Staking - Delegate With Auto-Compound - wrong delegation hint", (context) => {
  before("setup candidates alith & baltathar, and delegators ethan & charleth", async () => {
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, 1)
          .signAsync(baltathar),
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
          .signAsync(charleth),
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(baltathar.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
          .signAsync(ethan),
      ])
    );
  });

  it("should fail", async () => {
    const block = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 1, 0, 0)
        .signAsync(ethan)
    );
    expect(block.result.successful).to.be.false;
    expect(block.result.error.name).to.equal("TooLowDelegationCountToDelegate");
  });
});

describeDevMoonbeam("Staking - Delegate With Auto-Compound - 101%", (context) => {
  it("should fail", async () => {
    await expect(
      context.createBlock(
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 101, 0, 0, 0)
          .signAsync(ethan)
      )
    ).to.eventually.be.rejectedWith("Value is greater than allowed maximum!");
  });
});

describeDevMoonbeam("Staking - Delegate With Auto-Compound - valid request", (context) => {
  const numberToHex = (n: BigInt): string => `0x${n.toString(16).padStart(32, "0")}`;

  let events;
  before("should delegate", async () => {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
        .signAsync(ethan)
    );
    expect(result.successful).to.be.true;
    events = result.events;
  });

  it("should succeed", async () => {
    const delegatorState = await context.polkadotApi.query.parachainStaking.delegatorState(
      ethan.address
    );
    const autoCompoundConfig = (
      (await context.polkadotApi.query.parachainStaking.autoCompoundingDelegations(
        alith.address
      )) as any
    )
      .toJSON()
      .find((d) => d.delegator === ethan.address);
    const delegationEvents = events.reduce((acc, event) => {
      if (context.polkadotApi.events.parachainStaking.Delegation.is(event.event)) {
        acc.push({
          account: event.event.data[0].toString(),
          amount: event.event.data[1].toBigInt(),
          autoCompound: event.event.data[4].toJSON(),
        });
      }
      return acc;
    }, []);

    expect(delegationEvents).to.deep.equal([
      {
        account: ethan.address,
        amount: 1000000000000000000n,
        autoCompound: 50,
      },
    ]);
    expect(delegatorState.unwrap().toJSON()).to.deep.equal({
      delegations: [
        {
          amount: numberToHex(MIN_GLMR_DELEGATOR),
          owner: alith.address,
        },
      ],
      id: ethan.address,
      lessTotal: 0,
      status: { active: null },
      total: numberToHex(MIN_GLMR_DELEGATOR),
    });
    expect(autoCompoundConfig).to.deep.equal({
      delegator: ethan.address,
      value: 50,
    });
  });
});
