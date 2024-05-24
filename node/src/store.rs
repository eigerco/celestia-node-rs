//! Primitives related to the [`ExtendedHeader`] storage.

use std::fmt::Debug;
use std::io::Cursor;
use std::ops::{Bound, RangeBounds, RangeInclusive};

use async_trait::async_trait;
use celestia_tendermint_proto::Protobuf;
use celestia_types::hash::Hash;
use celestia_types::ExtendedHeader;
use cid::Cid;
use prost::Message;
use serde::{Deserialize, Serialize};
use smallvec::{IntoIter, SmallVec};
use thiserror::Error;

pub use in_memory_store::InMemoryStore;
#[cfg(target_arch = "wasm32")]
pub use indexed_db_store::IndexedDbStore;
#[cfg(not(target_arch = "wasm32"))]
pub use redb_store::RedbStore;

mod in_memory_store;
#[cfg(target_arch = "wasm32")]
mod indexed_db_store;
#[cfg(not(target_arch = "wasm32"))]
mod redb_store;

pub(crate) mod utils;

pub(crate) use utils::calculate_missing_ranges;

/// Sampling status for a header.
///
/// This struct persists DAS-ing information in a header store for future reference.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SamplingMetadata {
    /// Indicates whether this node was able to successfuly sample the block
    pub accepted: bool,

    /// List of CIDs used, when decision to accept or reject the header was taken. Can be used
    /// to remove associated data from Blockstore, when cleaning up the old ExtendedHeaders
    pub cids_sampled: Vec<Cid>,
}

type Result<T, E = StoreError> = std::result::Result<T, E>;

pub type HeaderRange = RangeInclusive<u64>;

pub(crate) trait RangeLengthExt {
    fn len(&self) -> u64;
}

impl RangeLengthExt for RangeInclusive<u64> {
    fn len(&self) -> u64 {
        self.end() - self.start() + 1
    }
}

// TODO: less pub?
#[derive(Debug, Clone, PartialEq, Default)] // TODO: manual Display implementation probably
pub struct HeaderRanges(pub SmallVec<[RangeInclusive<u64>; 2]>);

impl HeaderRanges {
    pub fn validate(&self) -> Result<()> {
        // TODO
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|r| r.is_empty())
    }
}

impl<const T: usize> From<[RangeInclusive<u64>; T]> for HeaderRanges {
    fn from(value: [RangeInclusive<u64>; T]) -> Self {
        Self(value.into_iter().collect())
    }
}

impl IntoIterator for HeaderRanges {
    type Item = u64;
    type IntoIter = HeaderRangesIterator;
    fn into_iter(self) -> Self::IntoIter {
        let mut outer_iter = self.0.into_iter();
        HeaderRangesIterator {
            inner_iter: outer_iter.next(),
            outer_iter,
        }
    }
}

pub struct HeaderRangesIterator {
    inner_iter: Option<RangeInclusive<u64>>,
    outer_iter: IntoIter<[RangeInclusive<u64>; 2]>,
}

impl HeaderRangesIterator {
    pub fn next_batch(&mut self, limit: u64) -> Option<RangeInclusive<u64>> {
        let current_range = self.inner_iter.take()?;

        if current_range.len() <= limit {
            self.inner_iter = self.outer_iter.next();
            Some(current_range)
        } else {
            let returned_range = *current_range.start()..=*current_range.start() + limit - 1;
            self.inner_iter = Some(*current_range.start() + limit..=*current_range.end());
            Some(returned_range)
        }
    }
}

impl Iterator for HeaderRangesIterator {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.inner_iter.as_mut()?.next() {
            return Some(v);
        }
        self.inner_iter = self.outer_iter.next();
        self.next()
    }
}

/// An asynchronous [`ExtendedHeader`] storage.
///
/// Currently it is required that all the headers are inserted to the storage
/// in order, starting from the genesis.
#[async_trait]
pub trait Store: Send + Sync + Debug {
    /// Returns the [`ExtendedHeader`] with the highest height.
    async fn get_head(&self) -> Result<ExtendedHeader>;

    /// Returns the header of a specific hash.
    async fn get_by_hash(&self, hash: &Hash) -> Result<ExtendedHeader>;

