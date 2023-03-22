import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { dispatchAsGeneralAdmin } from "../../util/governance";

describeDevMoonbeam("Precompiles - xcm transactor", (context) => {
  before("Setup genesis account and relay accounts", async () => {
    // register index 0 for Alith
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.xcmTransactor.register(alith.address, 0)
      )
    );
  });

  it("allows to retrieve index through precompiles", async function () {
    const resp = await context.polkadotApi.query.xcmTransactor.indexToAccount(0);
    expect(resp.toString()).to.eq(alith.address);
  });
});

describeDevMoonbeam("Precompiles - xcm transactor", (context) => {
  it("Moonbase: GeneralAdmin should be able to dispatch hrmpManage", async function () {
    // Just build the arguments. They dont matter that much though, since
    // we will not make sure it executes in the relay
    const transactWeights = context.polkadotApi.createType("PalletXcmTransactorTransactWeights", {
      transactRequiredWeightAtMost: { refTime: 10000, proofSize: 10000 },
      overallWeight: { refTime: 10000, proofSize: 10000 },
    });

    let fee = context.polkadotApi.createType("PalletXcmTransactorCurrencyPayment", {
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
    }) as any;

    // send HrmpManage
    await dispatchAsGeneralAdmin(
      context,
      (context.polkadotApi.tx.xcmTransactor as any).hrmpManage(
        {
          Accept: {
            para_id: 2000,
          },
        },
        fee,
        transactWeights
      ) as any
    );

    // Filter for HrmpManagementSent events
    const records = (await context.polkadotApi.query.system.events()) as any;
    const events = records.filter(
      ({ event }) => event.section == "xcmTransactor" && event.method == "HrmpManagementSent"
    );

    // It executed!
    expect(events).to.have.lengthOf(1);
  });
});
