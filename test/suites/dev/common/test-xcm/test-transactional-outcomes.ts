import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith, ZERO_ADDRESS, BALTATHAR_ADDRESS, baltathar } from "@moonwall/util";
import {
  ERC20_TOTAL_SUPPLY,
  XcmFragment,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
  getFreeBalance,
} from "../../../../helpers";

describeSuite({
  id: "D016000",
  title: "Validate XCM TransactionalProcessor outcomes",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const paraId = 1000;
    let paraSovereign: string;

    let initialParaSovereignBalance = 0n;
    let erc20ContractAddress: string;
    let balancesPalletIndex: number;
    let erc20XcmPalletIndex: number;

    const DEPOSIT = 50_000_000_000_000_000n;
    // On moonriver and moonbeam the cost should be 100 times lower (caused by the SUPPLY_FACTOR)
    const MAX_EXECUTION_COST = 32_000_000_000_000_000n;

    beforeAll(async () => {
      paraSovereign = sovereignAccountOfSibling(context, paraId);

      // Drain account
      await context.createBlock(
        context.polkadotJs().tx.balances.transferAll(ZERO_ADDRESS, false).signAsync(baltathar),
        { allowFailures: false }
      );

      const initialBaltatharBalance = await getFreeBalance(BALTATHAR_ADDRESS, context);
      expect(initialBaltatharBalance).eq(0n);

      // Send some native tokens to the sovereign account of paraId
      await context.createBlock(
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(paraSovereign, DEPOSIT)
          .signAsync(alith),
        { allowFailures: false }
      );

      initialParaSovereignBalance = await getFreeBalance(paraSovereign as any, context);

      // Deploy an erc20 contract
      const erc20_contract = await context.deployContract!("ERC20WithInitialSupply", {
        args: ["Token", "Token", ALITH_ADDRESS, ERC20_TOTAL_SUPPLY],
      });
      erc20ContractAddress = erc20_contract.contractAddress;
      expect(erc20_contract.status).eq("success");

      // Get pallet indices
      const metadata = await context.polkadotJs().rpc.state.getMetadata();
      balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Balances")!
        .index.toNumber();
      erc20XcmPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Erc20XcmBridge")!
        .index.toNumber();
    });

    it({
      id: "T01",
      title: "Assert balances after XCM message",
      test: async function () {
        const xcmMessage = new XcmFragment(null)
          .push_any({
            WithdrawAsset: [
              {
                id: {
                  Concrete: {
                    parents: 0,
                    interior: {
                      X1: { PalletInstance: Number(balancesPalletIndex) },
                    },
                  },
                },
                fun: {
                  Fungible: DEPOSIT,
                },
              },
              {
                id: {
                  Concrete: {
                    parents: 0,
                    interior: {
                      X2: [
                        {
                          PalletInstance: erc20XcmPalletIndex,
                        },
                        {
                          AccountKey20: {
                            network: null,
                            key: erc20ContractAddress,
                          },
                        },
                      ],
                    },
                  },
                },
                fun: {
                  // The sovereign account balance should be zero, this will fail
                  Fungible: 1,
                },
              },
            ],
          })
          .clear_origin()
          .push_any({
            BuyExecution: {
              fees: {
                id: {
                  Concrete: {
                    parents: 0,
                    interior: { X1: { PalletInstance: Number(balancesPalletIndex) } },
                  },
                },
                fun: { Fungible: MAX_EXECUTION_COST },
              },
              weightLimit: { Unlimited: null },
            },
          })
          // Let's set an error handler just for extra checks against the holding registry
          // at the end of the message
          .push_any({
            SetErrorHandler: [
              {
                RefundSurplus: null,
              },
              {
                DepositAsset: {
                  assets: {
                    Definite: [
                      {
                        id: {
                          Concrete: {
                            parents: 0,
                            interior: { X1: { PalletInstance: Number(balancesPalletIndex) } },
                          },
                        },
                        fun: { Fungible: DEPOSIT },
                      },
                    ],
                  },
                  beneficiary: {
                    parents: 0,
                    interior: { X1: { AccountKey20: { network: null, key: BALTATHAR_ADDRESS } } },
                  },
                },
              },
            ],
          })
          .push_any({
            DepositAsset: {
              assets: {
                Definite: [
                  {
                    id: {
                      Concrete: {
                        parents: 0,
                        interior: { X1: { PalletInstance: Number(balancesPalletIndex) } },
                      },
                    },
                    fun: { Fungible: DEPOSIT },
                  },
                  {
                    id: {
                      Concrete: {
                        parents: 0,
                        interior: {
                          X2: [
                            { PalletInstance: erc20XcmPalletIndex },
                            { AccountKey20: { network: null, key: erc20ContractAddress } },
                          ],
                        },
                      },
                    },
                    // The sovereign account balance should be zero, this will fail
                    fun: { Fungible: 1 },
                  },
                ],
              },
              beneficiary: {
                parents: 0,
                interior: { X1: { AccountKey20: { network: null, key: BALTATHAR_ADDRESS } } },
              },
            },
          })
          .as_v3();

        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });

        const events = await context.polkadotJs().query.system.events();
        const mints = events
          .filter((evt) => context.polkadotJs().events.balances.Minted.is(evt.event))
          .map((evt) => evt.event.toJSON().data);
        const totalMinted = mints.reduce((prev, cur) => prev + BigInt(cur[1]), 0n);
        const executionCost = BigInt(mints[1][1]);

        expect(mints.length).toBe(2);
        expect(totalMinted).toBe(DEPOSIT);

        const finalBaltatharBalance = await getFreeBalance(BALTATHAR_ADDRESS, context);
        console.log("final_baltathar_balance:", finalBaltatharBalance);
        expect(finalBaltatharBalance).toBe(DEPOSIT - executionCost);

        const finalParaSovereignBalance = await getFreeBalance(paraSovereign as any, context);
        const paraSovereignBalanceDiff = initialParaSovereignBalance - finalParaSovereignBalance;
        console.log("initial_paraSovereign_balance:", initialParaSovereignBalance);
        console.log("final_paraSovereign_balance:", finalParaSovereignBalance);
        console.log("diff:", paraSovereignBalanceDiff);
        expect(paraSovereignBalanceDiff).toBe(DEPOSIT);
      },
    });
  },
});
