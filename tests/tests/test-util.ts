import { expect } from "chai";
import { calculateTxnDataCost } from "../util/transactions";

// even tests need tests

describe("should calculate txn data cost correctly", function() {
  it("should handle basic cases", function() {
    expect(calculateTxnDataCost("00")).to.equal(4);
    expect(calculateTxnDataCost("0000000000000000000000000000000000")).to.equal(17 * 4);
    expect(calculateTxnDataCost("0002")).to.equal(16 + 4);
    expect(calculateTxnDataCost("ff0002")).to.equal(16 + 4 + 16);
  });

  it("should handle hex prefix", function() {
    expect(calculateTxnDataCost("0x00")).to.equal(4);
    expect(calculateTxnDataCost("0x0000")).to.equal(4 + 4);
    expect(calculateTxnDataCost("0xffff")).to.equal(16 + 16);
  });

  it("should handle istanbul parameter", function() {
    expect(calculateTxnDataCost("00", false)).to.equal(4);
    expect(calculateTxnDataCost("ff", false)).to.equal(64);
    expect(calculateTxnDataCost("00ff00", false)).to.equal(4 + 64 + 4);
    expect(calculateTxnDataCost("00ff00", true)).to.equal(4 + 16 + 4);
  });
});
