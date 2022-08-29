import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

const debug = require("debug")("smoke:ethereum-current");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

const BLOCK_WINDOW = 600;

// Ethereum use Patricia trees for the various trees in blocks.
// Since we're going to check that no transactions means an empty receipt
// tree, we must compute what is the root of such empty trie.
// The following Rust snippet allow to do such that (using crates used by
// the implementation in Frontier) :
//
// ```rust
// use ethereum_types::H256;
// use keccak_hasher::KeccakHasher;
// use triehash::ordered_trie_root;
//
// fn main() {
//     let root = ordered_trie_root::<KeccakHasher, &[H256]>(&[]);
//     let root = H256::from(root);
//     println!("{:?}", root);
// }
// ```
//
// It outputs the following constant:
const EMPTY_TRIE_ROOT = "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421";

function* range(from, to, step = 1) {
  let value = from;
  while (value <= to) {
    yield value;
    value += step;
  }
}

describeSmokeSuite(
  `Ethereum CurrentBlock and CurrentReceipts should never be 0x00..`,
  { wssUrl, relayWssUrl },
  (context) => {
    it("should have non default field values", async function () {
      this.timeout(6_000_000); // 30 minutes
      const lastBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
      const firstBlockNumber = lastBlockNumber - BLOCK_WINDOW + 1;

      for (let blockNumber of range(firstBlockNumber, lastBlockNumber)) {
        let api = await context.polkadotApi.at(
          await context.polkadotApi.rpc.chain.getBlockHash(blockNumber)
        );

        const block = (await api.query.ethereum.currentBlock()).unwrap();
        const receipts = (await api.query.ethereum.currentReceipts()).unwrap().toArray();

        expect(block.header.parentHash).to.not.be.equal(
          "0x" + "".padEnd(64, "0"),
          `Parent hash of block ${blockNumber} shound not be 0x00...`
        );
        expect(block.header.stateRoot).to.not.be.equal(
          "0x" + "".padEnd(64, "0"),
          `Stateroot of block ${blockNumber} shound not be 0x00...`
        );

        // No transactions
        if (block.transactions.length == 0) {
          // Receipt trie
          expect(block.header.receiptsRoot.toString()).to.be.equal(
            EMPTY_TRIE_ROOT,
            `Receipts root of block ${blockNumber} shound be the empty trie root as the block
            contains no transaction`
          );

          // Receipts
          expect(receipts.length).to.be.equal(
            0,
            `Receipts of blocks ${blockNumber} should be empty as the block contains no
            transactions`
          );
        }
        // Some transactions
        else {
          // Receipt tree
          expect(block.header.receiptsRoot.toString()).to.not.be.equal(
            "0x" + "".padEnd(64, "0"),
            `Receipts root of block ${blockNumber} shound not be 0x00`
          );
          expect(block.header.receiptsRoot.toString()).to.not.be.equal(
            EMPTY_TRIE_ROOT,
            `Receipts root of block ${blockNumber} shound not be the empty tree root as the block
            contains transactions`
          );

          // Receipts
          expect(receipts.length).to.not.be.equal(
            0,
            `Receipts of blocks ${blockNumber} should be not be empty as the block contains
            transactions`
          );
        }
      }
    });
  }
);
