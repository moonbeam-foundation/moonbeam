#!/bin/bash

# Runs all benchmarks for all pallets, for a given runtime, provided by $1
# Should be run on a reference machine to gain accurate benchmarks
# current reference machine: https://github.com/paritytech/substrate/pull/5848

runtime="${1:-moonbase}"
output="${runtime}"
profile="${2:-production}"

echo "[+] Compiling benchmarks with $profile profile... (this will take a while)"
cargo build --profile=$profile --locked --features=runtime-benchmarks

# Load all pallet names in an array.
  PALLETS=($(
    ./target/${profile}/moonbeam benchmark pallet \
      --list \
      --runtime="./target/${profile}/wbuild/${runtime}-runtime/${runtime}_runtime.wasm" \
      --genesis-builder=runtime \
      --genesis-builder-preset=development |\
    tail -n+2 |\
    cut -d',' -f1 |\
    sort |\
    uniq
  ))

  echo "[+] Benchmarking ${#PALLETS[@]} pallets for runtime $runtime with $profile profile"

# Define the error file.
ERR_FILE="benchmarking_errors.txt"
# Delete the error file before each run.
rm -f $ERR_FILE

# Install frame-omni-bencher if not already installed
if ! frame-omni-bencher --version > /dev/null 2>&1; then
  echo "[+] Installing frame-omni-bencher"
  cargo install frame-omni-bencher --profile=production
fi

# Benchmark each pallet.
for PALLET in "${PALLETS[@]}"; do
  echo "[+] Benchmarking $PALLET for $runtime";

  output_file=""
  if [[ $PALLET == *"::"* ]]; then
    # translates e.g. "pallet_foo::bar" to "pallet_foo_bar"
    output_file="${PALLET//::/_}.rs"
  fi

  OUTPUT=$(
    frame-omni-bencher v1 benchmark pallet \
      --runtime="./target/${profile}/wbuild/${runtime}-runtime/${runtime}_runtime.wasm" \
      --genesis-builder=runtime \
      --genesis-builder-preset=development \
      --pallet="$PALLET" \
      --extrinsic="*" \
      --steps=50 \
      --repeat=20 \
      --wasm-execution=compiled \
      --header=./file_header.txt \
      --template="./benchmarking/frame-weight-template.hbs" \
      --output="./runtime/${output}/src/weights" 2>&1
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
