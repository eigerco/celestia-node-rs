# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/eigerco/lumina/compare/celestia-grpc-v0.1.0...celestia-grpc-v0.2.0) - 2025-01-16

### Added

- *(grpc)* [**breaking**] add wasm support and transaction client (#474)

### Other

- *(ci)* migrate toolchain action, parallelize (#503)
- *(grpc)* Increase sleep before blob submission validation to reduce test flakyness (#481)

## [0.1.0](https://github.com/eigerco/lumina/releases/tag/celestia-grpc-v0.1.0) - 2024-12-02

### Added

- *(proto,types,rpc)* [**breaking**] celestia node v0.20.4 upgrade ([#469](https://github.com/eigerco/lumina/pull/469))
- *(grpc, types, proto)* [**breaking**] Add tonic gRPC ([#454](https://github.com/eigerco/lumina/pull/454))

### Other

- *(proto,types,node,grpc)* [**breaking**] Use `tendermint-rs` instead of `celestia-tendermint-rs` fork ([#463](https://github.com/eigerco/lumina/pull/463))
