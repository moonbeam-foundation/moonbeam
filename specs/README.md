# Embedded Spec Files

This directory contains chain specs for well-known public networks.

## Context

The Moonbase node is designed to support multiple networks including Moonbase Alpha, MoonRiver
(Kusama) and Moonbeam (Polkadot). Some of these networks are already live and others are planned.

In order to support multiple networks with the same binary, Moonbase relies on a chain specification
to know which network to sync. Rather than require node operators to obtain spec files separately,
it is convenient to "bake" specs for popular networks into the node.

## Which specs will come pre-baked?

- Moonbase Stage V6 - internal
- Moonbase Alpha V6 - live
- MoonRock - Potential future deployment to Rococo
- MoonRiver - Future Kusama Deployment
- Moonbeam - Future Polkadot deployment

## Relay chain specs

Because Moonbase networks are parachains, each network instance requires both a parachain and a
relay chain spec. For popular relay chains like kusama and polkadot, we rely on the specs being
already included with Polkadot. For smaller relay chains, like the one that exists solely to support
moonbase alpha, we also bake the relay spec into the moonbase binary.