    /// Returns the header of a specific height.
    async fn get_by_height(&self, height: u64) -> Result<ExtendedHeader>;

    /// Returns when `height` is available in the `Store`.
    async fn wait_height(&self, height: u64) -> Result<()>;

    /// Returns the headers from the given heights range.
    ///
    /// If start of the range is unbounded, the first returned header will be of height 1.
    /// If end of the range is unbounded, the last returned header will be the last header in the
    /// store.
    ///
    /// # Errors
    ///
    /// If range contains a height of a header that is not found in the store or [`RangeBounds`]
    /// cannot be converted to a valid range.
    async fn get_range<R>(&self, range: R) -> Result<Vec<ExtendedHeader>>
    where
        R: RangeBounds<u64> + Send,
    {
        let head_height = self.head_height().await?;
        let range = to_headers_range(range, head_height)?;

        let amount = if range.is_empty() {
            0
        } else {
            range.end() - range.start() + 1 // add one as it's inclusive
        };

        let mut headers = Vec::with_capacity(
            amount
                .try_into()
                .map_err(|_| StoreError::InvalidHeadersRange)?,
        );

        for height in range {
            let header = self.get_by_height(height).await?;
            headers.push(header);
        }

        Ok(headers)
    }

    /// Returns the highest known height.
    async fn head_height(&self) -> Result<u64>;

    /// Returns true if hash exists in the store.
    async fn has(&self, hash: &Hash) -> bool;

    /// Returns true if height exists in the store.
    async fn has_at(&self, height: u64) -> bool;

    // === LEGACY APPENDS ===
    async fn append_single_unchecked(&self, header: ExtendedHeader) -> Result<()> {
        self.insert_single(header, false).await
    }

    async fn append_single(&self, header: ExtendedHeader) -> Result<()> {
        self.insert_single(header, true).await
    }

    async fn append_unchecked(&self, headers: Vec<ExtendedHeader>) -> Result<()> {
        self.insert(headers, false).await
    }

    async fn append(&self, headers: Vec<ExtendedHeader>) -> Result<()> {
        self.insert(headers, true).await
    }

    /// Returns height of the lowest header that wasn't sampled yet
    async fn next_unsampled_height(&self) -> Result<u64>;

    /// Sets or updates sampling result for the header.
    ///
    /// In case of update, provided CID list is appended onto the existing one, as not to lose
    /// references to previously sampled blocks.
    ///
    /// Returns next unsampled header or error, if occured
    async fn update_sampling_metadata(
        &self,
        height: u64,
        accepted: bool,
        cids: Vec<Cid>,
    ) -> Result<u64>;

    /// Gets the sampling metadata for the height.
    ///
    /// `Err(StoreError::NotFound)` indicates that both header **and** sampling metadata for the requested
    /// height are not in the store.
    ///
    /// `Ok(None)` indicates that header is in the store but sampling metadata is not set yet.
    async fn get_sampling_metadata(&self, height: u64) -> Result<Option<SamplingMetadata>>;

    /// Append a range of headers maintaining continuity from the genesis to the head.
    ///
    /// # Note
    ///
    /// This method does not validate or verify that `headers` are indeed correct.
    /*
    async fn append_unchecked(&self, headers: Vec<ExtendedHeader>) -> Result<()> {
        for header in headers.into_iter() {
            self.append_single_unchecked(header).await?;
        }

        Ok(())
    }
    */

    /// new main insertion function
    async fn insert_single(&self, header: ExtendedHeader, verify_neighbours: bool) -> Result<()> {
        self.insert(vec![header], verify_neighbours).await
    }

    async fn insert(&self, headers: Vec<ExtendedHeader>, verify_neighbours: bool) -> Result<()>;

    /*
    /// Append a range of headers maintaining continuity from the genesis to the head.
    async fn insert(&self, headers: Vec<ExtendedHeader>) -> Result<()> {
        validate_headers(&headers).await?;

        let Some(front) = headers.first() else {
            return Ok(());
        };


        front.verify_adjacent_range(&headers[1..])?;

        let last_index = headers.len().saturating_sub(1);

        for (i, header) in headers.into_iter().enumerate() {
            let verify_neighbours = i == 0 || i == last_index;
            self.insert_single(header, verify_neighbours).await?;
        }
        Ok(())
    }
    */

