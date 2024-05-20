# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/eigerco/lumina/compare/lumina-node-v0.2.0...lumina-node-v0.3.0) - 2024-05-20

### Added
- *(node)* Generate events for data sampling that can be used by front-end ([#276](https://github.com/eigerco/lumina/pull/276))
- *(node/daser)* [**breaking**] Implement backward sampling and sampling window ([#269](https://github.com/eigerco/lumina/pull/269))

### Fixed
- new lints coming with 1.78 and 1.80-nightly ([#275](https://github.com/eigerco/lumina/pull/275))

### Other
- *(node)* Replace `instant` crate with `web-time` ([#280](https://github.com/eigerco/lumina/pull/280))

## [0.2.0](https://github.com/eigerco/lumina/compare/lumina-node-v0.1.1...lumina-node-v0.2.0) - 2024-04-18

### Added
- *(cli)* [**breaking**] Replace sled stores with redb stores ([#267](https://github.com/eigerco/lumina/pull/267))
- *(node)* Implement `RedbStore` ([#266](https://github.com/eigerco/lumina/pull/266))
- *(node/p2p)* Relax internal `Send` bounds ([#260](https://github.com/eigerco/lumina/pull/260))
- [**breaking**] Refactor RowId/SampleId/NamespacedDataId related API ([#236](https://github.com/eigerco/lumina/pull/236))
- Expose get_sampling_metadata in node and node-wasm ([#234](https://github.com/eigerco/lumina/pull/234))
- *(node)* implement fraud-sub and services stopping on valid befp ([#233](https://github.com/eigerco/lumina/pull/233))
- *(types)* add encoding check when verifying befp ([#231](https://github.com/eigerco/lumina/pull/231))
- feat!(node): Implement DASer ([#223](https://github.com/eigerco/lumina/pull/223))
- *(blockstore)* add IndexedDb blockstore ([#221](https://github.com/eigerco/lumina/pull/221))
- feat!(node): use generic blockstore in node ([#218](https://github.com/eigerco/lumina/pull/218))
- *(node)* Extend header Store for use with DAS-er ([#209](https://github.com/eigerco/lumina/pull/209))
- *(node)* Integrate bitswap protocol for shwap ([#202](https://github.com/eigerco/lumina/pull/202))
- *(lumina-node)* update the bootstrap peers for testnets ([#184](https://github.com/eigerco/lumina/pull/184))

### Fixed
- fix!(node/sled_store): Use `transaction` when more than one value get read ([#230](https://github.com/eigerco/lumina/pull/230))

### Other
- *(node)* [**breaking**] Remove sled store implementation ([#268](https://github.com/eigerco/lumina/pull/268))
- *(node)* Upgrade blockstore, beetswap, and leopard-codec ([#264](https://github.com/eigerco/lumina/pull/264))
- *(node)* Upgrade blockstore and beetswap ([#259](https://github.com/eigerco/lumina/pull/259))
- *(node)* minor cleanup of `parse_request` ([#258](https://github.com/eigerco/lumina/pull/258))
- *(node)* fix unused warnings on HeaderRequestExt ([#220](https://github.com/eigerco/lumina/pull/220))
- *(node)* Replace unmaintained tempdir and outdated quinn ([#214](https://github.com/eigerco/lumina/pull/214))
- chore!(types): Shwap API changes for consistency  ([#212](https://github.com/eigerco/lumina/pull/212))
- *(node)* Move p2p related files in p2p directory ([#208](https://github.com/eigerco/lumina/pull/208))
- Update libp2p to 0.53.2 ([#203](https://github.com/eigerco/lumina/pull/203))

## [0.1.1](https://github.com/eigerco/lumina/compare/lumina-node-v0.1.0...lumina-node-v0.1.1) - 2024-01-15

### Other
- add authors and homepage ([#180](https://github.com/eigerco/lumina/pull/180))

## [0.1.0](https://github.com/eigerco/lumina/releases/tag/lumina-node-v0.1.0) - 2024-01-12

### Added
- Return bootstrap peers as iterator and filter relevant ones ([#147](https://github.com/eigerco/lumina/pull/147))
- Bootstrap more aggresively when too few peers ([#146](https://github.com/eigerco/lumina/pull/146))
- *(types)* Add `wasm-bindgen` feature flag ([#143](https://github.com/eigerco/lumina/pull/143))
- *(node)* Implement sessions ([#130](https://github.com/eigerco/lumina/pull/130))
- *(node)* Implement running node in browser ([#112](https://github.com/eigerco/lumina/pull/112))
- *(node)* Use Kademlia bootstrap to recover connections and refresh routing table ([#120](https://github.com/eigerco/lumina/pull/120))
- *(rpc)* create wrappers for jsonrpsee clients ([#114](https://github.com/eigerco/lumina/pull/114))
- *(node)* Implement persistent header storage in browser using IndexedDB ([#102](https://github.com/eigerco/lumina/pull/102))
- Improve performance of Exchange ([#104](https://github.com/eigerco/lumina/pull/104))
- Choose transport based on target_arch and improve TCP connections ([#103](https://github.com/eigerco/lumina/pull/103))
- *(node)* Implement Syncer ([#94](https://github.com/eigerco/lumina/pull/94))
- Add trusted peers and keep track of multiple connections per peer ([#92](https://github.com/eigerco/lumina/pull/92))
- Forward only verified and new HeaderSub messages ([#89](https://github.com/eigerco/lumina/pull/89))
- Store improvements ([#88](https://github.com/eigerco/lumina/pull/88))
- Improve verification and implement verification in Exchange client ([#85](https://github.com/eigerco/lumina/pull/85))
- Peer discovery with Kademlia ([#79](https://github.com/eigerco/lumina/pull/79))
- *(node)* Add state of peer in PeerTracker ([#82](https://github.com/eigerco/lumina/pull/82))
- *(node/exchange)* Request HEAD from multiple peers and choose the best result ([#67](https://github.com/eigerco/lumina/pull/67))
- *(node)* Implement exchange client ([#63](https://github.com/eigerco/lumina/pull/63))
- *(exchange)* Add pre-allocating the buffer for reading in HeaderCodec ([#64](https://github.com/eigerco/lumina/pull/64))
- *(node)* hide all services behind traits ([#54](https://github.com/eigerco/lumina/pull/54))
- add RPC calls for p2p module and tests for them ([#52](https://github.com/eigerco/lumina/pull/52))
- *(p2p)* add NetworkInfo command ([#45](https://github.com/eigerco/lumina/pull/45))
- Support WASM for Web in celestia-node ([#44](https://github.com/eigerco/lumina/pull/44))
- Implement initial architecture of node crate ([#42](https://github.com/eigerco/lumina/pull/42))

### Fixed
- Use pre-defined DNS nameservers ([#129](https://github.com/eigerco/lumina/pull/129))
- *(node)* Allow HeaderSub reinitialization ([#128](https://github.com/eigerco/lumina/pull/128))
- *(node/syncer)* Stop fetching header when all peers disconnected ([#111](https://github.com/eigerco/lumina/pull/111))
- Yield between multiple `ExtendedHeader::validate` ([#107](https://github.com/eigerco/lumina/pull/107))
- *(node)* Adjust PeerTrackerInfo when peer trust is changed ([#105](https://github.com/eigerco/lumina/pull/105))
- *(node)* Remove keep_alive::Behaviour ([#95](https://github.com/eigerco/lumina/pull/95))
- *(node/exchange)* Forward handling of pending connections to req_resp ([#80](https://github.com/eigerco/lumina/pull/80))
- Use `get_header_by_height(0)` to get the HEAD ([#71](https://github.com/eigerco/lumina/pull/71))

### Other
- add missing metadata to the toml files ([#170](https://github.com/eigerco/lumina/pull/170))
- document public api ([#161](https://github.com/eigerco/lumina/pull/161))
- error message for missing token and cleanups ([#168](https://github.com/eigerco/lumina/pull/168))
- update bootstrap nodes to lumina ([#163](https://github.com/eigerco/lumina/pull/163))
- rename the node implementation to Lumina ([#156](https://github.com/eigerco/lumina/pull/156))
- hide p2p and syncer components from public node api ([#127](https://github.com/eigerco/lumina/pull/127))
- Upgrade libp2p to v0.53.0 ([#126](https://github.com/eigerco/lumina/pull/126))
- rename Exchange to HeaderEx ([#122](https://github.com/eigerco/lumina/pull/122))
- *(node)* Optimize transport for memory and bandwitdh ([#113](https://github.com/eigerco/lumina/pull/113))
- Produce an error if bootnode multiaddr do not have peer ID ([#106](https://github.com/eigerco/lumina/pull/106))
- Implement persistent storage for native builds using sled  ([#97](https://github.com/eigerco/lumina/pull/97))
- trim the features of workspace dependencies ([#99](https://github.com/eigerco/lumina/pull/99))
- Add integration tests for exchange server/client ([#93](https://github.com/eigerco/lumina/pull/93))
- Implement ExchangeServerHandler ([#72](https://github.com/eigerco/lumina/pull/72))
- Write test cases for invalid and bad headers ([#87](https://github.com/eigerco/lumina/pull/87))
- Remove flume crate ([#86](https://github.com/eigerco/lumina/pull/86))
- Split gossipsub code in smaller functions ([#81](https://github.com/eigerco/lumina/pull/81))
- Implement header Store ([#73](https://github.com/eigerco/lumina/pull/73))
- Migrate from `log` to `tracing` ([#55](https://github.com/eigerco/lumina/pull/55))
