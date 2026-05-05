import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "moonwall";

// Must match the `flume::bounded(100)` capacity for the HRMP channel
// in node/service/src/lib.rs.
const HRMP_CHANNEL_CAPACITY = 100;
const HANG_TIMEOUT_MS = 5_000;

function withTimeout<T>(p: Promise<T>, ms: number): Promise<T | "__timeout__"> {
  let timer: ReturnType<typeof setTimeout>;
  const timeout = new Promise<"__timeout__">((resolve) => {
    timer = setTimeout(() => resolve("__timeout__"), ms);
  });
  return Promise.race([p, timeout]).finally(() => clearTimeout(timer));
}

describeSuite({
  id: "D023917",
  title: "Mock XCM - xcm_injectHrmpMessage backpressure",
  foundationMethods: "dev",
  testCases: ({ it }) => {
    it({
      id: "T01",
      title: "should not hang when the inject channel is full",
      timeout: 30_000,
      test: async function () {
        // The dev RPC pushes onto a bounded(100) flume channel that is only
        // drained inside create_inherent_data_providers, i.e. on block
        // authorship. Fill it up without sealing a block.
        for (let i = 0; i < HRMP_CHANNEL_CAPACITY; i++) {
          await customDevRpcRequest("xcm_injectHrmpMessage", [1, []]);
        }

        // Once the channel is full, the previous (buggy) implementation
        // parked send_async and the RPC handler never returned. The fixed
        // handler must fail fast — either with a JSON-RPC error or, if a
        // block was authored in between, with a successful result.
        const call = customDevRpcRequest("xcm_injectHrmpMessage", [1, []]).then(
          () => "ok" as const,
          (err: Error) => ({ rpcError: err.message })
        );
        const result = await withTimeout(call, HANG_TIMEOUT_MS);

        expect(
          result,
          "xcm_injectHrmpMessage should return promptly when channel is full"
        ).not.toBe("__timeout__");
      },
    });
  },
});
