import { typesBundle } from "moonbeam-types-bundle";
import { ApiPromise, WsProvider } from "@polkadot/api";
import * as fs from "fs";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex } from "@polkadot/util";

const main = async () => {
  const wsProvider = new WsProvider("ws://localhost:56992");
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });

  const collatorAddress = "0x37006dd9d226425c901c5d7f9434c2fb8ac3f533";
  const parachainCollatorStorageKey = u8aToHex(
    u8aConcat(
      xxhashAsU8a("ParachainStaking", 128),
      xxhashAsU8a("CollatorState2", 128),
      xxhashAsU8a(collatorAddress, 64),
      collatorAddress
    )
  );

  // const startBlock = 1;
  const startBlock = 401000;
  const currentBlockHash = await polkadotApi.rpc.chain.getBlockHash(406800);
  let { block } = await polkadotApi.rpc.chain.getBlock(currentBlockHash);
  while (block.header.number.toNumber() > startBlock) {
    let calledStaking = false;
    const records = await polkadotApi.query.system.events.at(block.header.hash);

    block.extrinsics.forEach((extrinsic, index) => {
      const {
        method: { args, method, section },
      } = extrinsic;

      if (
        section == "parachainStaking" &&
        args.find((argv) => argv.toString().toLowerCase() == collatorAddress.toLowerCase())
      ) {
        console.log(
          `#${block.header.number.toString().padStart(5, " ")} ${section}.${method}(${args
            .map((a) => a.toString())
            .join(", ")})`
        );
        calledStaking = true;
      }

      if (section != "ethereum" || method != "transact") {
        return;
      }

      const events = records.filter(
        ({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index)
      );

      // Search if it is a transfer
      events.forEach(({ event }) => {
        const types = event.typeDef;
        if (
          event.section == "parachainStaking" &&
          event.data.find(
            (d) =>
              d.toString().toLowerCase() ==
              "0x37006DD9D226425C901c5d7F9434C2FB8aC3f533".toLowerCase()
          )
        ) {
          console.log(
            `#${block.header.number.toString().padStart(5, " ")} [Eth] ${event.section}.${
              event.method
            }(${event.data
              .map((data, index) => `${types[index].type}: ${data.toString()}`)
              .join(", ")})`
          );
          calledStaking = true;
        }
      });
    });
    const storage: any = await polkadotApi.rpc.state.getStorage.raw(
      parachainCollatorStorageKey,
      block.header.hash
    );

    if (calledStaking) {
      fs.writeFileSync(
        `block-${block.header.number.toString()}-storage.json`,
        JSON.stringify(polkadotApi.registry.createType("Collator2", storage), null, 2)
      );
    }

    block = (await polkadotApi.rpc.chain.getBlock(block.header.parentHash)).block;
  }
};

main();
