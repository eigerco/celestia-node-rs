# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/eigerco/lumina/compare/lumina-cli-v0.5.2...lumina-cli-v0.6.0) - 2025-01-21

### Added

- Add remaining node types for wasm (#476)
- *(cli)* Add `in-memory-store` and `pruning-delay` parameters (#490)
- *(node)* [**breaking**] Implement `NodeBuilder` and remove `NodeConfig` (#472)

### Fixed

- *(cli)* align with dah javascript breaking changes (#501)

### Other

- *(ci)* migrate toolchain action, parallelize (#503)
- *(node,node-wasm)* [**breaking**] Rename `syncing_window` to `sampling_window` (#477)

## [0.5.2](https://github.com/eigerco/lumina/compare/lumina-cli-v0.5.1...lumina-cli-v0.5.2) - 2024-12-02

### Added

- *(grpc, types, proto)* [**breaking**] Add tonic gRPC ([#454](https://github.com/eigerco/lumina/pull/454))

### Other

- *(node-wasm)* Add integration tests for node-wasm ([#420](https://github.com/eigerco/lumina/pull/420))

## [0.5.1](https://github.com/eigerco/lumina/compare/lumina-cli-v0.5.0...lumina-cli-v0.5.1) - 2024-11-07

### Other

- updated the following local packages: celestia-types, lumina-node

## [0.5.0](https://github.com/eigerco/lumina/compare/lumina-cli-v0.4.1...lumina-cli-v0.5.0) - 2024-10-25

### Added

- *(node,node-wasm)* [**breaking**] Allow customising syncing window size ([#442](https://github.com/eigerco/lumina/pull/442))

## [0.4.1](https://github.com/eigerco/lumina/compare/lumina-cli-v0.4.0...lumina-cli-v0.4.1) - 2024-10-11

### Added

- setup local demo page with webpack ([#388](https://github.com/eigerco/lumina/pull/388))

## [0.4.0](https://github.com/eigerco/lumina/compare/lumina-cli-v0.3.1...lumina-cli-v0.4.0) - 2024-10-03

### Added

- *(node,node-wasm)* [**breaking**] Integrate graceful shutdown in WASM ([#396](https://github.com/eigerco/lumina/pull/396))

## [0.3.1](https://github.com/eigerco/lumina/compare/lumina-cli-v0.3.0...lumina-cli-v0.3.1) - 2024-09-24

### Other

- update Cargo.lock dependencies

## [0.3.0](https://github.com/eigerco/lumina/compare/lumina-cli-v0.2.0...lumina-cli-v0.3.0) - 2024-08-13

### Added
- feat!(node): make syncer batch sizes configurable ([#327](https://github.com/eigerco/lumina/pull/327))
- add support for dnsaddr resolving in browser ([#319](https://github.com/eigerco/lumina/pull/319))
- *(node)* Generate syncing related events ([#312](https://github.com/eigerco/lumina/pull/312))
- *(wasm)* Run Lumina in a Shared Worker ([#265](https://github.com/eigerco/lumina/pull/265))
- *(node/syncer)* [**breaking**] Implement backwards header sync ([#279](https://github.com/eigerco/lumina/pull/279))
- *(node)* Generate events for data sampling that can be used by front-end ([#276](https://github.com/eigerco/lumina/pull/276))

### Fixed
- *(node-wasm)* require serving and providing worker script ([#313](https://github.com/eigerco/lumina/pull/313))

### Other
- remove genesis hash from node configuration ([#316](https://github.com/eigerco/lumina/pull/316))
- [**breaking**] Upgrade dependencies but exclude the ones that are patched by risc0 ([#292](https://github.com/eigerco/lumina/pull/292))

## [0.2.0](https://github.com/eigerco/lumina/compare/lumina-cli-v0.1.0...lumina-cli-v0.2.0) - 2024-04-18

### Added
- *(cli)* [**breaking**] Replace sled stores with redb stores ([#267](https://github.com/eigerco/lumina/pull/267))
- feat!(node): use generic blockstore in node ([#218](https://github.com/eigerco/lumina/pull/218))

### Other
- Add note about WebTransport requiring Secure Context ([#211](https://github.com/eigerco/lumina/pull/211))

## [0.1.0](https://github.com/eigerco/lumina/releases/tag/lumina-cli-v0.1.0) - 2024-01-12

### Other
- add missing metadata to the toml files ([#170](https://github.com/eigerco/lumina/pull/170))
- document public api ([#161](https://github.com/eigerco/lumina/pull/161))
- error message for missing token and cleanups ([#168](https://github.com/eigerco/lumina/pull/168))
- rename the node implementation to Lumina ([#156](https://github.com/eigerco/lumina/pull/156))
