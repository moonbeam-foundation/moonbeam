#!/bin/bash

# Runs all benchmarks for all pallets, for a given runtime, provided by $1
# Should be run on a reference machine to gain accurate benchmarks
# current reference machine: https://github.com/paritytech/substrate/pull/5848

runtime="${1:-moonbase}"
output="common"

echo "[+] Compiling benchmarks..."
# cargo build --release --locked --features=runtime-benchmarks
cargo build --release --features=runtime-benchmarks

# Load all pallet names in an array.
PALLETS=($(
  ./target/release/moonbeam benchmark pallet --list --chain="${runtime}-dev" |\
    tail -n+2 |\
    cut -d',' -f1 |\
    sort |\
    uniq
))

echo "[+] Benchmarking ${#PALLETS[@]} pallets for runtime $runtime"

# Define the error file.
ERR_FILE="benchmarking_errors.txt"
# Delete the error file before each run.
rm -f $ERR_FILE

# Benchmark each pallet.
for PALLET in "${PALLETS[@]}"; do
  echo "[+] Benchmarking $PALLET for $runtime";

  output_file=""
  if [[ $PALLET == *"::"* ]]; then
    # translates e.g. "pallet_foo::bar" to "pallet_foo_bar"
    output_file="${PALLET//::/_}.rs"
  fi

  OUTPUT=$(
    ./target/release/moonbeam benchmark pallet \
    --chain="${runtime}-dev" \
    --steps=50 \
    --repeat=20 \
    --pallet="$PALLET" \
    --extrinsic="*" \
    --wasm-execution=compiled \
    --header=./file_header.txt \
    --output="./runtime/${output}/src/weights/${output_file}" 2>&1
  )
  if [ $? -ne 0 ]; then
    echo "$OUTPUT" >> "$ERR_FILE"
    echo "[-] Failed to benchmark $PALLET. Error written to $ERR_FILE; continuing..."
  fi
done

# Check if the error file exists.
if [ -f "$ERR_FILE" ]; then
  echo "[-] Some benchmarks failed. See: $ERR_FILE"
else
  echo "[+] All benchmarks passed."
fi
