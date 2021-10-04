// This script is expected to run against a parachain network (using launch.ts script)
import yargs from "yargs";
import BN from "bn.js";

import { getApiFor, NETWORK_YARGS_OPTIONS } from "./utils/networks";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    ...NETWORK_YARGS_OPTIONS,
    at: {
      type: "number",
      description: "Block number to look into",
      demandOption: true,
    },
    collator: {
      type: "string",
      description: "Display only given collator information",
    },
    nominator: {
      type: "string",
      description: "Display specific nominator information",
    },
  }).argv;

const main = async () => {
  const api = await getApiFor(argv);

  const blockNumber = argv.at;
  const blockHash = (await api.rpc.chain.getBlockHash(blockNumber)).toString();
  const records = await api.query.system.events.at(blockHash);
  const roundInfo = (await api.query.parachainStaking.round.at(blockHash)) as any;

  const roundDuration = 300;
  const paymentDurationRounds = 2;
  const roundNumber = roundInfo.current.toNumber() - paymentDurationRounds;

  // If we are querying a new round, get previous 2 rounds last block
  const roundLastBlockNumber =
    roundInfo.first.toNumber() == argv.at
      ? roundInfo.first.toNumber() - (roundDuration * (paymentDurationRounds - 1) + 1)
      : roundInfo.first.toNumber() +
        roundInfo.length.toNumber() -
        (roundDuration * (paymentDurationRounds - 1) + 1);
  console.log(
    `========= Checking payout at round ${roundInfo.current.toNumber()}, for round ${roundNumber}  (ending at ${roundLastBlockNumber})`
  );
  const roundBlockHash = (await api.rpc.chain.getBlockHash(roundLastBlockNumber)).toString();
  // const selectedCandidates = (
  //   (await api.query.parachainStaking.selectedCandidates.at(roundBlockHash)) as any
  // ).map((data) => data.toString()) as any[];

  // console.log(selectedCandidates.length);

  // const collatorStates = (
  //   await api.query.parachainStaking.collatorState2.entriesAt(roundBlockHash)
  // ).map(([_, data]) => api.registry.createType("Collator2", data.toHex())) as any[];

  // const collators = collatorStates
  //   .filter((collator) => selectedCandidates.includes(collator.id.toString()))
  //   .reduce((p, collator) => {
  //     p[collator.id.toString()] = {
  //       bond: collator.bond,
  //       total: collator.total_counted,
  //     };
  //     return p;
  //   }, {});
  const atStake = (await api.query.parachainStaking.atStake.entriesAt(
    roundBlockHash,
    roundNumber
  )) as any;

  const collators = atStake.reduce((p, [key, stake]) => {
    // console.log(key.args.map((k) => k.toHuman()).join("."), stake.total.toHuman());
    // stake.nominators.forEach((n) => {
    //   console.log(`${n.owner}: ${n.amount.toString()}`);
    // });
    p[key.args[1].toString()] = {
      bond: stake.bond,
      total: stake.total,
      nominators: stake.nominators,
      points: 0,
      pay: {},
    };
    return p;
  }, {});

  const awardedPts = await api.query.parachainStaking.awardedPts.entriesAt(
    roundBlockHash,
    roundNumber
  );
  awardedPts.forEach(([key, award]) => {
    collators[key.args[1].toString()].points = award;
  });

  let total = 0n;
  let lastCollator = 0;
  let nominators = {};
  records.forEach(({ event }) => {
    if (event.section == "parachainStaking" && event.method == "Rewarded") {
      const [acc, amount] = event.data as any;
      if (Object.keys(collators).includes(acc.toString())) {
        lastCollator = acc.toString();
      }
      if (!(acc in nominators)) {
        nominators[acc] = {};
      }
      nominators[acc][lastCollator] = amount;
      if (acc in collators) {
        collators[acc].pay[lastCollator] = amount;
      }
      total += amount.toBigInt();
    }
  });

  const precisionExp = 9n;
  const uMOVRPerPoint =
    (Number(total / 10n ** precisionExp) * 10 ** (12 - Number(precisionExp))) / 6000;

  console.log(`========= collators`);
  awardedPts.forEach(([key]) => {
    const id = key.args[1].toString();
    const collator = collators[id];

    let totalPaid = collator.pay[id].toBigInt();
    for (const staked of collator.nominators) {
      const nominator = nominators[staked.owner.toString()];
      const paid = nominator[id];
      totalPaid += paid.toBigInt();
    }

    if (!argv.collator || id.toLowerCase() == argv.collator.toLowerCase()) {
      console.log(
        `${id}, pay: ${collator.pay[id].toHuman().padStart(13)}, score: ${collator.points
          .toString()
          .padStart(4)} [bond: ${collator.bond.toHuman().padStart(14)}, total: ${collator.total
          .toHuman()
          .padStart(14)}][nominators: ${collator.nominators.length
          .toString()
          .padStart(4)}] - collected: ${
          Math.floor((uMOVRPerPoint * collator.points.toNumber()) / 10 ** 9) / 1000
        } MOVR - paid: ${
          Math.floor(Number(totalPaid / 10n ** precisionExp) / 10 ** (15 - Number(precisionExp))) /
          1000
        } MOVR`
      );
    }

    if (!argv.collator || id.toLowerCase() == argv.collator.toLowerCase()) {
      for (const staked of collator.nominators) {
        const nominator = nominators[staked.owner.toString()];
        const paid = nominator[id];
        console.log(
          `  - ${staked.owner.toString()}, pay: ${paid
            .toHuman()
            .padStart(14)}, staked: ${staked.amount.toHuman().padStart(14)}`
        );
      }
    }
  });

  console.log(`========= nominators`);

  records.forEach(({ event }, index) => {
    if (event.section == "parachainStaking" && event.method == "Rewarded") {
      const [acc, amount] = event.data as any;
      if (
        argv.nominator &&
        argv.nominator.toString().toLocaleLowerCase() == acc.toString().toLocaleLowerCase()
      ) {
        console.log(
          `${acc.toString()}, pay: ${amount.toHuman().padStart(13)} [${Object.keys(collators)
            .filter((id) =>
              collators[id].nominators
                .map((n) => n.owner.toString().toLowerCase())
                .includes(argv.nominator.toString().toLowerCase())
            )
            .map((id) => {
              return `collator ${id.toString().substr(0, 10)}...: ${collators[id].nominators
                .find((n) => n.owner.toString().toLowerCase())
                .amount.toHuman()
                .padStart(13)}`;
            })
            .join(" - ")}]`
        );
      }
    }
  });
  console.log(
    `\n#${blockNumber} Total : ${Number(total / 10n ** 15n) / 1000} MOVRs (${Object.keys(
      collators
    ).reduce((p, i) => p + collators[i].points.toNumber(), 0)} pts)`
  );
  api.disconnect();
};

main();
