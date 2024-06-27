import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { Abi, encodeFunctionData } from "viem";
import {
  RawXcmMessage,
  XcmFragment,
  descendOriginFromAddress20,
  injectHrmpMessage,
} from "../../helpers";

describeSuite({
  id: "T11",
  title: "Trace ethereum xcm #2",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let ethereumXcmDescendedOrigin: `0x${string}`;
    let xcmContractAddress: `0x${string}`;
    let ethContractAddress: `0x${string}`;
    let xcmContractABI: Abi;

    beforeAll(async () => {
      const { contractAddress, abi } = await context.deployContract!("Incrementor");
      xcmContractAddress = contractAddress;
      xcmContractABI = abi;

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      ethereumXcmDescendedOrigin = descendOriginAddress;

      const sendingAddress = originAddress;
      const transferredBalance = 10_000_000_000_000_000_000n;

      // We first fund parachain 2000 sovreign account
      await context.createBlock(
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(ethereumXcmDescendedOrigin, transferredBalance),
        { allowFailures: false }
      );

      // Get Pallet balances index
      const metadata = await context.polkadotJs().rpc.state.getMetadata();
      const balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() == "Balances")!
        .index.toNumber();

      const xcmTransaction = {
        V2: {
          gas_limit: 100000,
          action: {
            Call: xcmContractAddress,
          },
          value: 0n,
          input: encodeFunctionData({
            abi: xcmContractABI,
            functionName: "incr",
            args: [],
          }),
          access_list: null,
        },
      };

      const transferCall = context.polkadotJs().tx.ethereumXcm.transact(xcmTransaction);
      const transferCallEncoded = transferCall?.method.toHex();
      const xcmMessage = new XcmFragment({
        assets: [
          {
            multilocation: {
              parents: 0,
              interior: {
                X1: { PalletInstance: balancesPalletIndex },
              },
            },
            fungible: transferredBalance / 2n,
          },
        ],
        weight_limit: {
          refTime: 4000000000n,
          proofSize: 80000n,
        } as any,
        descend_origin: sendingAddress,
      })
        .descend_origin()
        .withdraw_asset()
        .buy_execution()
        .push_any({
          Transact: {
            originKind: "SovereignAccount",
            requireWeightAtMost: {
              refTime: 3000000000n,
              proofSize: 50000n,
            },
            call: {
              encoded: transferCallEncoded,
            },
          },
        })
        .as_v3();

      // Send an XCM and create block to execute it
      await injectHrmpMessage(context, 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

      // By calling deployContract() a new block will be created,
      // including the ethereum xcm call + regular ethereum transaction
      const { contractAddress: eventEmitterAddress } = await context.deployContract!(
        "EventEmitter",
        {
          from: alith.address,
        } as any
      );
      ethContractAddress = eventEmitterAddress;
    });

    it({
      id: "T01",
      title: "should trace ethereum xcm transactions with debug_traceBlockByNumber",
      test: async function () {
        const number = await context.viem().getBlockNumber();
        const trace = await customDevRpcRequest("debug_traceBlockByNumber", [
          number.toString(),
          { tracer: "callTracer" },
        ]);
        // 2 ethereum transactions: ethereum xcm call + regular ethereum transaction
        expect(trace.length).to.eq(2);
        // 1st transaction is regular ethereum transaction.
        // - `From` is Alith's adddress.
        // - `To` is the ethereum contract address.
        expect(trace[0].from).to.eq(alith.address.toLowerCase());
        expect(trace[0].to).to.eq(ethContractAddress.toLowerCase());
        expect(trace[0].type).be.eq("CREATE");
        // 2nd transaction is xcm.
        // - `From` is the descended origin.
        // - `To` is the xcm contract address.
        expect(trace[1].from).to.eq(ethereumXcmDescendedOrigin.toLowerCase());
        expect(trace[1].to).to.eq(xcmContractAddress.toLowerCase());
        expect(trace[1].type).to.eq("CALL");
      },
    });
  },
});
