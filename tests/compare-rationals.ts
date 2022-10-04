import { BN } from "@polkadot/util";

function compareRationals(n1: BN, d1: BN, n2: BN, d2: BN): boolean {
  // Uses a continued fractional representation for a non-overflowing compare.
  // Detailed at https://janmr.com/blog/2014/05/comparing-rational-numbers-without-overflow/.
  while (true) {
    const q1 = n1.div(d1);
    const q2 = n2.div(d2);
    if (q1.lt(q2)) {
      return true;
    }
    if (q2.lt(q1)) {
      return false;
    }
    const r1 = n1.mod(d1);
    const r2 = n2.mod(d2);
    if (r2.isZero()) {
      return false;
    }
    if (r1.isZero()) {
      return true;
    }
    n1 = d2;
    n2 = d1;
    d1 = r2;
    d2 = r1;
  }
}

const [n1, d1, n2, d2] = process.argv.slice(2).map((arg) => new BN(arg));
console.log(compareRationals(n1, d1, n2, d2));
