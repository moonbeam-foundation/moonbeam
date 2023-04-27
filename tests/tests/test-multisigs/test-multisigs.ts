import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { blake2AsHex, createKeyMulti } from "@polkadot/util-crypto";
import {u8aToHex} from "@polkadot/util"
import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, ethan , ALITH_ADDRESS, DOROTHY_ADDRESS, CHARLETH_ADDRESS, BALTATHAR_ADDRESS} from "../../util/accounts";
import { expectOk } from "../../util/expect";

describeDevMoonbeam("Multisigs - perform multisigs operations", (context) => {
    const threshold = 2;
    const otherSignatories = [BALTATHAR_ADDRESS, CHARLETH_ADDRESS];
    let call: any;
    let encodedCall: any;
    let encodedCallHash: any;

    let encodedMultisigId = createKeyMulti([ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS],2);
    let multisigId = u8aToHex(encodedMultisigId.slice(0,20));

    it("Should create a multisig operation with asMulti", async function () {

        call = context.polkadotApi.tx.balances.transferKeepAlive(DOROTHY_ADDRESS, 20);
        encodedCall = call.method.toHex();
        encodedCallHash = blake2AsHex(encodedCall);
        const block = await context.createBlock(
            context.polkadotApi.tx.multisig.asMulti(threshold, otherSignatories, null, encodedCall, {})
            .signAsync(alith)
        );

        expect(block.result.successful).to.be.true;
    });

    it("should be able to cancel the multisig operation", async () => {

        console.log("CALL: ", encodedCall)
        console.log("HASH: ", encodedCallHash)

        const multisigInfo = await context.polkadotApi.query.multisig.multisigs(multisigId, encodedCallHash);
        console.log(multisigInfo.toHuman())
        
        /* const records = (await context.polkadotApi.query.system.events()) as any;
        const events = records.filter(
            ({ event }) => console.log(event.method)
        ); */
    });
});