mkdir -p target/release/wbuild/moonbase-runtime/
mkdir -p test/tmp

# When we remove this step, re-add "runScripts": ["download-polkadot.sh"]," to moonwall config
cd test/tmp
wget https://opslayer-dev-artifacts.s3.us-east-2.amazonaws.com/bins/moonbeam/polkadot/1.3.0/polkadot
wget https://opslayer-dev-artifacts.s3.us-east-2.amazonaws.com/bins/moonbeam/polkadot/1.3.0/polkadot-execute-worker
wget https://opslayer-dev-artifacts.s3.us-east-2.amazonaws.com/bins/moonbeam/polkadot/1.3.0/polkadot-prepare-worker

chmod +x polkadot
chmod +x polkadot-execute-worker
chmod +x polkadot-prepare-worker

cd ..
pnpm install

## Generate old spec using latest published node, modify it, and generate raw spec
chmod uog+x tmp/moonbeam_rt
chmod uog+x ../target/release/moonbeam
tmp/moonbeam_rt build-spec --chain moonbase-local > tmp/moonbase-plain-spec.json
pnpm tsx scripts/modify-plain-specs.ts process tmp/moonbase-plain-spec.json tmp/moonbase-modified-spec.json
tmp/moonbeam_rt build-spec --chain tmp/moonbase-modified-spec.json --raw > tmp/moonbase-raw-spec.json
pnpm tsx scripts/preapprove-rt-rawspec.ts process tmp/moonbase-raw-spec.json tmp/moonbase-modified-raw-spec.json ../target/release/wbuild/moonbase-runtime/moonbase_runtime.compact.compressed.wasm
      