#!/bin/bash

CHAINS=(
  moonbase
  moonriver
  moonbeam
)

# params
RUNTIME_CHAIN_SPEC=$1

# Bump package version
if [[ $# -gt 0 ]]; then
  npm version --no-git-tag-version 0.$RUNTIME_CHAIN_SPEC.0
fi

# Install dependencies
npm install

# Generate typescript api code
npm run generate

# Manually fix BTreeSet issue
echo "Manually fix BTreeSet issue..."
for CHAIN in ${CHAINS[@]}; do
  sed -i -e 's/BTreeSet,/BTreeSet as BTreeSetType,/g' src/$CHAIN/interfaces/types-lookup.ts
  sed -i -e 's/BTreeSet<Bytes>/BTreeSetType<Bytes>/g' src/$CHAIN/interfaces/types-lookup.ts
done

# Build the package
npm run build
