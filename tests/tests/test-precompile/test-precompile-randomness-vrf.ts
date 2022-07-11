import "@moonbeam-network/api-augment";
import { EventRecord } from "@polkadot/types/interfaces";
import { ethers } from "ethers";
import { alith } from "../../util/accounts";
import { GLMR, PRECOMPILE_RANDOMNESS_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  createTransaction,
} from "../../util/transactions";
import { expect } from "chai";

const RANDOMNESS_CONTRACT = getCompiled("Randomness");
const RANDOMNESS_INTERFACE = new ethers.utils.Interface(RANDOMNESS_CONTRACT.contract.abi);

describeDevMoonbeam("Precompile Randomness - request relay block number", (context) => {
  it("should succeed with Returned", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("relayBlockNumber"),
      })
    );
    expectEVMResult(result.events, "Succeed", "Returned");
  });
});

describeDevMoonbeam("Precompile Randomness - request relay epoch index", (context) => {
  it("should succeed with Returned", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("relayEpochIndex"),
      })
    );
    expectEVMResult(result.events, "Succeed", "Returned");
  });
});

describeDevMoonbeam(
  "Precompile Randomness - request babe randomness - current block",
  (context) => {
    it("should succeed with Returned", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestBabeRandomnessCurrentBlock", [
            alith.address,
            1n * GLMR,
            1_000_000,
            new Array(32).fill(0x1f),
            1n,
          ]),
        })
      );

      expectEVMResult(result.events, "Succeed", "Returned");
      expect(
        result.events.find(
          ({ event: { section, method } }) =>
            section === "randomness" && method === "RandomnessRequestedCurrentBlock"
        )
      ).to.exist;
    });
  }
);

describeDevMoonbeam(
  "Precompile Randomness - request babe randomness - one epoch ago",
  (context) => {
    it("should succeed with Returned", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestBabeRandomnessOneEpochAgo", [
            alith.address,
            1n * GLMR,
            1_000_000,
            new Array(32).fill(0x1f),
          ]),
        })
      );

      expectEVMResult(result.events, "Succeed", "Returned");
      expect(
        result.events.find(
          ({ event: { section, method } }) =>
            section === "randomness" && method === "RandomnessRequestedBabeEpoch"
        )
      ).to.exist;
    });
  }
);

describeDevMoonbeam(
  "Precompile Randomness - request babe randomness - two epoch ago",
  (context) => {
    it("should succeed with Returned", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestBabeRandomnessTwoEpochsAgo", [
            alith.address,
            1n * GLMR,
            1_000_000,
            new Array(32).fill(0x1f),
          ]),
        })
      );

      expectEVMResult(result.events, "Succeed", "Returned");
      expect(
        result.events.find(
          ({ event: { section, method } }) =>
            section === "randomness" && method === "RandomnessRequestedBabeTwoEpochsAgo"
        )
      ).to.exist;
    });
  }
);

describeDevMoonbeam("Precompile Randomness - request local randomness - in past", (context) => {
  it("should fail with Error", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
          alith.address,
          1n * GLMR,
          1_000_000,
          new Array(32).fill(0x1f),
          0n,
        ]),
      })
    );

    expectEVMResult(result.events, "Error", "Other");
  });
});

describeDevMoonbeam("Precompile Randomness - request local randomness - low gas fee", (context) => {
  it("should succeed", async function () {
    const blockNumber = await context.polkadotApi.query.system.number();
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
          alith.address,
          0n,
          1_000_000,
          new Array(32).fill(0x1f),
          blockNumber.addn(1).toNumber(),
        ]),
      })
    );

    expectEVMResult(result.events, "Succeed", "Returned");
  });
});

describeDevMoonbeam(
  "Precompile Randomness - request local randomness - low gas limit",
  (context) => {
    it("should succeed", async function () {
      const blockNumber = await context.polkadotApi.query.system.number();
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
            alith.address,
            1n * GLMR,
            0,
            new Array(32).fill(0x1f),
            blockNumber.addn(1).toNumber(),
          ]),
        })
      );

      expectEVMResult(result.events, "Succeed", "Returned");
    });
  }
);

describeDevMoonbeam("Precompile Randomness - request local randomness", (context) => {
  it("should succeed with Returned", async function () {
    const blockNumber = await context.polkadotApi.query.system.number();
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
          alith.address,
          1n * GLMR,
          1_000_000,
          new Array(32).fill(0x1f),
          blockNumber.addn(1).toNumber(),
        ]),
      })
    );

    expectEVMResult(result.events, "Succeed", "Returned");
    expect(
      result.events.find(
        ({ event: { section, method } }) =>
          section === "randomness" && method === "RandomnessRequestedLocal"
      )
    ).to.exist;
  });
});

describeDevMoonbeam("Precompile Randomness - fulfill request - missing request", (context) => {
  it("should fail with Error", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [1]),
      })
    );

    expectEVMResult(result.events, "Error", "Other");
  });
});

