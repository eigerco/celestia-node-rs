# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0](https://github.com/eigerco/lumina/compare/celestia-proto-v0.6.0...celestia-proto-v0.7.0) - 2025-01-21

### Added

- *(grpc)* [**breaking**] add wasm support and transaction client (#474)

### Other

- *(ci)* migrate toolchain action, parallelize (#503)
- [**breaking**] Add notes about Celestia's Tendermint modifications (#471)

## [0.6.0](https://github.com/eigerco/lumina/compare/celestia-proto-v0.5.0...celestia-proto-v0.6.0) - 2024-12-02

### Added

- *(proto,types,rpc)* [**breaking**] celestia node v0.20.4 upgrade ([#469](https://github.com/eigerco/lumina/pull/469))
- *(grpc, types, proto)* [**breaking**] Add tonic gRPC ([#454](https://github.com/eigerco/lumina/pull/454))
- *(proto)* [**breaking**] update celestia-app and node proto definitios ([#459](https://github.com/eigerco/lumina/pull/459))

### Other

- *(proto,types,node,grpc)* [**breaking**] Use `tendermint-rs` instead of `celestia-tendermint-rs` fork ([#463](https://github.com/eigerco/lumina/pull/463))

## [0.5.0](https://github.com/eigerco/lumina/compare/celestia-proto-v0.4.1...celestia-proto-v0.5.0) - 2024-10-25

### Added

- *(types,rpc,node)* [**breaking**] refactor Share to work for parity and data ([#443](https://github.com/eigerco/lumina/pull/443))

## [0.4.1](https://github.com/eigerco/lumina/compare/celestia-proto-v0.4.0...celestia-proto-v0.4.1) - 2024-10-11

### Fixed

- *(proto)* handle nulls due to changes in TxResponse ([#421](https://github.com/eigerco/lumina/pull/421))

### Other

- *(types)* Use protox instead of requiring protoc when building ([#402](https://github.com/eigerco/lumina/pull/402))

## [0.4.0](https://github.com/eigerco/lumina/compare/celestia-proto-v0.3.1...celestia-proto-v0.4.0) - 2024-10-03

### Added

- [**breaking**] shwap protocol updates ([#369](https://github.com/eigerco/lumina/pull/369))

## [0.3.1](https://github.com/eigerco/lumina/compare/celestia-proto-v0.3.0...celestia-proto-v0.3.1) - 2024-09-24

### Other

- update Cargo.toml dependencies

## [0.3.0](https://github.com/eigerco/lumina/compare/celestia-proto-v0.2.0...celestia-proto-v0.3.0) - 2024-08-13

### Fixed
- *(types)* [**breaking**] Align byzantine fraud proofs with Go's implementation ([#338](https://github.com/eigerco/lumina/pull/338))

### Other
- [**breaking**] Upgrade dependencies but exclude the ones that are patched by risc0 ([#292](https://github.com/eigerco/lumina/pull/292))

## [0.2.0](https://github.com/eigerco/lumina/compare/celestia-proto-v0.1.1...celestia-proto-v0.2.0) - 2024-04-18

### Added
- feat!(types): Align with Shwap spec ([#232](https://github.com/eigerco/lumina/pull/232))
- feat!(node): Implement DASer ([#223](https://github.com/eigerco/lumina/pull/223))

## [0.1.1](https://github.com/eigerco/lumina/compare/celestia-proto-v0.1.0...celestia-proto-v0.1.1) - 2024-01-15

### Other
- add authors and homepage ([#180](https://github.com/eigerco/lumina/pull/180))

## [0.1.0](https://github.com/eigerco/lumina/releases/tag/celestia-proto-v0.1.0) - 2024-01-12

### Added
- *(node)* Add shwap data types ([#169](https://github.com/eigerco/lumina/pull/169))
- *(node)* Implement persistent header storage in browser using IndexedDB ([#102](https://github.com/eigerco/lumina/pull/102))
- Improve verification and implement verification in Exchange client ([#85](https://github.com/eigerco/lumina/pull/85))
- *(proto)* add cosmos Tx and celestia MsgPayForBlobs ([#75](https://github.com/eigerco/lumina/pull/75))
- Implement initial architecture of node crate ([#42](https://github.com/eigerco/lumina/pull/42))
- *(fraud)* Add fraud proof trait and byzantine encoding fraud ([#32](https://github.com/eigerco/lumina/pull/32))
- Add State RPC and types ([#31](https://github.com/eigerco/lumina/pull/31))
- align namespaced shares deserialization with latest celestia  ([#20](https://github.com/eigerco/lumina/pull/20))
- *(rpc)* Create celestia-rpc crate and add integration tests ([#17](https://github.com/eigerco/lumina/pull/17))
- *(proto)* add `empty_as_none` serializer ([#18](https://github.com/eigerco/lumina/pull/18))
- add NamespacedShares type ([#7](https://github.com/eigerco/lumina/pull/7))
- *(proto)* vendor protobuf definitions

### Fixed
- temporary backward compatibility for json proofs ([#96](https://github.com/eigerco/lumina/pull/96))

### Other
- add missing metadata to the toml files ([#170](https://github.com/eigerco/lumina/pull/170))
- document public api ([#161](https://github.com/eigerco/lumina/pull/161))
- Pin celestia-app version to 1.4 when updating protobuf, update protobufs ([#175](https://github.com/eigerco/lumina/pull/175))
- update celestia node to 0.11.0-rc15 ([#101](https://github.com/eigerco/lumina/pull/101))
- Update protobuf definitions ([#40](https://github.com/eigerco/lumina/pull/40))
- align to celestia node v0.11-rc ([#65](https://github.com/eigerco/lumina/pull/65))
- fix format ([#34](https://github.com/eigerco/lumina/pull/34))
- vendor cosmos protobuf definitions ([#30](https://github.com/eigerco/lumina/pull/30))
- *(license)* Set Apache 2.0 license ([#6](https://github.com/eigerco/lumina/pull/6))
- Implement Celestia types
- initial commit
