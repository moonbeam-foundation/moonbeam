# Embedded Spec Files

This directory contains chain specs for well-known public networks.

## Context

The Moonbase node is designed to support multiple networks including Moonbase Alpha, MoonRiver (Kusama) and Moonbeam (Polkadot). Some of these networks are already live and others are planned.

In order to support multiple networks with the same binary, Moonbase relies on a chain sepcification to know which network to sync. Rather than require node operators to obtain spec files seperately, it is convenient to "bake" specs for popular networks into the node.

Because Moonbase networks are parachains, each network instance requires both a parachain and a relay chain spec.

## Which specs are included?

* Moonbase Alpha V4 - live
* MoonRock - Potential future deployment to Rococo
* MoonRiver - Future Kusama Deployment
* Moonbeam - Future Polkadot deployment
