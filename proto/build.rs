//! A build script generating rust types from protobuf definitions.

use prost_types::FileDescriptorSet;
use std::collections::HashSet;

const DEFAULT: &str = r#"#[serde(default)]"#;
const HEXSTRING: &str = r#"#[serde(with = "crate::serializers::bytes::hexstring")]"#;
const VEC_HEXSTRING: &str = r#"#[serde(with = "crate::serializers::bytes::vec_hexstring")]"#;
const SERIALIZED: &str = r#"#[derive(::serde::Deserialize, ::serde::Serialize)]"#;
const SERIALIZED_DEFAULT: &str =
    r#"#[derive(::serde::Deserialize, ::serde::Serialize)] #[serde(default)]"#;
const TRANSPARENT: &str = r#"#[serde(transparent)]"#;
const BASE64STRING: &str = r#"#[serde(with = "crate::serializers::bytes::base64string")]"#;
const QUOTED_WITH_DEFAULT: &str = r#"#[serde(with = "crate::serializers::from_str", default)]"#;
const VEC_BASE64STRING: &str = r#"#[serde(with = "crate::serializers::bytes::vec_base64string")]"#;
const OPTION_TIMESTAMP: &str = r#"#[serde(with = "crate::serializers::option_timestamp")]"#;
const OPTION_PROTOBUF_DURATION: &str =
    r#"#[serde(with = "crate::serializers::option_protobuf_duration")]"#;
const NULL_DEFAULT: &str = r#"#[serde(with = "crate::serializers::null_default")]"#;
const VEC_SKIP_IF_EMPTY: &str = r#"#[serde(skip_serializing_if = "::std::vec::Vec::is_empty")]"#;
const BYTES_SKIP_IF_EMPTY: &str = r#"#[serde(skip_serializing_if = "bytes::Bytes::is_empty")]"#;

