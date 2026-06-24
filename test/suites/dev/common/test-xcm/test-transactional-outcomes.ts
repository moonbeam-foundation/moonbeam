import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  ZERO_ADDRESS,
  alith,
  baltathar,
  beforeAll,
  describeSuite,
  expect,
} from "moonwall";
import {
  ERC20_TOTAL_SUPPLY,
  XcmFragment,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
  getFreeBalance,
} from "../../../../helpers";

describeSuite({
  id: "D010701",
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
      title: "Native deposit is recovered when a multi-asset erc20 leg fails",
      test: async function () {
        // Multi-asset XCM (native + erc20) whose erc20 leg is doomed to fail: the paraId
        // sovereign holds 0 of the erc20, so its deferred EVM transfer reverts at DepositAsset.
        // Asserts the partial failure is contained transactionally -- atomic rollback, native
        // recovered via the SetErrorHandler, value conserved. Happy-path mirror (funded
        // sovereign, erc20 succeeds): test-xcm-v5/test-xcm-erc20-transfer.ts (T02).
        const xcmMessage = new XcmFragment({} as any)
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
                  // erc20 withdraw is notional: records the sovereign as drain origin, no balance
                  // check. It does NOT fail here -- the failure is the EVM transfer in DepositAsset.
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
                    // Deferred EVM transfer drains from the sovereign (0 balance) and reverts:
                    // the leg that fails and rolls back the whole DepositAsset instruction.
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

        // `injectHrmpMessageAndSeal` seals until the message queue actually
        // processes the message, so the current block's events reliably contain
        // the resulting mints (otherwise this test was racy/flaky).
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });

        const events = await context.polkadotJs().query.system.events();
        // stable2603 credit model (polkadot-sdk #10384) moves native value via Withdraw/Deposit
        // imbalances instead of minting, so the execution fee surfaces as a treasury
        // `balances.Deposit`, not `balances.Minted`. The multi-asset DepositAsset fails atomically
        // (erc20 revert rolls back the native leg via FrameTransactionalProcessor); Baltathar is
        // funded by the SetErrorHandler's native-only deposit afterwards.
        const deposits = events
          .filter((evt) => context.polkadotJs().events.balances.Deposit.is(evt.event))
          .map((evt) => evt.event.toJSON().data as [string, any]);
        const feeDeposit = deposits.find(
          ([who]) => who.toLowerCase() !== BALTATHAR_ADDRESS.toLowerCase()
        );
        expect(feeDeposit, "execution fee should be deposited to the treasury").toBeDefined();
        const executionCost = BigInt(feeDeposit![1]);
        expect(executionCost > 0n).toBe(true);
        expect(executionCost <= MAX_EXECUTION_COST).toBe(true);

        // Baltathar received the full deposit minus the execution fee.
        const finalBaltatharBalance = await getFreeBalance(BALTATHAR_ADDRESS, context);
        expect(finalBaltatharBalance).toBe(DEPOSIT - executionCost);

        // The full DEPOSIT was withdrawn from the sovereign account.
        const finalParaSovereignBalance = await getFreeBalance(paraSovereign as any, context);
        const paraSovereignBalanceDiff = initialParaSovereignBalance - finalParaSovereignBalance;
        expect(paraSovereignBalanceDiff).toBe(DEPOSIT);
      },
    });
  },
});
