#!/usr/bin/env bash

# This script can be used for running moonbeam's benchmarks.
#
# The moonbeam binary is required to be compiled with --features=runtime-benchmarks
# in release mode.

set -e

BINARY="./target/release/moonbeam"

function help {
    echo "USAGE:"
    echo "  ${0} [<pallet> <extrinsic>]" 
}

function choose_and_bench {
    readarray -t options < <(${BINARY} benchmark --list | sed 1d)
    options+=('EXIT')

    select opt in "${options[@]}"; do
        IFS=', ' read -ra parts <<< "${opt}"
        [[ "${opt}" == 'EXIT' ]] && exit 0
        
        bench "${parts[0]}" "${parts[1]}" "${1}"
        break
    done
}

function bench {
    echo "benchmarking '${1}::${2}' --check=${3}"
    STEPS=32
    REPEAT=64
    if [[ "${check}" -eq 1 ]]; then
        STEPS=16
        REPEAT=1
    fi

    WASMTIME_BACKTRACE_DETAILS=1 ${BINARY} benchmark \
        --chain dev \
        --execution=wasm \
        --wasm-execution=compiled \
        --pallet "${1}" \
        --extrinsic "${2}" \
        --steps 32 \
        --repeat 64 \
        --template=./benchmarking/frame-weight-template.hbs \
        --record-proof \
        --json-file raw.json \
        --output weights.rs
}

if [[ "${@}" =~ "--help" ]]; then
    help
else
    CHECK=0
    args="${@}"
    if [[ "${args}" =~ "--check" ]]; then
        CHECK=1
        set -o noglob && set -- ${args/'--check'} && set +o noglob
    fi

    if [[ $# -ne 2 ]]; then
        choose_and_bench "${CHECK}"
    else
        bench "${1}" "${2}" "${CHECK}"
    fi
fi
