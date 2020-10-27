#!/usr/bin/env bash

set -e
source $HOME/.cargo/env

echo "*** Initializing WASM build environment"

RUST_NIGHTLY_VERSION=$(cat rust-toolchain)

if [ -z ${WASM_BUILD_TOOLCHAIN+x} ]; then
	WASM_BUILD_TOOLCHAIN=$RUST_NIGHTLY_VERSION
fi

if [ -z $CI_PROJECT_NAME ] ; then
   rustup update $WASM_BUILD_TOOLCHAIN
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain $WASM_BUILD_TOOLCHAIN
