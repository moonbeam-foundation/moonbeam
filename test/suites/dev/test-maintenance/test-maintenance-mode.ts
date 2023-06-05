import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, execTechnicalCommitteeProposal, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { Result } from "@polkadot/types";
import { SpRuntimeDispatchError } from "@polkadot/types/lookup";

describeSuite({
  id: "D1904",
  title: "Maintenance Mode - General",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeEach(async () => {
      const { successful } = await execTechnicalCommitteeProposal(
        context,
        context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
      );
      expect(successful).to.be.true;
    });

    it({
      id: "T01",
      title: "should succeed with Technical Committee",
      test: async function () {
        const { events, successful } = await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
        );
        expect(successful).to.be.true;
        expect(events[3].event.method).to.eq("EnteredMaintenanceMode");
        expect(
          (await context.polkadotJs().query.maintenanceMode.maintenanceMode()).toHuman()
        ).to.equal(true);
      },
    });

    it({
      id: "T02",
      title: "should fail with half the technical Committee",
      test: async function () {
        const { events } = await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode(),
          [alith],
          1
        );
        expect((events[1].event.data[1] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin)
          .to.be.true;
      },
    });

    it({
      id: "T03",
      title: "should fail with sudo",
      test: async function () {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode())
        );
        expect(
          (result?.events[1].event.data[0] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin
        ).to.be.true;
      },
    });
    it({
      id: "T04",
      title: "should fail without sudo",
      test: async function () {
        const { result } = await context.createBlock(
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode(),
          { allowFailures: true }
        );
        expect(result!.error!.name).to.eq("BadOrigin");
      },
    });

    it({
      id: "T05",
      title: "resuming normal operation should fail with sudo",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode())
        );
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.maintenanceMode.resumeNormalOperation())
        );
        expect(
          (result?.events[1].event.data[0] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin
        ).to.be.true;
      },
    });

    it({
      id: "T06",
      title: "resuming normal operation should fail without sudo",
      test: async function () {
        await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
        );

        const { result } = await context.createBlock(
          context.polkadotJs().tx.maintenanceMode.resumeNormalOperation(),
          { allowFailures: true }
        );
        expect(result!.error!.name).to.eq("BadOrigin");
        expect((await context.polkadotJs().query.maintenanceMode.maintenanceMode()).isTrue).to.be
          .true;
      },
    });

    it({
      id: "T07",
      title: "resuming normal operation should succeed with council",
      test: async function () {
        await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
        );

        const { events, successful } = await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
        );
        expect(successful).to.be.true;
        expect(events[3].event.method).to.eq("NormalOperationResumed");
        expect(
          (await context.polkadotJs().query.maintenanceMode.maintenanceMode()).toHuman()
        ).to.equal(false);
      },
    });

    it({
      id: "T08",
      title: "resuming normal operation should fail with half the technical Committee",
      test: async function () {
        await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
        );

        const { events } = await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.resumeNormalOperation(),
          [alith],
          1
        );
        expect((events[1].event.data[1] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin)
          .to.be.true;
      },
    });
  },
});