    async fn get_stored_header_ranges(&self) -> Result<HeaderRanges>;
}

/// Representation of all the errors that can occur when interacting with the [`Store`].
#[derive(Error, Debug)]
pub enum StoreError {
    /// Hash already exists in the store.
    #[error("Hash {0} already exists in store")]
    HashExists(Hash),

    /// Height already exists in the store.
    #[error("Height {0} already exists in store")]
    HeightExists(u64),

    /// Inserted height is not following store's current head.
    #[error("Failed to append header at height {1}")]
    NonContinuousAppend(u64, u64),

    /// TODO: reword
    #[error("Failed to insert header range, it overlaps with one already existing in the store: {0}..={1}")]
    HeaderRangeOverlap(u64, u64),

    /// TODO: this is super unhelpful on its own
    #[error("Trying to insert new header range at disallowed position: {0}..={1}")]
    InsertPlacementDisallowed(u64, u64),

    #[error("provided header range has a gap between heights {0} and {1}")]
    InsertRangeWithGap(u64, u64),

    //#[error("Failed to find range to add header with height {0} to: {1}")]
    //RangeFinderError(u64, HeaderRange),
    /// Header validation has failed.
    #[error("Failed to validate header at height {0}")]
    HeaderChecksError(u64),

    /// Header not found.
    #[error("Header not found in store")]
    NotFound,

    /// Header not found but it should be present. Store is invalid.
    #[error("Store in inconsistent state; height {0} within known range, but missing header")]
    LostHeight(u64),

    /// Hash not found but it should be present. Store is invalid.
    #[error("Store in inconsistent state; height->hash mapping exists, {0} missing")]
    LostHash(Hash),

