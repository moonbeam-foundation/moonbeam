#!/bin/bash
# Build the node in --release mode but with debug symbols to allow debugger breakpoints to work.

RUSTFLAGS=-g cargo build --release