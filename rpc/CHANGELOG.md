# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.8.0...celestia-rpc-v0.9.0) - 2025-01-21

### Added

- *(node-wasm, types)* [**breaking**] Add method to get blobs for wasm (#468)
- *(types,rpc)* [**breaking**] move TxConfig to celestia-rpc (#485)

### Other

- *(ci)* migrate toolchain action, parallelize (#503)

## [0.8.0](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.7.1...celestia-rpc-v0.8.0) - 2024-12-02

### Added

- *(proto,types,rpc)* [**breaking**] celestia node v0.20.4 upgrade ([#469](https://github.com/eigerco/lumina/pull/469))
- *(grpc, types, proto)* [**breaking**] Add tonic gRPC ([#454](https://github.com/eigerco/lumina/pull/454))

### Other

- *(proto,types,node,grpc)* [**breaking**] Use `tendermint-rs` instead of `celestia-tendermint-rs` fork ([#463](https://github.com/eigerco/lumina/pull/463))
- *(node-wasm)* Add integration tests for node-wasm ([#420](https://github.com/eigerco/lumina/pull/420))

## [0.7.1](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.7.0...celestia-rpc-v0.7.1) - 2024-11-07

### Other

- updated the following local packages: celestia-types

## [0.7.0](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.6.0...celestia-rpc-v0.7.0) - 2024-10-25

### Added

- *(types)* [**breaking**] Add versioned consts ([#412](https://github.com/eigerco/lumina/pull/412))
- *(types)* [**breaking**] add blob reconstruction from shares ([#450](https://github.com/eigerco/lumina/pull/450))
- *(types,rpc,node)* [**breaking**] refactor Share to work for parity and data ([#443](https://github.com/eigerco/lumina/pull/443))

## [0.6.0](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.5.0...celestia-rpc-v0.6.0) - 2024-10-11

### Fixed

- *(rpc)* [**breaking**] use correct blob type in state submit pay for blob ([#418](https://github.com/eigerco/lumina/pull/418))

### Other

- *(rpc)* Remove a workaround for go-jsonrpc not returning null ok status correctly ([#422](https://github.com/eigerco/lumina/pull/422))

## [0.5.0](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.4.1...celestia-rpc-v0.5.0) - 2024-10-03

### Added

- [**breaking**] shwap protocol updates ([#369](https://github.com/eigerco/lumina/pull/369))

## [0.4.1](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.4.0...celestia-rpc-v0.4.1) - 2024-09-24

### Added

- feat!(types,rpc): Add share, row, merkle proofs and share.GetRange ([#375](https://github.com/eigerco/lumina/pull/375))

## [0.4.0](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.3.0...celestia-rpc-v0.4.0) - 2024-08-22

### Added
- updating API for parity with celestia-node v0.15.0 ([#340](https://github.com/eigerco/lumina/pull/340))

## [0.3.0](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.2.0...celestia-rpc-v0.3.0) - 2024-08-13

### Added
- Add `#[track_caller]` on test utils and spawn utils ([#305](https://github.com/eigerco/lumina/pull/305))
- *(node/syncer)* [**breaking**] Implement backwards header sync ([#279](https://github.com/eigerco/lumina/pull/279))
- *(rpc)* Implement WASM Client ([#210](https://github.com/eigerco/lumina/pull/210))
- feat!(types): Add Blob::index field introduced in celestia 0.13 ([#274](https://github.com/eigerco/lumina/pull/274))

### Fixed
- *(rpc)* Increase max response size ([#336](https://github.com/eigerco/lumina/pull/336))
- new lints coming with 1.78 and 1.80-nightly ([#275](https://github.com/eigerco/lumina/pull/275))

### Other
- Upgrade jsonprsee to 0.24.2 ([#360](https://github.com/eigerco/lumina/pull/360))
- [**breaking**] Upgrade to nmt-rs 0.2.0 ([#322](https://github.com/eigerco/lumina/pull/322))
- [**breaking**] Upgrade dependencies but exclude the ones that are patched by risc0 ([#292](https://github.com/eigerco/lumina/pull/292))

## [0.2.0](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.1.1...celestia-rpc-v0.2.0) - 2024-04-18

### Added
- [**breaking**] Refactor RowId/SampleId/NamespacedDataId related API ([#236](https://github.com/eigerco/lumina/pull/236))
- feat!(node): Implement DASer ([#223](https://github.com/eigerco/lumina/pull/223))

### Other
- chore!(types,rpc): compatibility with celestia-node 0.13.1 ([#239](https://github.com/eigerco/lumina/pull/239))

## [0.1.1](https://github.com/eigerco/lumina/compare/celestia-rpc-v0.1.0...celestia-rpc-v0.1.1) - 2024-01-15

### Other
- add authors and homepage ([#180](https://github.com/eigerco/lumina/pull/180))

## [0.1.0](https://github.com/eigerco/lumina/releases/tag/celestia-rpc-v0.1.0) - 2024-01-12

### Added
- *(types)* Add `wasm-bindgen` feature flag ([#143](https://github.com/eigerco/lumina/pull/143))
- *(node)* Implement running node in browser ([#112](https://github.com/eigerco/lumina/pull/112))
- *(rpc)* align to celestia-node 0.12.0 ([#125](https://github.com/eigerco/lumina/pull/125))
- *(rpc)* create wrappers for jsonrpsee clients ([#114](https://github.com/eigerco/lumina/pull/114))
- *(node)* Implement Syncer ([#94](https://github.com/eigerco/lumina/pull/94))
- Improve verification and implement verification in Exchange client ([#85](https://github.com/eigerco/lumina/pull/85))
- add RPC calls for p2p module and tests for them ([#52](https://github.com/eigerco/lumina/pull/52))
- Implement initial architecture of node crate ([#42](https://github.com/eigerco/lumina/pull/42))
- *(fraud)* Add fraud proof trait and byzantine encoding fraud ([#32](https://github.com/eigerco/lumina/pull/32))
- Add State RPC and types ([#31](https://github.com/eigerco/lumina/pull/31))
- *(rpc)* Add all calls for Blob, Share, and Header ([#24](https://github.com/eigerco/lumina/pull/24))
- align namespaced shares deserialization with latest celestia  ([#20](https://github.com/eigerco/lumina/pull/20))
- serialize proof and commitment ([#19](https://github.com/eigerco/lumina/pull/19))
- *(rpc)* Create celestia-rpc crate and add integration tests ([#17](https://github.com/eigerco/lumina/pull/17))

### Fixed
- *(types)* fix the json representation of SubmitOptions ([#66](https://github.com/eigerco/lumina/pull/66))
- make celestia-rpc to compile in wasm32 target ([#46](https://github.com/eigerco/lumina/pull/46))

### Other
- add missing metadata to the toml files ([#170](https://github.com/eigerco/lumina/pull/170))
- document public api ([#161](https://github.com/eigerco/lumina/pull/161))
- add validation of EDS in tests ([#165](https://github.com/eigerco/lumina/pull/165))
- Make sure we run clippy for wasm and fix wasm build/lints ([#115](https://github.com/eigerco/lumina/pull/115))
- Migrate to nmt-rs of crates.io ([#144](https://github.com/eigerco/lumina/pull/144))
- Upgrade libp2p to v0.53.0 ([#126](https://github.com/eigerco/lumina/pull/126))
- update celestia node to 0.11.0-rc15 ([#101](https://github.com/eigerco/lumina/pull/101))
- trim the features of workspace dependencies ([#99](https://github.com/eigerco/lumina/pull/99))
- align to celestia node v0.11-rc ([#65](https://github.com/eigerco/lumina/pull/65))
- Migrate from `log` to `tracing` ([#55](https://github.com/eigerco/lumina/pull/55))
- update jsonrpsee to 0.20 ([#41](https://github.com/eigerco/lumina/pull/41))
