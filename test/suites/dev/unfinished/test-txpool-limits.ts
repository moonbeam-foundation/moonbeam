import "@moonbeam-network/api-augment";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("TxPool - Limits", (context) => {
  it.skip("should be able to fill a block with 260 tx", async function () {});

  it.skip("should be able to fill a block with 64 contract creations tx", async function () {});

  // 8192 is the number of tx that can be sent to the Pool
  // before it throws an error and drops all tx
  it.skip("should be able to send 8192 tx to the pool and have them all published\
    within the following blocks", async function () {});

  it.skip("shouldn't work for 8193", async function () {});

  it.skip("should be able to send 8192 tx to the pool and have them all published\
    within the following blocks - bigger tx", async function () {});

  it.skip("shouldn't work for 8193 - bigger tx", async function () {});
});
