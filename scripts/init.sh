#!/usr/bin/env bash

set -e
source $HOME/.cargo/env

# This script is basically a noop no that rust-toolchain is
# a full toml file. This can probably be removed, but I'm keeping
# it for now in case the `source` line of the existence of the file
# is necessary elsewhere
