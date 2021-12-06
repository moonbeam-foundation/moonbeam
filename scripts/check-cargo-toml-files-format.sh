#!/bin/bash

find . -name "Cargo.toml" -not -path "*/target/*" -exec toml-sort {} \;

CMD="git diff --name-only"

stdbuf -oL $CMD | {
  while IFS= read -r line; do
    echo ║ $line
    if [[ "$line" == *"Cargo.toml" ]]; then 
      echo "Check fails: $line"
      OK="false"
      exit 1
    fi
  done
}
