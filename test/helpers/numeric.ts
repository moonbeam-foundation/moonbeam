import { BN } from "@polkadot/util";

/**
 * Percent class - represents a percentage value (0-100)
 * Used for calculating percentage portions of amounts
 */
export class Percent {
  private readonly percent: bigint;

  constructor(value: number | bigint) {
    this.percent = BigInt(value);
  }

  /**
   * Calculate percentage of an amount (floor division)
   * Supports both BN and bigint inputs
   */
  of(amount: BN | bigint): BN | bigint {
    if (amount instanceof BN) {
      return amount.muln(Number(this.percent)).divn(100);
    }
    return (amount * this.percent) / 100n;
  }

  /**
   * Calculate percentage of an amount (ceiling division)
   * Supports both BN and bigint inputs
   */
  ofCeil(amount: BN | bigint): BN | bigint {
    if (amount instanceof BN) {
      const result = amount.muln(Number(this.percent));
      return result.addn(99).divn(100);
    }
    return (amount * this.percent + 99n) / 100n;
  }
}

/**
 * Perbill class - represents parts per billion (0-1,000,000,000)
 * 1,000,000,000 (1e9) = 100%
 */
export class Perbill {
  private readonly perbill: BN;

  /**
   * Create a Perbill from a raw value or a fraction
   * @param numerator - The raw perbill value, or numerator if denominator is provided
   * @param denominator - Optional denominator to express as fraction (e.g., 355/1000 = 35.5%)
   */
  constructor(numerator: number | bigint | BN, denominator?: number) {
    if (denominator !== undefined) {
      // Fractional form: numerator/denominator expressed as perbill
      // e.g., Perbill(355, 1000) = 355/1000 * 1e9 = 355_000_000
      const num =
        typeof numerator === "bigint"
          ? Number(numerator)
          : numerator instanceof BN
            ? numerator.toNumber()
            : numerator;
      this.perbill = new BN(Math.floor((num / denominator) * 1_000_000_000));
    } else {
      // Raw perbill value
      if (numerator instanceof BN) {
        this.perbill = numerator;
      } else {
        this.perbill = new BN(numerator.toString());
      }
    }
  }

  /**
   * Get the raw perbill value as BN
   */
  value(): BN {
    return this.perbill;
  }

  /**
   * Calculate the perbill portion of an amount
   */
  of(amount: BN | bigint): BN | bigint {
    if (amount instanceof BN) {
      return amount.mul(this.perbill).div(new BN(1_000_000_000));
    }
    return (amount * BigInt(this.perbill.toString())) / 1_000_000_000n;
  }
}
