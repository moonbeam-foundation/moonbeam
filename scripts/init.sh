#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ -z ${WASM_BUILD_TOOLCHAIN+x} ]; then
	WASM_BUILD_TOOLCHAIN=nightly
fi

if [ -z $CI_PROJECT_NAME ] ; then
   rustup update $WASM_BUILD_TOOLCHAIN
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain $WASM_BUILD_TOOLCHAIN
