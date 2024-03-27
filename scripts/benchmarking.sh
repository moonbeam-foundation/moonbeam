#!/usr/bin/env bash

# This script can be used for running moonbeam's benchmarks.
#
# The moonbeam binary is required to be compiled with --features=runtime-benchmarks
# in release mode.

set -e

BINARY="./target/release/moonbeam"
STEPS=50
REPEAT=20

if [[ ! -f "${BINARY}" ]]; then
    echo "binary '${BINARY}' does not exist."
    echo "ensure that the moonbeam binary is compiled with '--features=runtime-benchmarks' and in release mode."
    exit 1
fi

function help {
    echo "USAGE:"
    echo "  ${0} [<pallet> <benchmark>] [--check]"
    echo ""
    echo "EXAMPLES:"
    echo "  ${0}                       " "list all benchmarks and provide a selection to choose from" 
    echo "  ${0} --check               " "list all benchmarks and provide a selection to choose from, runs in 'check' mode (reduced steps and repetitions)" 
    echo "  ${0} foo bar               " "run a benchmark for pallet 'foo' and benchmark 'bar'" 
    echo "  ${0} foo bar --check       " "run a benchmark for pallet 'foo' and benchmark 'bar' in 'check' mode (reduced steps and repetitions)" 
    echo "  ${0} foo bar --all         " "run a benchmark for all pallets" 
    echo "  ${0} foo bar --all --check " "run a benchmark for all pallets in 'check' mode (reduced steps and repetitions)" 
}

function choose_and_bench {
    readarray -t options < <(${BINARY} benchmark pallet --list | sed 1d)
    options+=('EXIT')

    select opt in "${options[@]}"; do
        IFS=', ' read -ra parts <<< "${opt}"
        [[ "${opt}" == 'EXIT' ]] && exit 0
        
        bench "${parts[0]}" "${parts[1]}" "${1}"
        break
    done
}

function bench {
    OUTPUT=${4:-weights.rs}
    echo "benchmarking '${1}::${2}' --check=${3}, writing results to '${OUTPUT}'"

    # Check enabled
    if [[ "${3}" -eq 1 ]]; then
        STEPS=16
        REPEAT=1
    fi

    WASMTIME_BACKTRACE_DETAILS=1 ${BINARY} benchmark pallet \
        --execution=wasm \
        --wasm-execution=compiled \
        --pallet "${1}" \
        --extrinsic "${2}" \
        --steps "${STEPS}" \
        --repeat "${REPEAT}" \
        --header=./file_header.txt \
        --template=./benchmarking/frame-weight-template.hbs \
        --json-file raw.json \
        --output "${OUTPUT}"
}

echo -e "\e[33m**Warning**: The benchmarking script will overwrite the pallet_proxy.rs file in runtime/common/src/weights. This file was manually modified to include an additional DB Read for proxy.proxy. Make sure to add it again after running the benchmarking script.\e[0m"

if [[ "${@}" =~ "--help" ]]; then
    help
else
    CHECK=0
    if [[ "${@}" =~ "--check" ]]; then
        CHECK=1
        set -o noglob && set -- ${@/'--check'} && set +o noglob
    fi

    ALL=0
    if [[ "${@}" =~ "--all" ]]; then
        ALL=1
    fi

    if [[ "${ALL}" -eq 1 ]]; then
        mkdir -p weights/
        bench '*' '*' "${CHECK}" "weights/"
    elif [[ $# -ne 2 ]]; then
        choose_and_bench "${CHECK}"
    else
        bench "${1}" "${2}" "${CHECK}"
    fi
fi
