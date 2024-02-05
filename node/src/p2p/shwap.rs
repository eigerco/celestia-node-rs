use beetswap::multihasher::Multihasher;
use blockstore::block::CidError;
use celestia_proto::share::p2p::shwap::{
    Data as RawNamespacedData, Row as RawRow, Sample as RawSample,
};
use celestia_types::namespaced_data::{NamespacedDataId, NAMESPACED_DATA_ID_MULTIHASH_CODE};
use celestia_types::nmt::Namespace;
use celestia_types::row::{RowId, ROW_ID_MULTIHASH_CODE};
use celestia_types::sample::{SampleId, SAMPLE_ID_MULTIHASH_CODE};
use cid::{Cid, CidGeneric};
use libp2p::multihash::Multihash;
use prost::Message;

use crate::p2p::Result;

use super::{P2pError, MAX_MH_SIZE};

/// Multihasher for Shwap types.
pub(super) struct ShwapMultihasher;

impl Multihasher<MAX_MH_SIZE> for ShwapMultihasher {
    fn digest(&self, multihash_code: u64, input: &[u8]) -> Option<Multihash<MAX_MH_SIZE>> {
        let data = match multihash_code {
            NAMESPACED_DATA_ID_MULTIHASH_CODE => RawNamespacedData::decode(input).ok()?.data_id,
            ROW_ID_MULTIHASH_CODE => RawRow::decode(input).ok()?.row_id,
            SAMPLE_ID_MULTIHASH_CODE => RawSample::decode(input).ok()?.sample_id,
            _ => return None,
        };

        Multihash::wrap(multihash_code, &data).ok()
    }
}

pub(super) fn row_cid(row_index: u16, block_height: u64) -> Result<Cid> {
    let row_id = RowId::new(row_index, block_height).map_err(P2pError::Cid)?;
    convert_cid(&row_id.into())
}

pub(super) fn sample_cid(index: usize, square_len: usize, block_height: u64) -> Result<Cid> {
    let sample_id = SampleId::new(index, square_len, block_height).map_err(P2pError::Cid)?;
    convert_cid(&sample_id.into())
}

pub(super) fn namespaced_data_cid(
    namespace: Namespace,
    row_index: u16,
    block_height: u64,
) -> Result<Cid> {
    let data_id =
        NamespacedDataId::new(namespace, row_index, block_height).map_err(P2pError::Cid)?;
    convert_cid(&data_id.into())
}

fn convert_cid<const S: usize>(cid: &CidGeneric<S>) -> Result<Cid> {
    beetswap::utils::convert_cid(cid).ok_or(P2pError::Cid(celestia_types::Error::CidError(
        CidError::InvalidMultihashLength(64),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest() {
        let hash = ShwapMultihasher
            .digest(
                0x7821,
                &[
                    10, 39, 6, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 26, 0,
                ],
            )
            .unwrap();

        let cid = "bagqpaanb6aasobqaaaaaaaaaaacqaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaq"
            .parse::<Cid>()
            .unwrap();
        let expected_hash = cid.hash();

        assert_eq!(hash, *expected_hash);
    }
}