#[rustfmt::skip]
static CUSTOM_TYPE_ATTRIBUTES: &[(&str, &str)] = &[
    (".celestia.blob.v1.MsgPayForBlobs", SERIALIZED_DEFAULT),
    (".celestia.core.v1.da.DataAvailabilityHeader", SERIALIZED_DEFAULT),
    (".celestia.core.v1.proof.NMTProof", SERIALIZED_DEFAULT),
    (".celestia.core.v1.proof.Proof", SERIALIZED_DEFAULT),
    (".celestia.core.v1.proof.RowProof", SERIALIZED_DEFAULT),
    (".celestia.core.v1.proof.ShareProof", SERIALIZED_DEFAULT),
    (".cosmos.base.abci.v1beta1.ABCIMessageLog", SERIALIZED_DEFAULT),
    (".cosmos.base.abci.v1beta1.Attribute", SERIALIZED_DEFAULT),
    (".cosmos.base.abci.v1beta1.StringEvent", SERIALIZED_DEFAULT),
    (".cosmos.base.abci.v1beta1.TxResponse", SERIALIZED_DEFAULT),
    (".cosmos.base.v1beta1.Coin", SERIALIZED_DEFAULT),
    (".cosmos.base.query.v1beta1.PageResponse", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.QueryDelegationResponse", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.DelegationResponse", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.Delegation", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.QueryRedelegationsResponse", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.RedelegationResponse", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.Redelegation", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.RedelegationEntryResponse", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.RedelegationEntry", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.QueryUnbondingDelegationResponse", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.UnbondingDelegation", SERIALIZED_DEFAULT),
    (".cosmos.staking.v1beta1.UnbondingDelegationEntry", SERIALIZED_DEFAULT),
    (".cosmos.tx.v1beta1.SignDoc", SERIALIZED_DEFAULT),
    (".cosmos.tx.v1beta1.SignDoc", r#"#[serde(rename_all = "camelCase")]"#),
    (".header.pb.ExtendedHeader", SERIALIZED_DEFAULT),
    (".proof.pb.Proof", SERIALIZED_DEFAULT),
    (".proto.blob.v1.BlobProto", SERIALIZED),
    (".shwap.AxisType", SERIALIZED),
    (".shwap.Row", SERIALIZED),
    (".shwap.RowNamespaceData", SERIALIZED_DEFAULT),
    (".shwap.Sample", SERIALIZED_DEFAULT),
    (".shwap.Share", SERIALIZED_DEFAULT),
    (".shwap.Share", TRANSPARENT),

    // Celestia's mods
    (".tendermint_celestia_mods.types.Block", SERIALIZED),
    (".tendermint_celestia_mods.types.Data", SERIALIZED),
    (".tendermint_celestia_mods.abci.TimeoutsInfo", SERIALIZED),
    (".tendermint_celestia_mods.abci.ResponseInfo", SERIALIZED),
];

#[rustfmt::skip]
static CUSTOM_FIELD_ATTRIBUTES: &[(&str, &str)] = &[
    (".celestia.core.v1.da.DataAvailabilityHeader.column_roots", VEC_BASE64STRING),
    (".celestia.core.v1.da.DataAvailabilityHeader.row_roots", VEC_BASE64STRING),
    (".celestia.core.v1.proof.NMTProof.leaf_hash", BASE64STRING),
    (".celestia.core.v1.proof.NMTProof.nodes", VEC_BASE64STRING),
    (".celestia.core.v1.proof.Proof.aunts", VEC_BASE64STRING),
    (".celestia.core.v1.proof.Proof.leaf_hash", BASE64STRING),
    (".celestia.core.v1.proof.RowProof.root", BASE64STRING),
    (".celestia.core.v1.proof.RowProof.row_roots", VEC_HEXSTRING),
    (".celestia.core.v1.proof.ShareProof.data", VEC_BASE64STRING),
    (".celestia.core.v1.proof.ShareProof.namespace_id", BASE64STRING),
    (".cosmos.base.abci.v1beta1.TxResponse.logs", NULL_DEFAULT),
    (".cosmos.base.abci.v1beta1.TxResponse.events", NULL_DEFAULT),
    (".cosmos.base.query.v1beta1.PageResponse.next_key", BASE64STRING),
    (".cosmos.staking.v1beta1.RedelegationEntry.completion_time", OPTION_TIMESTAMP),
    (".cosmos.staking.v1beta1.UnbondingDelegationEntry.completion_time", OPTION_TIMESTAMP),
    (".proof.pb.Proof.leaf_hash", BASE64STRING),
    (".proof.pb.Proof.nodes", VEC_BASE64STRING),
    (".proto.blob.v1.BlobProto.data", BASE64STRING),
    (".proto.blob.v1.BlobProto.namespace_id", BASE64STRING),
    (".proto.blob.v1.BlobProto.signer", VEC_SKIP_IF_EMPTY),
    (".proto.blob.v1.BlobProto.signer", BASE64STRING),
    (".shwap.RowNamespaceData.shares", NULL_DEFAULT),
    (".shwap.Share", BASE64STRING),

    // Celestia's mods
    (".tendermint_celestia_mods.types.Data.txs", VEC_BASE64STRING),
    (".tendermint_celestia_mods.types.Data.hash", HEXSTRING),
    (".tendermint_celestia_mods.abci.TimeoutsInfo.timeout_propose", OPTION_PROTOBUF_DURATION),
    (".tendermint_celestia_mods.abci.TimeoutsInfo.timeout_commit", OPTION_PROTOBUF_DURATION),
    (".tendermint_celestia_mods.abci.ResponseInfo.data", DEFAULT),
    (".tendermint_celestia_mods.abci.ResponseInfo.version", DEFAULT),
    (".tendermint_celestia_mods.abci.ResponseInfo.app_version", QUOTED_WITH_DEFAULT),
    (".tendermint_celestia_mods.abci.ResponseInfo.last_block_height", QUOTED_WITH_DEFAULT),
    (".tendermint_celestia_mods.abci.ResponseInfo.last_block_app_hash", DEFAULT),
    (".tendermint_celestia_mods.abci.ResponseInfo.last_block_app_hash", BYTES_SKIP_IF_EMPTY),
];

#[rustfmt::skip]
static EXTERN_PATHS: &[(&str, &str)] = &[
    (".google.protobuf.Any", "::tendermint_proto::google::protobuf::Any"),
    (".google.protobuf.Duration", "::tendermint_proto::google::protobuf::Duration"),
    (".google.protobuf.Timestamp", "::tendermint_proto::google::protobuf::Timestamp"),
    (".tendermint", "::tendermint_proto::v0_34"),
];

const PROTO_FILES: &[&str] = &[
    "vendor/celestia/blob/v1/params.proto",
    "vendor/celestia/blob/v1/query.proto",
    "vendor/celestia/blob/v1/tx.proto",
    "vendor/celestia/core/v1/da/data_availability_header.proto",
    "vendor/celestia/core/v1/proof/proof.proto",
    "vendor/celestia/core/v1/tx/tx.proto",
    "vendor/cosmos/auth/v1beta1/auth.proto",
    "vendor/cosmos/auth/v1beta1/query.proto",
    "vendor/cosmos/bank/v1beta1/bank.proto",
    "vendor/cosmos/bank/v1beta1/genesis.proto",
    "vendor/cosmos/bank/v1beta1/query.proto",
    "vendor/cosmos/bank/v1beta1/tx.proto",
    "vendor/cosmos/base/abci/v1beta1/abci.proto",
    "vendor/cosmos/base/node/v1beta1/query.proto",
    "vendor/cosmos/base/tendermint/v1beta1/query.proto",
    "vendor/cosmos/base/v1beta1/coin.proto",
    "vendor/cosmos/crypto/ed25519/keys.proto",
    "vendor/cosmos/crypto/multisig/v1beta1/multisig.proto",
    "vendor/cosmos/crypto/secp256k1/keys.proto",
    "vendor/cosmos/staking/v1beta1/query.proto",
    "vendor/cosmos/tx/v1beta1/service.proto",
    "vendor/cosmos/tx/v1beta1/tx.proto",
    "vendor/go-header/p2p/pb/header_request.proto",
    "vendor/go-square/blob/v1/blob.proto",
    "vendor/header/pb/extended_header.proto",
    "vendor/share/eds/byzantine/pb/share.proto",
    "vendor/share/shwap/p2p/bitswap/pb/bitswap.proto",
    "vendor/share/shwap/pb/shwap.proto",
    "vendor/tendermint-celestia-mods/abci/types.proto",
    "vendor/tendermint-celestia-mods/blockchain/types.proto",
    "vendor/tendermint-celestia-mods/mempool/types.proto",
    "vendor/tendermint-celestia-mods/state/types.proto",
    "vendor/tendermint-celestia-mods/store/types.proto",
    "vendor/tendermint-celestia-mods/types/block.proto",
    "vendor/tendermint-celestia-mods/types/types.proto",
    "vendor/tendermint/types/types.proto",
];

const INCLUDES: &[&str] = &["vendor", "vendor/nmt"];

fn main() {
    let fds = protox_compile();
    #[cfg(not(feature = "tonic"))]
    prost_build(fds);
    #[cfg(feature = "tonic")]
    tonic_build(fds)
}

fn protox_compile() -> FileDescriptorSet {
    protox::compile(PROTO_FILES, INCLUDES).expect("protox failed to build")
}

#[cfg(not(feature = "tonic"))]
fn prost_build(fds: FileDescriptorSet) {
    let mut config = prost_build::Config::new();

    for (type_path, attr) in CUSTOM_TYPE_ATTRIBUTES {
        config.type_attribute(type_path, attr);
    }

    for (field_path, attr) in CUSTOM_FIELD_ATTRIBUTES {
        config.field_attribute(field_path, attr);
    }

    for (proto_path, rust_path) in EXTERN_PATHS {
        config.extern_path(proto_path.to_string(), rust_path.to_string());
    }

    for (proto_path, rust_path) in tendermint_mods_extern_paths(&fds) {
        config.extern_path(proto_path, rust_path);
    }

    config
        .include_file("mod.rs")
        .enable_type_names()
        .bytes([".tendermint_celestia_mods.abci"])
        .compile_fds(fds)
        .expect("prost failed");
}

#[cfg(feature = "tonic")]
fn tonic_build(fds: FileDescriptorSet) {
    let mut prost_config = prost_build::Config::new();
    prost_config.enable_type_names();

    let mut tonic_config = tonic_build::configure()
        .include_file("mod.rs")
        .build_client(true)
        .build_server(false)
        .use_arc_self(true)
        .compile_well_known_types(true)
        .skip_protoc_run()
        .bytes([".tendermint_celestia_mods.abci"]);

    for (type_path, attr) in CUSTOM_TYPE_ATTRIBUTES {
        tonic_config = tonic_config.type_attribute(type_path, attr);
    }

    for (field_path, attr) in CUSTOM_FIELD_ATTRIBUTES {
        tonic_config = tonic_config.field_attribute(field_path, attr);
    }

    for (proto_path, rust_path) in EXTERN_PATHS {
        tonic_config = tonic_config.extern_path(proto_path, rust_path);
    }

    for (proto_path, rust_path) in tendermint_mods_extern_paths(&fds) {
        tonic_config = tonic_config.extern_path(proto_path, rust_path);
    }

    tonic_config
        .compile_fds_with_config(prost_config, fds)
        .expect("should be able to compile protobuf using tonic");
}

/// Create a list of Tentermint messages that needs to be replaced with Celestia's modifications.
fn tendermint_mods_extern_paths(fds: &FileDescriptorSet) -> Vec<(String, String)> {
    let mut extern_paths = Vec::new();

    let tendermint_types = get_proto_types_of(fds, "tendermint");
    let celestia_mods_types = get_proto_types_of(fds, "tendermint_celestia_mods");

    for mods_type in celestia_mods_types {
        let tm_type = mods_type.replace(".tendermint_celestia_mods.", ".tendermint.");

        if tendermint_types.contains(&tm_type) {
            let new_extern_path = format!("crate{}", mods_type.replace(".", "::"));
            extern_paths.push((tm_type, new_extern_path));
        }
    }

    extern_paths
}

/// Returns the names of all the Protobuf messages of a proto package.
fn get_proto_types_of(fds: &FileDescriptorSet, proto_package: &str) -> HashSet<String> {
    let proto_package_prefix = format!("{proto_package}.");
    let mut types = HashSet::new();

    for fd_proto in &fds.file {
        let Some(ref pkg_name) = fd_proto.package else {
            continue;
        };

        if pkg_name == proto_package || pkg_name.starts_with(&proto_package_prefix) {
            for msg in &fd_proto.message_type {
                if let Some(ref name) = msg.name {
                    types.insert(format!(".{pkg_name}.{name}"));
                }
            }
        }
    }

    types
}
