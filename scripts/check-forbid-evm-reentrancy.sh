#!/bin/bash

while IFS= read -r path
do
    echo "$path"
    while IFS= read -r line
    do
        matches=$(echo "$line" | awk '(/^pallet-evm = / || /^pallet-ethereum = /) && !/reentrancy/');
        if [ ! -z "$matches" ]; then
            echo "Check fails: $line"
            echo "Please add 'forbid-evm-reentrancy' feature to 'pallet-evm' and 'pallet-ethereum'."
            exit 1
        fi
    done < "$path"
done < <(find . -name "Cargo.toml" -not -path "*/target/*" -not -path "*/build/*")