    /// An error propagated from the [`celestia_types`].
    #[error(transparent)]
    CelestiaTypes(#[from] celestia_types::Error),

    /// Storage corrupted.
    #[error("Stored data in inconsistent state, try reseting the store: {0}")]
    StoredDataError(String),

    /// Unrecoverable error reported by the database.
    #[error("Database reported unrecoverable error: {0}")]
    FatalDatabaseError(String),

    /// An error propagated from the async executor.
    #[error("Received error from executor: {0}")]
    ExecutorError(String),

    /// Failed to open the store.
    #[error("Error opening store: {0}")]
    OpenFailed(String),

    /// Invalid range of headers provided.
    #[error("Invalid headers range")]
    InvalidHeadersRange,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<tokio::task::JoinError> for StoreError {
    fn from(error: tokio::task::JoinError) -> StoreError {
        StoreError::ExecutorError(error.to_string())
    }
}

#[derive(Message)]
struct RawSamplingMetadata {
    #[prost(bool, tag = "1")]
    accepted: bool,

    #[prost(message, repeated, tag = "2")]
    cids_sampled: Vec<Vec<u8>>,
}

impl Protobuf<RawSamplingMetadata> for SamplingMetadata {}

impl TryFrom<RawSamplingMetadata> for SamplingMetadata {
    type Error = cid::Error;

    fn try_from(item: RawSamplingMetadata) -> Result<Self, Self::Error> {
        let cids_sampled = item
            .cids_sampled
            .iter()
            .map(|cid| {
                let buffer = Cursor::new(cid);
                Cid::read_bytes(buffer)
            })
            .collect::<Result<_, _>>()?;

        Ok(SamplingMetadata {
            accepted: item.accepted,
            cids_sampled,
        })
    }
}

impl From<SamplingMetadata> for RawSamplingMetadata {
    fn from(item: SamplingMetadata) -> Self {
        let cids_sampled = item.cids_sampled.iter().map(|cid| cid.to_bytes()).collect();

        RawSamplingMetadata {
            accepted: item.accepted,
            cids_sampled,
        }
    }
}

/// a helper function to convert any kind of range to the inclusive range of header heights.
fn to_headers_range(bounds: impl RangeBounds<u64>, last_index: u64) -> Result<RangeInclusive<u64>> {
    let start = match bounds.start_bound() {
        // in case of unbounded, default to the first height
        Bound::Unbounded => 1,
        // range starts after the last index or before first height
        Bound::Included(&x) if x > last_index || x == 0 => return Err(StoreError::NotFound),
        Bound::Excluded(&x) if x >= last_index => return Err(StoreError::NotFound),
        // valid start indexes
        Bound::Included(&x) => x,
        Bound::Excluded(&x) => x + 1, // can't overflow thanks to last_index check
    };
    let end = match bounds.end_bound() {
        // in case of unbounded, default to the last index
        Bound::Unbounded => last_index,
        // range ends after the last index
        Bound::Included(&x) if x > last_index => return Err(StoreError::NotFound),
        Bound::Excluded(&x) if x > last_index + 1 => return Err(StoreError::NotFound),
        // prevent the underflow later on
        Bound::Excluded(&0) => 0,
        // valid end indexes
        Bound::Included(&x) => x,
        Bound::Excluded(&x) => x - 1,
    };

    Ok(start..=end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use celestia_types::test_utils::ExtendedHeaderGenerator;
    use rstest::rstest;

    // rstest only supports attributes which last segment is `test`
    // https://docs.rs/rstest/0.18.2/rstest/attr.rstest.html#inject-test-attribute
    use crate::test_utils::async_test as test;

    #[test]
    async fn converts_bounded_ranges() {
        assert_eq!(1..=15, to_headers_range(1..16, 100).unwrap());
        assert_eq!(1..=15, to_headers_range(1..=15, 100).unwrap());
        assert_eq!(300..=400, to_headers_range(300..401, 500).unwrap());
        assert_eq!(300..=400, to_headers_range(300..=400, 500).unwrap());
    }

    #[test]
    async fn starts_from_one_when_unbounded_start() {
        assert_eq!(&1, to_headers_range(..=10, 100).unwrap().start());
        assert_eq!(&1, to_headers_range(..10, 100).unwrap().start());
        assert_eq!(&1, to_headers_range(.., 100).unwrap().start());
    }

    #[test]
    async fn ends_on_last_index_when_unbounded_end() {
        assert_eq!(&10, to_headers_range(1.., 10).unwrap().end());
        assert_eq!(&11, to_headers_range(1.., 11).unwrap().end());
        assert_eq!(&10, to_headers_range(.., 10).unwrap().end());
    }

    #[test]
    async fn handle_ranges_ending_precisely_at_last_index() {
        let last_index = 10;

        let bounds_ending_at_last_index = [
            (Bound::Unbounded, Bound::Included(last_index)),
            (Bound::Unbounded, Bound::Excluded(last_index + 1)),
        ];

        for bound in bounds_ending_at_last_index {
            let range = to_headers_range(bound, last_index).unwrap();
            assert_eq!(*range.end(), last_index);
        }
    }

    #[test]
    async fn handle_ranges_ending_after_last_index() {
        let last_index = 10;

        let bounds_ending_after_last_index = [
            (Bound::Unbounded, Bound::Included(last_index + 1)),
            (Bound::Unbounded, Bound::Excluded(last_index + 2)),
        ];

        for bound in bounds_ending_after_last_index {
            to_headers_range(bound, last_index).unwrap_err();
        }
    }

    #[test]
    async fn errors_if_zero_heigth_is_included() {
        let includes_zero_height = 0..5;
        to_headers_range(includes_zero_height, 10).unwrap_err();
    }

    #[test]
    async fn handle_ranges_starting_precisely_at_last_index() {
        let last_index = 10;

        let bounds_starting_at_last_index = [
            (Bound::Included(last_index), Bound::Unbounded),
            (Bound::Excluded(last_index - 1), Bound::Unbounded),
        ];

        for bound in bounds_starting_at_last_index {
            let range = to_headers_range(bound, last_index).unwrap();
            assert_eq!(*range.start(), last_index);
        }
    }

    #[test]
    async fn handle_ranges_starting_after_last_index() {
        let last_index = 10;

        let bounds_starting_after_last_index = [
            (Bound::Included(last_index + 1), Bound::Unbounded),
            (Bound::Excluded(last_index), Bound::Unbounded),
        ];

        for bound in bounds_starting_after_last_index {
            to_headers_range(bound, last_index).unwrap_err();
        }
    }

    #[test]
    async fn handle_ranges_that_lead_to_empty_ranges() {
        let last_index = 10;

        let bounds_leading_to_empty_range = [
            (Bound::Unbounded, Bound::Excluded(0)),
            (Bound::Included(3), Bound::Excluded(3)),
            (Bound::Included(3), Bound::Included(2)),
            (Bound::Excluded(2), Bound::Included(2)),
        ];

        for bound in bounds_leading_to_empty_range {
            assert!(to_headers_range(bound, last_index).unwrap().is_empty());
        }
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_contains_height<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut s = s;
        fill_store(&mut s, 2).await;

        assert!(!s.has_at(0).await);
        assert!(s.has_at(1).await);
        assert!(s.has_at(2).await);
        assert!(!s.has_at(3).await);
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_empty_store<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        assert!(matches!(s.head_height().await, Err(StoreError::NotFound)));
        assert!(matches!(s.get_head().await, Err(StoreError::NotFound)));
        assert!(matches!(
            s.get_by_height(1).await,
            Err(StoreError::NotFound)
        ));
        assert!(matches!(
            s.get_by_hash(&Hash::Sha256([0; 32])).await,
            Err(StoreError::NotFound)
        ));
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_read_write<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut gen = ExtendedHeaderGenerator::new();

        let header = gen.next();

        s.append_single_unchecked(header.clone()).await.unwrap();
        assert_eq!(s.head_height().await.unwrap(), 1);
        assert_eq!(s.get_head().await.unwrap(), header);
        assert_eq!(s.get_by_height(1).await.unwrap(), header);
        assert_eq!(s.get_by_hash(&header.hash()).await.unwrap(), header);
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_pregenerated_data<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut s = s;
        fill_store(&mut s, 100).await;

        assert_eq!(s.head_height().await.unwrap(), 100);
        let head = s.get_head().await.unwrap();
        assert_eq!(s.get_by_height(100).await.unwrap(), head);
        assert!(matches!(
            s.get_by_height(101).await,
            Err(StoreError::NotFound)
        ));

        let header = s.get_by_height(54).await.unwrap();
        assert_eq!(s.get_by_hash(&header.hash()).await.unwrap(), header);
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_duplicate_insert<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut s = s;
        let mut gen = fill_store(&mut s, 100).await;

        let header101 = gen.next();
        s.append_single_unchecked(header101.clone()).await.unwrap();

        //s.append_single_unchecked(header101).await.unwrap();
        assert!(matches!(
            s.append_single_unchecked(header101).await,
            Err(StoreError::HeaderRangeOverlap(101, 101))
        ));
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_overwrite_height<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut s = s;
        let gen = fill_store(&mut s, 100).await;

        // Height 30 with different hash
        let header29 = s.get_by_height(29).await.unwrap();
        let header30 = gen.next_of(&header29);

        let insert_existing_result = s.append_single_unchecked(header30).await;
        assert!(matches!(
            insert_existing_result,
            Err(StoreError::HeaderRangeOverlap(30, 30))
        ));
    }

    /*
    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_overwrite_hash<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut s = s;
        fill_store(&mut s, 100).await;

        let mut dup_header = s.get_by_height(33).await.unwrap();
        dup_header.header.height = Height::from(101u32);
        let insert_existing_result = s.append_single_unchecked(dup_header).await;
        insert_existing_result.unwrap();
        /*
        assert!(matches!(
            insert_existing_result,
            Err(StoreError::HashExists(_))
        ));
        */
    }
    */

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_append_range<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut s = s;
        let mut gen = fill_store(&mut s, 10).await;

        let hs = gen.next_many(4);
        s.append_unchecked(hs).await.unwrap();
        s.get_by_height(14).await.unwrap();
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_fill_range_gap<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut s = s;
        let mut gen = fill_store(&mut s, 10).await;

        // height 11
        let skipped = gen.next();
        // height 12
        let upcoming_head = gen.next();

        s.append_single_unchecked(upcoming_head).await.unwrap();
        s.append_single_unchecked(skipped).await.unwrap();
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_fill_range_gap_with_invalid_header<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut s = s;
        let mut gen = fill_store(&mut s, 10).await;

        let mut gen_prime = gen.fork();
        // height 11
        let _skipped = gen.next();
        let another_chain = gen_prime.next();
        // height 12
        let upcoming_head = gen.next();

        s.append_single(upcoming_head).await.unwrap();
        s.append_single(another_chain).await.unwrap_err(); // TODO: match?
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_non_continuous_append<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut s = s;
        let mut gen = fill_store(&mut s, 10).await;
        let mut hs = gen.next_many(6);

        // remove height 14
        hs.remove(3);

        assert!(matches!(
            s.append_unchecked(hs).await,
            Err(StoreError::InsertRangeWithGap(13, 15))
        ));
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_appends_with_gaps<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut gen = ExtendedHeaderGenerator::new_from_height(5);
        let header5 = gen.next();
        gen.next_many(4);
        let header10 = gen.next();
        gen.next_many(4);
        let header15 = gen.next();

        s.append_single_unchecked(header5).await.unwrap();
        s.append_single_unchecked(header15).await.unwrap();
        s.append_single_unchecked(header10).await.unwrap_err();
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_sampling_height_empty_store<S: Store>(
        #[case]
        #[future(awt)]
        store: S,
    ) {
        store
            .update_sampling_metadata(0, true, vec![])
            .await
            .unwrap_err();
        store
            .update_sampling_metadata(1, true, vec![])
            .await
            .unwrap_err();
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_sampling_height<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut store = s;
        fill_store(&mut store, 9).await;

        store
            .update_sampling_metadata(0, true, vec![])
            .await
            .unwrap_err();
        store
            .update_sampling_metadata(1, true, vec![])
            .await
            .unwrap();
        store
            .update_sampling_metadata(2, true, vec![])
            .await
            .unwrap();
        store
            .update_sampling_metadata(3, false, vec![])
            .await
            .unwrap();
        store
            .update_sampling_metadata(4, true, vec![])
            .await
            .unwrap();
        store
            .update_sampling_metadata(5, false, vec![])
            .await
            .unwrap();
        store
            .update_sampling_metadata(6, false, vec![])
            .await
            .unwrap();

        store
            .update_sampling_metadata(8, true, vec![])
            .await
            .unwrap();

        assert_eq!(store.next_unsampled_height().await.unwrap(), 7);

        store
            .update_sampling_metadata(7, true, vec![])
            .await
            .unwrap();

        assert_eq!(store.next_unsampled_height().await.unwrap(), 9);

        store
            .update_sampling_metadata(9, true, vec![])
            .await
            .unwrap();

        assert_eq!(store.next_unsampled_height().await.unwrap(), 10);

        store
            .update_sampling_metadata(10, true, vec![])
            .await
            .unwrap_err();
        store
            .update_sampling_metadata(10, false, vec![])
            .await
            .unwrap_err();
        store
            .update_sampling_metadata(20, true, vec![])
            .await
            .unwrap_err();
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_sampling_merge<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut store = s;
        fill_store(&mut store, 1).await;

        let cid0 = "zdpuAyvkgEDQm9TenwGkd5eNaosSxjgEYd8QatfPetgB1CdEZ"
            .parse()
            .unwrap();
        let cid1 = "zb2rhe5P4gXftAwvA4eXQ5HJwsER2owDyS9sKaQRRVQPn93bA"
            .parse()
            .unwrap();
        let cid2 = "bafkreieq5jui4j25lacwomsqgjeswwl3y5zcdrresptwgmfylxo2depppq"
            .parse()
            .unwrap();

        store
            .update_sampling_metadata(1, false, vec![cid0])
            .await
            .unwrap();
        assert_eq!(store.next_unsampled_height().await.unwrap(), 2);

        store
            .update_sampling_metadata(1, false, vec![])
            .await
            .unwrap();
        assert_eq!(store.next_unsampled_height().await.unwrap(), 2);

        let sampling_data = store.get_sampling_metadata(1).await.unwrap().unwrap();
        assert!(!sampling_data.accepted);
        assert_eq!(sampling_data.cids_sampled, vec![cid0]);

        store
            .update_sampling_metadata(1, true, vec![cid1])
            .await
            .unwrap();
        assert_eq!(store.next_unsampled_height().await.unwrap(), 2);

        let sampling_data = store.get_sampling_metadata(1).await.unwrap().unwrap();
        assert!(sampling_data.accepted);
        assert_eq!(sampling_data.cids_sampled, vec![cid0, cid1]);

        store
            .update_sampling_metadata(1, true, vec![cid0, cid2])
            .await
            .unwrap();
        assert_eq!(store.next_unsampled_height().await.unwrap(), 2);

        let sampling_data = store.get_sampling_metadata(1).await.unwrap().unwrap();
        assert!(sampling_data.accepted);
        assert_eq!(sampling_data.cids_sampled, vec![cid0, cid1, cid2]);
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_sampled_cids<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let mut store = s;
        fill_store(&mut store, 5).await;

        let cids: Vec<Cid> = [
            "bafkreieq5jui4j25lacwomsqgjeswwl3y5zcdrresptwgmfylxo2depppq",
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
            "zdpuAyvkgEDQm9TenwGkd5eNaosSxjgEYd8QatfPetgB1CdEZ",
            "zb2rhe5P4gXftAwvA4eXQ5HJwsER2owDyS9sKaQRRVQPn93bA",
        ]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();

        store
            .update_sampling_metadata(1, true, cids.clone())
            .await
            .unwrap();
        store
            .update_sampling_metadata(2, true, cids[0..1].to_vec())
            .await
            .unwrap();
        store
            .update_sampling_metadata(4, false, cids[3..].to_vec())
            .await
            .unwrap();
        store
            .update_sampling_metadata(5, false, vec![])
            .await
            .unwrap();

        assert_eq!(store.next_unsampled_height().await.unwrap(), 3);

        let sampling_data = store.get_sampling_metadata(1).await.unwrap().unwrap();
        assert_eq!(sampling_data.cids_sampled, cids);
        assert!(sampling_data.accepted);

        let sampling_data = store.get_sampling_metadata(2).await.unwrap().unwrap();
        assert_eq!(sampling_data.cids_sampled, cids[0..1]);
        assert!(sampling_data.accepted);

        assert!(store.get_sampling_metadata(3).await.unwrap().is_none());

        let sampling_data = store.get_sampling_metadata(4).await.unwrap().unwrap();
        assert_eq!(sampling_data.cids_sampled, cids[3..]);
        assert!(!sampling_data.accepted);

        let sampling_data = store.get_sampling_metadata(5).await.unwrap().unwrap();
        assert_eq!(sampling_data.cids_sampled, vec![]);
        assert!(!sampling_data.accepted);

        assert!(matches!(
            store.get_sampling_metadata(0).await,
            Err(StoreError::NotFound)
        ));
        assert!(matches!(
            store.get_sampling_metadata(6).await,
            Err(StoreError::NotFound)
        ));
        assert!(matches!(
            store.get_sampling_metadata(100).await,
            Err(StoreError::NotFound)
        ));
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_empty_store_range<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let store = s;

        assert_eq!(store.get_stored_header_ranges().await.unwrap(), [].into());
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_single_header_range<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let store = s;
        let mut gen = ExtendedHeaderGenerator::new();

        gen.skip(19);

        let prepend0 = gen.next();
        let prepend1 = gen.next_many(5);
        store.append(gen.next_many(4)).await.unwrap();
        store.append(gen.next_many(5)).await.unwrap();
        store.append(prepend1).await.unwrap();
        store.append_single(prepend0).await.unwrap();
        store.append(gen.next_many(5)).await.unwrap();
        store.append_single(gen.next()).await.unwrap();

        let final_ranges = store.get_stored_header_ranges().await.unwrap();
        assert_eq!(final_ranges, [20..=40].into());
    }

    // no in-memory store for tests below. It doesn't expect to be resumed from disk,
    // so it doesn't support multiple ranges.
    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_ranges_consolidation<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let store = s;
        let mut gen = ExtendedHeaderGenerator::new();

        gen.skip(9);

        let skip0 = gen.next_many(5);
        store.append(gen.next_many(2)).await.unwrap();
        store.append(gen.next_many(3)).await.unwrap();

        let skip1 = gen.next();
        store.append_single_unchecked(gen.next()).await.unwrap();

        let skip2 = gen.next_many(5);

        store.append_single_unchecked(gen.next()).await.unwrap();

        let skip3 = gen.next_many(5);
        let skip4 = gen.next_many(5);
        let skip5 = gen.next_many(5);

        store.append(skip5).await.unwrap();
        store.append(skip4).await.unwrap();
        store.append(skip3).await.unwrap();
        store.append(skip2).await.unwrap();
        store.append_single(skip1).await.unwrap();
        store.append(skip0).await.unwrap();

        let final_ranges = store.get_stored_header_ranges().await.unwrap();
        assert_eq!(final_ranges, [10..=42].into());
    }

    #[rstest]
    #[case::in_memory(new_in_memory_store())]
    #[cfg_attr(not(target_arch = "wasm32"), case::redb(new_redb_store()))]
    #[cfg_attr(target_arch = "wasm32", case::indexed_db(new_indexed_db_store()))]
    #[self::test]
    async fn test_neighbour_validation<S: Store>(
        #[case]
        #[future(awt)]
        s: S,
    ) {
        let store = s;
        let mut gen = ExtendedHeaderGenerator::new();

        store.append(gen.next_many(5)).await.unwrap();
        let mut fork = gen.fork();
        let _gap = gen.next();
        store.append(gen.next_many(4)).await.unwrap();

        store.append_single(fork.next()).await.unwrap_err();
    }

    #[test]
    async fn test_iter() {
        let ranges = HeaderRanges::from([1..=5, 7..=10]);
        assert_eq!(
            ranges.into_iter().collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 7, 8, 9, 10]
        );

        let ranges = HeaderRanges::from([1..=1, 2..=4, 8..=8]);
        assert_eq!(ranges.into_iter().collect::<Vec<_>>(), vec![1, 2, 3, 4, 8]);

        let mut iter = HeaderRanges::from([1..=1]).into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    async fn test_iter_batches() {
        let mut ranges = HeaderRanges::from([1..=100]).into_iter();
        assert_eq!(ranges.next_batch(10), Some(1..=10));
        assert_eq!(ranges.next_batch(10), Some(11..=20));
        assert_eq!(ranges.next_batch(100), Some(21..=100));

        let mut ranges = HeaderRanges::from([1..=10, 21..=30, 41..=50]).into_iter();
        assert_eq!(ranges.next_batch(20), Some(1..=10));
        assert_eq!(ranges.next_batch(1), Some(21..=21));
        assert_eq!(ranges.next_batch(2), Some(22..=23));
        assert_eq!(ranges.next_batch(3), Some(24..=26));
        assert_eq!(ranges.next_batch(4), Some(27..=30));
        assert_eq!(ranges.next_batch(5), Some(41..=45));
        assert_eq!(ranges.next_batch(100), Some(46..=50));
    }

    /// Fills an empty store
    async fn fill_store<S: Store>(store: &mut S, amount: u64) -> ExtendedHeaderGenerator {
        assert!(!store.has_at(1).await, "Store is not empty");

        let mut gen = ExtendedHeaderGenerator::new();
        //let headers = gen.next_many(amount);

        store
            .append_unchecked(gen.next_many(amount))
            .await
            .expect("inserting test data failed");

        gen
    }

    async fn new_in_memory_store() -> InMemoryStore {
        InMemoryStore::new()
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn new_redb_store() -> RedbStore {
        RedbStore::in_memory().await.unwrap()
    }

    #[cfg(target_arch = "wasm32")]
    async fn new_indexed_db_store() -> IndexedDbStore {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let db_name = format!("indexeddb-lumina-node-store-test-{id}");

        // DB can persist if test run within the browser
        rexie::Rexie::delete(&db_name).await.unwrap();

        IndexedDbStore::new(&db_name)
            .await
            .expect("creating test store failed")
    }

    #[test]
    async fn test_header_ranges_empty() {
        assert!(HeaderRanges::from([]).is_empty());
        assert!(!HeaderRanges::from([1..=3]).is_empty());
    }
}
