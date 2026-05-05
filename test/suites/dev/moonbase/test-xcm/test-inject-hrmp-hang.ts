import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "moonwall";

const HRMP_CHANNEL_CAPACITY = 100;
const HANG_TIMEOUT_MS = 5_000;

function withTimeout<T>(p: Promise<T>, ms: number): Promise<T | "__timeout__"> {
  return Promise.race([
    p,
    new Promise<"__timeout__">((resolve) => setTimeout(() => resolve("__timeout__"), ms)),
  ]);
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

        // Once the channel is full, send_async parks the future and the
        // RPC handler never returns — the connection is held open until
        // a block drains the queue. The handler should fail fast instead.
        const result = await withTimeout(
          customDevRpcRequest("xcm_injectHrmpMessage", [1, []]),
          HANG_TIMEOUT_MS
        );

        expect(
          result,
          "xcm_injectHrmpMessage should return promptly when channel is full"
        ).not.toBe("__timeout__");
      },
    });
  },
});
