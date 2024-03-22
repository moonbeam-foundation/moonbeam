import "@moonbeam-network/api-augment";
import { describeSuite, expect, dispatchAsGeneralAdmin } from "@moonwall/cli";

describeSuite({
  id: "D014138",
  title: "Precompiles - xcm transactor",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "Moonbase: GeneralAdmin should be able to dispatch hrmpManage",
      test: async function () {
        // Just build the arguments. They dont matter that much though, since
        // we will not make sure it executes in the relay
        const transactWeights = context
          .polkadotJs()
          .createType("PalletXcmTransactorTransactWeights", {
            transactRequiredWeightAtMost: { refTime: 10000, proofSize: 10000 },
            overallWeight: { Limited: { refTime: 10000, proofSize: 10000 } },
          });

        const fee = context.polkadotJs().createType("PalletXcmTransactorCurrencyPayment", {
          currency: {
            AsMultiLocation: {
              V3: {
                parents: 1,
                interior: {
                  Here: null,
                },
              },
            },
          },
          feeAmount: 10000,
        });

        // send HrmpManage
        await dispatchAsGeneralAdmin(
          context,
          (context.polkadotJs().tx.xcmTransactor as any).hrmpManage(
            {
              Accept: {
                para_id: 2000,
              },
            },
            fee,
            transactWeights
          )
        );

        // Filter for HrmpManagementSent events
        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.xcmTransactor.HrmpManagementSent.is(event)
        );

        // It executed!
        expect(events).to.have.lengthOf(1);
      },
    });
  },
});
