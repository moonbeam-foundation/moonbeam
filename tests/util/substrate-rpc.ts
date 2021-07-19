import { AddressOrPair } from "@polkadot/api/types";

export const sendSubstrateTxAndListenInBlockEvents = async (
  context,
  sender: AddressOrPair,
  polkadotCall,
  inBlockCallback: (events: any[]) => void
): Promise<void> => {
  await new Promise(async (resolve) => {
    const unsub = await polkadotCall.signAndSend(sender, ({ events = [], status }) => {
      if (status.isInBlock) {
        inBlockCallback(events);
        unsub();
        resolve(null);
      }
    });

    await context.createBlock();
  });
};
