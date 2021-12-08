#!/bin/bash

# From the workspace directory, run :
# ./scripts/toml-sort.sh
# to format all Cargo.toml files, and 
# ./scripts/toml-sort.sh --check
# to only check the formatting.

find . -name "Cargo.toml" -not -path "*/target/*" -exec toml-sort {} $@ \;