import type { DevModeContext } from "@moonwall/cli";

export const getPalletIndex = async (name: string, context: DevModeContext): Promise<number> => {
  const metadata = await context.polkadotJs().rpc.state.getMetadata();
  return metadata.asLatest.pallets
    .find(({ name: palletName }) => palletName.toString() === name)!
    .index.toNumber();
};
