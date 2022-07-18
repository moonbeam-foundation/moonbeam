import "@moonbeam-network/api-augment";

import { Result } from "@polkadot/types";
import { SpRuntimeDispatchError } from "@polkadot/types/lookup";
import { expect } from "chai";

import { alith } from "../../util/accounts";
import { execTechnicalCommitteeProposal } from "../../util/governance";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Maintenance Mode - Entering Maintenance Mode", (context) => {
  it("should succeed with Technical Committee", async function () {
    const { events, successful } = await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
    );
    expect(successful).to.be.true;
    expect(events[3].event.method).to.eq("EnteredMaintenanceMode");
    expect((await context.polkadotApi.query.maintenanceMode.maintenanceMode()).toHuman()).to.equal(
      true
    );
  });
});

describeDevMoonbeam("Maintenance Mode - Entering Maintenance Mode", (context) => {
  it("should fail with half the technical Committee", async function () {
    const { events } = await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode(),
      [alith],
      1
    );
    expect((events[1].event.data[1] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin).to.be
      .true;
  });
});

describeDevMoonbeam("Maintenance Mode - Entering Maintenance Mode", (context) => {
  it("should fail with sudo", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      )
    );
    expect((events[1].event.data[0] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin).to.be
      .true;
  });
});

describeDevMoonbeam("Maintenance Mode - Entering Maintenance Mode", (context) => {
  it("should fail without sudo", async function () {
    const {
      result: { error },
    } = await context.createBlock(context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode());
    expect(error.name).to.eq("BadOrigin");
  });
});

describeDevMoonbeam("Maintenance Mode - Resuming normal operation", (context) => {
  before("entering maintenance mode", async function () {
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
      )
    );
  });

  it("should fail with sudo", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.maintenanceMode.resumeNormalOperation()
      )
    );
    expect((events[1].event.data[0] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin).to.be
      .true;
  });
});

describeDevMoonbeam("Maintenance Mode - Resuming normal operation", (context) => {
  before("entering maintenance mode", async () => {
    await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
    );
  });

  it("should fail without sudo", async function () {
    // and try to turn it off
    const {
      result: { error },
    } = await context.createBlock(context.polkadotApi.tx.maintenanceMode.resumeNormalOperation());
    expect(error.name).to.eq("BadOrigin");
    expect((await context.polkadotApi.query.maintenanceMode.maintenanceMode()).isTrue).to.be.true;
  });
});

describeDevMoonbeam("Maintenance Mode - Resuming normal operation", (context) => {
  before("entering maintenance mode", async () => {
    await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
    );
  });

  it("should succeed with council", async function () {
    const { events, successful } = await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.maintenanceMode.resumeNormalOperation()
    );
    expect(successful).to.be.true;
    expect(events[3].event.method).to.eq("NormalOperationResumed");
    expect((await context.polkadotApi.query.maintenanceMode.maintenanceMode()).toHuman()).to.equal(
      false
    );
  });
});

describeDevMoonbeam("Maintenance Mode - Resuming normal operation", (context) => {
  before("entering maintenance mode", async () => {
    await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.maintenanceMode.enterMaintenanceMode()
    );
  });

  it("should fail with half the technical Committee", async function () {
    const { events } = await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.maintenanceMode.resumeNormalOperation(),
      [alith],
      1
    );
    expect((events[1].event.data[1] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin).to.be
      .true;
  });
});
