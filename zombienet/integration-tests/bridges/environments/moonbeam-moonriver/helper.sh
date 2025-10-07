#!/bin/bash


function wait_for_process() {
  if [[ $(uname -s) == "Darwin" ]]; then
    wait "$@";
  else
    wait "wait -n $@"
  fi
}