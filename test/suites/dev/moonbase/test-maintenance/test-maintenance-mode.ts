import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect, execOpenTechCommitteeProposal } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import type { Result } from "@polkadot/types";
import type { SpRuntimeDispatchError } from "@polkadot/types/lookup";

describeSuite({
  id: "D012004",
  title: "Maintenance Mode - General",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeEach(async () => {
      const { successful } = await execOpenTechCommitteeProposal(
        context,
        context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
      );

      expect(successful).to.be.true;
    });

    it({
      id: "T01",
      title: "should succeed with Technical Committee",
      test: async () => {
        const { successful, events } = await execOpenTechCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
        );
        expect(successful).to.be.true;
        expect(events[3].event.method).to.eq("EnteredMaintenanceMode");
        expect(
          (await context.polkadotJs().query.maintenanceMode.maintenanceMode()).isTrue
        ).to.equal(true);
      },
    });

    it({
      id: "T02",
      title: "should fail with half the technical Committee",
      test: async () => {
        const { events } = await execOpenTechCommitteeProposal(
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
      test: async () => {
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
      test: async () => {
        const { result } = await context.createBlock(
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode(),
          { allowFailures: true }
        );
        expect(result?.error?.name).to.eq("BadOrigin");
      },
    });

    it({
      id: "T05",
      title: "resuming normal operation should fail with sudo",
      test: async () => {
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
      test: async () => {
        await execOpenTechCommitteeProposal(
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
      test: async () => {
        await execOpenTechCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
        );

        const { successful, events } = await execOpenTechCommitteeProposal(
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
      test: async () => {
        await execOpenTechCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
        );

        const { events } = await execOpenTechCommitteeProposal(
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
