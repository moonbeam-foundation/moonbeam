import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { CHARLETH_ADDRESS, alith } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import {
  sovereignAccountOfSibling,
  injectEncodedHrmpMessageAndSeal,
} from "../../../../helpers/xcm.js";
import { parseEther } from "ethers";

export const ERC20_TOTAL_SUPPLY = 1_000_000_000n;

describeSuite({
  id: "D014032",
  title: "Mock ERC20 <> XCM - Test wrong size of GeneralKey data field",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let polkadotJs: ApiPromise;
    const paraId = 888;

    beforeEach(async function () {
      polkadotJs = context.polkadotJs();
    });

    it({
      id: "T01",
      title: "Incoming ERC20 transfer should be ignored if data < 32 bytes length",
      test: async function () {
        const paraSovereign = sovereignAccountOfSibling(context, paraId);

        // Deploy contract
        const { contractAddress, status } = await context.deployContract!("ERC20ExcessGas", {
          args: ["ERC20", "20S", paraSovereign, ERC20_TOTAL_SUPPLY],
        });
        expect(status).eq("success");

        // Send some native tokens to the sovereign account of paraId (to pay fees)
        await polkadotJs.tx.balances
          .transferAllowDeath(paraSovereign, parseEther("1"))
          .signAndSend(alith);
        await context.createBlock();

        // Original GeneralKey to properly override the gas_limit value:
        // b'gas_limit:' + 300000(little endian) + zeros padding
        //
        // GeneralKey: {
        //  data: [
        //      103, 97, 115, 95, 108, 105, 109, 105, 116, 58, 224, 147, 4, 0, 0, 0, 0, 0,
        //      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //    ],
        //    length: 32,
        // }

        // This encoded message contains WithdrawAsset, ClearOrigin, BuyExecution, DepositAsset
        // instructions and a data field inside GeneralKey of 11 bytes, so it should be ignored.
        const wrongGeneralKeyMessage = [
          0, 3, 16, 0, 8, 0, 0, 1, 4, 3, 0, 19, 0, 0, 138, 93, 120, 69, 99, 1, 0, 0, 3, 4, 48, 3, 0,
          92, 195, 7, 38, 138, 19, 147, 171, 154, 118, 74, 32, 218, 206, 132, 138, 184, 39, 92, 70,
          6, 4, 103, 97, 115, 95, 108, 105, 109, 105, 116, 58, 224, 147, 0, 2, 9, 61, 0, 10, 19, 0,
          0, 1, 4, 3, 0, 19, 0, 0, 138, 93, 120, 69, 99, 1, 0, 13, 1, 2, 8, 0, 1, 3, 0, 121, 141,
          75, 169, 186, 240, 6, 78, 193, 158, 180, 240, 161, 164, 87, 133, 174, 157, 109, 252,
        ];

        // Mock the reception of the encoded xcm message
        const result = await injectEncodedHrmpMessageAndSeal(
          context,
          paraId,
          wrongGeneralKeyMessage
        );

        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.system.ExtrinsicSuccess.is(event)
        );

        // Check the block was produced successfully
        expect(events.length).toBeGreaterThanOrEqual(1);
        expect(result.block.duration).toBeGreaterThanOrEqual(1);
        expect(result.block.hash).to.not.be.undefined;
        expect(result.block.proofSize).toBeGreaterThanOrEqual(7000n);

        // Charleth should have NOT received the tokens
        expect(
          await context.readContract!({
            contractName: "ERC20ExcessGas",
            contractAddress: contractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [CHARLETH_ADDRESS],
          })
        ).equals(0n);
      },
    });

    it({
      id: "T02",
      title: "Incoming ERC20 transfer should be ignored if data < 10 bytes length",
      test: async function () {
        const paraSovereign = sovereignAccountOfSibling(context, paraId);

        // Deploy contract
        const { contractAddress, status } = await context.deployContract!("ERC20ExcessGas", {
          args: ["ERC20", "20S", paraSovereign, ERC20_TOTAL_SUPPLY],
        });
        expect(status).eq("success");

        // Send some native tokens to the sovereign account of paraId (to pay fees)
        await polkadotJs.tx.balances
          .transferAllowDeath(paraSovereign, parseEther("1"))
          .signAndSend(alith);
        await context.createBlock();

        // Original GeneralKey to properly override the gas_limit value:
        // b'gas_limit:' + 300000(little endian) + zeros padding
        //
        // GeneralKey: {
        //  data: [
        //      103, 97, 115, 95, 108, 105, 109, 105, 116, 58, 224, 147, 4, 0, 0, 0, 0, 0,
        //      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        //    ],
        //    length: 32,
        // }

        // This encoded message contains WithdrawAsset, ClearOrigin, BuyExecution, DepositAsset
        // instructions and a data field inside GeneralKey of only 5 bytes, so it should be ignored.
        const wrongGeneralKeyMessage = [
          0, 3, 16, 0, 8, 0, 0, 1, 4, 3, 0, 19, 0, 0, 138, 93, 120, 69, 99, 1, 0, 0, 3, 4, 48, 3, 0,
          92, 195, 7, 38, 138, 19, 147, 171, 154, 118, 74, 32, 218, 206, 132, 138, 184, 39, 92, 70,
          6, 4, 103, 97, 115, 95, 108, 0, 2, 9, 61, 0, 10, 19, 0, 0, 1, 4, 3, 0, 19, 0, 0, 138, 93,
          120, 69, 99, 1, 0, 13, 1, 2, 8, 0, 1, 3, 0, 121, 141, 75, 169, 186, 240, 6, 78, 193, 158,
          180, 240, 161, 164, 87, 133, 174, 157, 109, 252,
        ];

        // Mock the reception of the encoded xcm message
        const result = await injectEncodedHrmpMessageAndSeal(
          context,
          paraId,
          wrongGeneralKeyMessage
        );

        const events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.system.ExtrinsicSuccess.is(event)
        );

        // Check the block was produced successfully
        expect(events.length).toBeGreaterThanOrEqual(1);
        expect(result.block.duration).toBeGreaterThanOrEqual(1);
        expect(result.block.hash).to.not.be.undefined;
        expect(result.block.proofSize).toBeGreaterThanOrEqual(7000n);

        // Charleth should have NOT received the tokens
        expect(
          await context.readContract!({
            contractName: "ERC20ExcessGas",
            contractAddress: contractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [CHARLETH_ADDRESS],
          })
        ).equals(0n);
      },
    });
  },
});
