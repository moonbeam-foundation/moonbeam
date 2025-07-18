const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';

let depth = 256;
let call = api.tx.system.remark("foo");
for (let i = 0; i < depth; i++) {
  call = api.tx.utility.batch([call]);
}

console.log(`call with depth: ${depth}: ${call.toU8a().length} bytes`);

await call.signAndSend(ALICE);