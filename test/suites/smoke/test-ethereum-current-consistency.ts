import "@moonbeam-network/api-augment";
import { expect, describeSuite } from "@moonwall/cli";
import { THIRTY_MINS } from "@moonwall/util";
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

function* range(from: number, to: number, step = 1) {
  let value = from;
  while (value <= to) {
    yield value;
    value += step;
  }
}

describeSuite({
  id: "S10",
  title: "Ethereum CurrentBlock and CurrentReceipts should never be 0x00",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    it({
      id: "C100",
      title: "should have non default field values",
      timeout: THIRTY_MINS,
      test: async function () {
        const paraApi = context.polkadotJs("para");

        const lastBlockNumber = (await paraApi.rpc.chain.getHeader()).number.toNumber();
        const roundLength = (await paraApi.query.parachainStaking.round()).length.toNumber();

        const blocksToWait = process.env.BATCH_OF
          ? parseInt(process.env.BATCH_OF)
          : process.env.ROUNDS_TO_WAIT
          ? Math.floor(Number(process.env.ROUNDS_TO_WAIT) * roundLength)
          : 200;
        const firstBlockNumber = Math.max(lastBlockNumber - blocksToWait + 1, 1);

        for (const blockNumber of range(firstBlockNumber, lastBlockNumber)) {
          const api = await paraApi.at(await paraApi.rpc.chain.getBlockHash(blockNumber));

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
      },
    });
  },
});
