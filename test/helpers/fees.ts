import { BN } from "@polkadot/util";

/// Recreation of fees.ration(burn_part, treasury_part)
export const split = (value: BN, part1: BN, part2: BN): [BN, BN] => {
  const total = part1.add(part2);
  if (total.eq(new BN(0)) || value.eq(new BN(0))) {
    return [new BN(0), new BN(0)];
  }
  const part1BN = value.mul(part1).div(total);
  const part2BN = value.sub(part1BN);
  return [part1BN, part2BN];
};

export const calculateFeePortions = (
  feesTreasuryProportion: bigint,
  fees: bigint
): {
  burnt: bigint;
  treasury: bigint;
} => {
  const feesBN = new BN(fees.toString());
  const treasuryPartBN = new BN(feesTreasuryProportion.toString());
  const burntPartBN = new BN(1e9).sub(treasuryPartBN);

  const [burntBN, treasuryBN] = split(feesBN, burntPartBN, treasuryPartBN);

  return {
    burnt: BigInt(burntBN.toString()),
    treasury: BigInt(treasuryBN.toString()),
  };
};