describeDevMoonbeam("Precompile Randomness - fulfill request - request in future", (context) => {
  before("request randomness", async function () {
    const blockNumber = await context.polkadotApi.query.system.number();
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
          alith.address,
          1n * GLMR,
          1_000_000,
          new Array(32).fill(0x1f),
          blockNumber.addn(10).toNumber(),
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed", "Returned");
  });

  it("should fail with Error", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );

    expectEVMResult(result.events, "Error", "Other");
  });
});

describeDevMoonbeam("Precompile Randomness - fulfill request - low gas fee", (context) => {
  before("request randomness", async function () {
    const blockNumber = await context.polkadotApi.query.system.number();
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
          alith.address,
          0n,
          1_000_000,
          new Array(32).fill(0x1f),
          blockNumber.addn(1).toNumber(),
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed", "Returned");
  });

  it("should revert", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );

    expectEVMResult(result.events, "Revert", "Reverted");
  });
});

describeDevMoonbeam("Precompile Randomness - fulfill request - low gas limit", (context) => {
  before("request randomness", async function () {
    const blockNumber = await context.polkadotApi.query.system.number();
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
          alith.address,
          1n * GLMR,
          0,
          new Array(32).fill(0x1f),
          blockNumber.addn(1).toNumber(),
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed", "Returned");
  });

  it("should revert", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );

    expectEVMResult(result.events, "Revert", "Reverted");
  });
});

describeDevMoonbeam("Precompile Randomness - fulfill request - valid request", (context) => {
  before("request randomness", async function () {
    const blockNumber = await context.polkadotApi.query.system.number();
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
          alith.address,
          1n * GLMR,
          1_000_000,
          new Array(32).fill(0x1f),
          blockNumber.addn(1).toNumber(),
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed", "Returned");
  });

  it("should succeed with Returned", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );

    expectEVMResult(result.events, "Succeed", "Returned");
  });
});

describeDevMoonbeam("Precompile Randomness - increase request fee - missing request", (context) => {
  it("should fail with Error", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("increaseRequestFee", [1, 5n * GLMR]),
      })
    );

    expectEVMResult(result.events, "Error", "Other");
  });
});

describeDevMoonbeam(
  "Precompile Randomness - increase request fee - different requester",
  (context) => {
    before("request randomness", async function () {
      const blockNumber = await context.polkadotApi.query.system.number();
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
            alith.address,
            1n * GLMR,
            1_000_000,
            new Array(32).fill(0x1f),
            blockNumber.addn(1).toNumber(),
          ]),
        })
      );
      expectEVMResult(result.events, "Succeed", "Returned");
    });

    it("should fail with Error", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("increaseRequestFee", [0, 2n * GLMR]),
        })
      );

      expectEVMResult(result.events, "Error", "Other");
    });
  }
);

describeDevMoonbeam("Precompile Randomness - increase request fee - valid request", (context) => {
  before("request randomness", async function () {
    const blockNumber = await context.polkadotApi.query.system.number();
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
          alith.address,
          1n * GLMR,
          1_000_000,
          new Array(32).fill(0x1f),
          blockNumber.addn(1).toNumber(),
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed", "Returned");
  });

  it("should succeed with Returned", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("increaseRequestFee", [0, 10n]),
      })
    );

    expectEVMResult(result.events, "Succeed", "Returned");
  });
});

describeDevMoonbeam(
  "Precompile Randomness - execute request expiration - missing request",
  (context) => {
    it("should fail with Error", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("executeRequestExpiration", [1]),
        })
      );

      expectEVMResult(result.events, "Error", "Other");
    });
  }
);

describeDevMoonbeam(
  "Precompile Randomness - execute request expiration - request not expired",
  (context) => {
    before("request randomness", async function () {
      const blockNumber = await context.polkadotApi.query.system.number();
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
            alith.address,
            1n * GLMR,
            1_000_000,
            new Array(32).fill(0x1f),
            blockNumber.addn(1).toNumber(),
          ]),
        })
      );
      expectEVMResult(result.events, "Succeed", "Returned");
    });

    it("should succeed with Returned", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("executeRequestExpiration", [0]),
        })
      );

      expectEVMResult(result.events, "Error", "Other");
    });
  }
);

describeDevMoonbeam(
  "Precompile Randomness - execute request expiration - valid request",
  (context) => {
    before("request randomness and skip to expiration delay", async function () {
      const blockNumber = await context.polkadotApi.query.system.number();
      const block = await context.createBlock([
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
            alith.address,
            1n * GLMR,
            1_000_000,
            new Array(32).fill(0x1f),
            blockNumber.addn(1).toNumber(),
          ]),
        }),
      ]);
      expectEVMResult(block.result[0].events, "Succeed", "Returned");

      for (let i = 0; i < Number(context.polkadotApi.consts.randomness.expirationDelay); i++) {
        await context.createBlock();
      }
    });

    it("should succeed with Returned", async function () {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("executeRequestExpiration", [0]),
        })
      );

      expectEVMResult(result.events, "Succeed", "Returned");
    });
  }
);
