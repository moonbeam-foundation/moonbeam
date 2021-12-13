#!/bin/bash

find . -name "Cargo.toml" -not -path "*/target/*" -exec toml-sort {} \;

CMD="git diff --name-only"

stdbuf -oL $CMD | {
  while IFS= read -r line; do
    echo ║ $line
    if [[ "$line" == *"Cargo.toml" ]]; then 
      echo "Check fails: $line"
      echo "Please run './scripts/toml-sort.sh' to format Cargo.toml files properly."
      exit 1
    fi
  done
}
