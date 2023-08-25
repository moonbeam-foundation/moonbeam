#!/bin/bash

if [ -z "$2" ]; then
  echo "Usage: $0 <from version> <to version>"
  echo "Ex: $0 0.8.3 0.8.5"
  exit 1
fi

FROM=$1
TO=$2
sed -i "s/moonbeam-foundation\/moonbeam:v$FROM/moonbeam-foundation\/moonbeam:v$TO/" README.md
sed -i "s/^version = '$FROM'$/version = '$TO'/" node/Cargo.toml
sed -i "s/^version = '$FROM'$/version = '$TO'/" node/cli/Cargo.toml
sed -i "s/^version = '$FROM'$/version = '$TO'/" node/cli-opt/Cargo.toml
sed -i "s/^version = '$FROM'$/version = '$TO'/" node/service/Cargo.toml

sed -i "s/^version = '$FROM'$/version = '$TO'/" runtime/moonbase/Cargo.toml
sed -i "s/^version = '$FROM'$/version = '$TO'/" runtime/moonriver/Cargo.toml
sed -i "s/^version = '$FROM'$/version = '$TO'/" runtime/moonbeam/Cargo.toml

cargo build --release