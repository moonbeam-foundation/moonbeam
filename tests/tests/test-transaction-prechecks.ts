import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import {
  FIRST_CONTRACT_ADDRESS,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  TEST_CONTRACT_ABI,
  TEST_CONTRACT_BYTECODE,
} from "./constants";

let tx;

describeWithMoonbeam(
  "Moonbeam RPC (Transaction validity pre-checks)",
  `simple-specs.json`,
  (context) => {
    it("should not return error on the same block", async function () {
      tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: TEST_CONTRACT_BYTECODE,
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x100000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      // `a` is a new submitted transaction.
      let a = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
      // `b` is already known at Rpc-level, and is not submitted to the pool.
      let b = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

      expect(a.result).to.be.equal(
        "0xe87ed993e4d186748404a52a2d13612eef8356331f30fa6b3fb9bc2c16be2e9c"
      );
      // Before adding the checks, bellow assertion would fail with `submit transaction to
      // pool failed: Pool(AlreadyImported(Any))`.
      expect(b.result).to.be.equal(
        "0xe87ed993e4d186748404a52a2d13612eef8356331f30fa6b3fb9bc2c16be2e9c"
      );
    });

    it("should be Stale next block", async function () {
      await createAndFinalizeBlock(context.polkadotApi);
      // In a new block, `a` is not known at Rpc-level because is not in the transaction pool
      // anymore.
      //
      // `a` will be submitted to the pool and identified as Stale in
      // pallet-ethereum::validate_unsigned(), because the transaction nonce (0) is lesser
      // than the account nonce (1).
      //
      // `a` therefore is added to the banned list when invalid::Stale.
      let a = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
      // `b` is now banned, and already known at Rpc-level, so is not submitted to the pool.
      let b = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
      expect(a).to.deep.equal({
        jsonrpc: "2.0",
        error: {
          code: -32603,
          message:
            "submit transaction to pool failed: " +
            "Pool(InvalidTransaction(InvalidTransaction::Stale))",
        },
        id: 1,
      });
      // Before adding the checks, bellow assertion would fail with `submit transaction to
      // pool failed: Pool(TemporarilyBanned)`.
      // `b` will now return the known (banned) transaction hash instead.
      expect(b.result).to.be.equal(
        "0xe87ed993e4d186748404a52a2d13612eef8356331f30fa6b3fb9bc2c16be2e9c"
      );
    });
  }
);
