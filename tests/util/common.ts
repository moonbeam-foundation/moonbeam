import { BN } from "@polkadot/util";

// Sort dict by key
export function sortObjectByKeys(o) {
  return Object.keys(o)
    .sort()
    .reduce((r, k) => ((r[k] = o[k]), r), {});
}

// Perthings arithmetic conformant type.
class Perthing {
  private unit: BN;
  private perthing: BN;

  constructor(unit: BN, numerator: BN | number, denominator?: BN | number) {
    if (!(numerator instanceof BN)) {
      numerator = new BN(numerator.toString());
    }
    if (denominator && !(denominator instanceof BN)) {
      denominator = new BN(denominator.toString());
    }

    this.unit = unit;
    if (denominator) {
      this.perthing = numerator.mul(unit).div(denominator as BN);
    } else {
      this.perthing = numerator;
    }
  }

  value(): BN {
    return this.perthing;
  }

  of(value: BN): BN {
    return this.divNearest(this.perthing.mul(value), this.unit);
  }

  ofCeil(value: BN): BN {
    return this.divCeil(this.perthing.mul(value), this.unit);
  }

  toString(): string {
    return `${this.perthing.toString()}`;
  }

  divCeil(a: any, num: BN) {
    var dm = a.divmod(num);

    // Fast case - exact division
    if (dm.mod.isZero()) return dm.div;

    // Round up
    return dm.div.negative !== 0 ? dm.div.isubn(1) : dm.div.iaddn(1);
  }

  divNearest(a: any, num: BN) {
    var dm = a.divmod(num);

    // Fast case - exact division
    if (dm.mod.isZero()) return dm.div;

    var mod = dm.div.negative !== 0 ? dm.mod.isub(num) : dm.mod;

    var half = num.ushrn(1);
    var r2 = num.andln(1) as any;
    var cmp = mod.cmp(half);

    // Round down
    if (cmp <= 0 || (r2 === 1 && cmp === 0)) return dm.div;

    // Round up
    return dm.div.negative !== 0 ? dm.div.isubn(1) : dm.div.iaddn(1);
  }
}

// Perthings arithmetic conformant type representing part(s) per billion.
export class Perbill extends Perthing {
  constructor(numerator: BN | number, denominator?: BN | number) {
    super(new BN(1_000_000_000), numerator, denominator);
  }
}

// Perthings arithmetic conformant type representing part(s) per cent.
export class Percent extends Perthing {
  constructor(numerator: BN | number, denominator?: BN | number) {
    super(new BN(100), numerator, denominator);
  }
}

export function getObjectMethods(obj) {
  let properties = new Set();
  let currentObj = obj;
  do {
    Object.getOwnPropertyNames(currentObj).map((item) => properties.add(item));
  } while ((currentObj = Object.getPrototypeOf(currentObj)));
  return [...properties.keys()].filter((item: any) => typeof obj[item] === "function");
}
