use std::cell::RefCell;
use std::convert::Infallible;
use std::pin::pin;

use async_trait::async_trait;
use celestia_tendermint_proto::Protobuf;
use celestia_types::hash::Hash;
use celestia_types::ExtendedHeader;
use cid::Cid;
use rexie::{Direction, Index, KeyRange, ObjectStore, Rexie, TransactionMode};
use send_wrapper::SendWrapper;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use tokio::sync::Notify;

use crate::store::utils::{ validate_headers, verify_range_contiguous, RangeScanResult, };
use crate::store::{Result, SamplingMetadata, SamplingStatus, Store, StoreError};
use crate::store::header_ranges::{HeaderRange, HeaderRanges};

/// indexeddb version, needs to be incremented on every schema schange
const DB_VERSION: u32 = 3;

// Data stores (SQL table analogue) used in IndexedDb
const HEADER_STORE_NAME: &str = "headers";
const SAMPLING_STORE_NAME: &str = "sampling";
const RANGES_STORE_NAME: &str = "ranges";

// Additional indexes set on HEADER_STORE, for querying by height and hash
const HASH_INDEX_NAME: &str = "hash";
const HEIGHT_INDEX_NAME: &str = "height";

#[derive(Debug, Serialize, Deserialize)]
struct ExtendedHeaderEntry {
    // We use those fields as indexes, names need to match ones in `add_index`
    height: u64,
    hash: Hash,
    header: Vec<u8>,
}

/// A [`Store`] implementation based on a `IndexedDB` browser database.
#[derive(Debug)]
pub struct IndexedDbStore {
    // SendWrapper usage is safe in wasm because we're running on a single thread
    head: SendWrapper<RefCell<Option<ExtendedHeader>>>,
    db: SendWrapper<Rexie>,
    header_added_notifier: Notify,
}

impl IndexedDbStore {
    /// Create or open a persistent store.
    pub async fn new(name: &str) -> Result<IndexedDbStore> {
        let rexie = Rexie::builder(name)
            .version(DB_VERSION)
            .add_object_store(
                ObjectStore::new(HEADER_STORE_NAME)
                    .key_path("id")
                    .auto_increment(true)
                    // These need to match names in `ExtendedHeaderEntry`
                    .add_index(Index::new(HASH_INDEX_NAME, "hash").unique(true))
                    .add_index(Index::new(HEIGHT_INDEX_NAME, "height").unique(true)),
            )
            .add_object_store(ObjectStore::new(SAMPLING_STORE_NAME))
            .add_object_store(ObjectStore::new(RANGES_STORE_NAME))
            .build()
            .await
            .map_err(|e| StoreError::OpenFailed(e.to_string()))?;

        let db_head = match get_head_from_database(&rexie).await {
            Ok(v) => Some(v),
            Err(StoreError::NotFound) => None,
            Err(e) => return Err(e),
        };

        let store = Self {
            head: SendWrapper::new(RefCell::new(db_head.clone())),
            db: SendWrapper::new(rexie),
            header_added_notifier: Notify::new(),
        };

        if let Some(head) = &db_head {
            if store.get_stored_header_ranges().await?.is_empty() {
                let tx = store
                    .db
                    .transaction(&[RANGES_STORE_NAME], TransactionMode::ReadWrite)?;
                let ranges_store = tx.store(RANGES_STORE_NAME)?;
                let jsvalue_range = to_value(&(1, head.height().value()))?;
                let jsvalue_key = to_value(&0)?;

                ranges_store.put(&jsvalue_range, Some(&jsvalue_key)).await?;

                tx.commit().await?;
            }
        }

        Ok(store)
    }

    /// Delete the persistent store.
    pub async fn delete_db(self) -> rexie::Result<()> {
        let name = self.db.name();
        self.db.take().close();
        Rexie::delete(&name).await
    }

    fn get_head(&self) -> Result<ExtendedHeader> {
        // this shouldn't panic, we don't borrow across await points and wasm is single threaded
        self.head.borrow().clone().ok_or(StoreError::NotFound)
    }

    fn get_head_height(&self) -> Result<u64> {
        // this shouldn't panic, we don't borrow across await points and wasm is single threaded
        self.head
            .borrow()
            .as_ref()
            .map(|h| h.height().value())
            .ok_or(StoreError::NotFound)
    }

    async fn get_by_height(&self, height: u64) -> Result<ExtendedHeader> {
        let tx = self
            .db
            .transaction(&[HEADER_STORE_NAME], TransactionMode::ReadOnly)?;

        get_by_height(&tx.store(HEADER_STORE_NAME)?, height).await
    }

    async fn get_by_hash(&self, hash: &Hash) -> Result<ExtendedHeader> {
        let tx = self
            .db
            .transaction(&[HEADER_STORE_NAME], TransactionMode::ReadOnly)?;
        let header_store = tx.store(HEADER_STORE_NAME)?;
        let hash_index = header_store.index(HASH_INDEX_NAME)?;

        let hash_key = to_value(&hash)?;
        let header_entry = hash_index.get(&hash_key).await?;

        if header_entry.is_falsy() {
            return Err(StoreError::NotFound);
        }

        let serialized_header = from_value::<ExtendedHeaderEntry>(header_entry)?.header;
        ExtendedHeader::decode(serialized_header.as_ref())
            .map_err(|e| StoreError::CelestiaTypes(e.into()))
    }

    async fn get_stored_header_ranges(&self) -> Result<HeaderRanges> {
        let tx = self
            .db
            .transaction(&[RANGES_STORE_NAME], TransactionMode::ReadOnly)?;
        let store = tx.store(RANGES_STORE_NAME)?;

        let ranges: HeaderRanges = store
            .get_all(None, None, None, Some(Direction::Next))
            .await?
            .into_iter()
            .map(|(_k, v)| from_value::<(u64, u64)>(v).map(|(begin, end)| begin..=end))
            .collect::<Result<_, _>>()?;

        Ok(ranges)
    }

    async fn insert(&self, headers: Vec<ExtendedHeader>, verify_neighbours: bool) -> Result<()> {
        let (Some(head), Some(tail)) = (headers.first(), headers.last()) else {
            return Ok(());
        };

        let tx = self.db.transaction(
            &[HEADER_STORE_NAME, RANGES_STORE_NAME],
            TransactionMode::ReadWrite,
        )?;
        let header_store = tx.store(HEADER_STORE_NAME)?;
        let ranges_store = tx.store(RANGES_STORE_NAME)?;

        let headers_range = head.height().value()..=tail.height().value();
        let neighbours_exist = try_insert_to_range(&ranges_store, headers_range).await?;

        if verify_neighbours {
            validate_headers(&headers).await?;
            verify_against_neighbours(&header_store, head, tail, neighbours_exist).await?;
        } else {
            verify_range_contiguous(&headers)?;
        }

        for header in &headers {
            let hash = header.hash();
            let hash_index = header_store.index(HASH_INDEX_NAME)?;
            let jsvalue_hash_key = KeyRange::only(&to_value(&hash)?)?;
            if hash_index.count(Some(&jsvalue_hash_key)).await.unwrap_or(0) != 0 {
                return Err(StoreError::HashExists(hash));
            }

            // make sure Result is Infallible, we unwrap it later
            let serialized_header: std::result::Result<_, Infallible> = header.encode_vec();

            let height = header.height().value();
            let header_entry = ExtendedHeaderEntry {
                height,
                hash,
                header: serialized_header.unwrap(),
            };

            let jsvalue_header = to_value(&header_entry)?;

            header_store.add(&jsvalue_header, None).await?;
        }

        if tail.height().value()
            > self
                .head
                .borrow()
                .as_ref()
                .map(|h| h.height().value())
                .unwrap_or(0)
        {
            self.head.replace(Some(tail.clone()));
        }
        tx.commit().await?;
        self.header_added_notifier.notify_waiters();

        Ok(())
    }

    async fn contains_hash(&self, hash: &Hash) -> Result<bool> {
        let tx = self
            .db
            .transaction(&[HEADER_STORE_NAME], TransactionMode::ReadOnly)?;

        let header_store = tx.store(HEADER_STORE_NAME)?;
        let hash_index = header_store.index(HASH_INDEX_NAME)?;

        let hash_key = KeyRange::only(&to_value(&hash)?)?;

        let hash_count = hash_index.count(Some(&hash_key)).await?;

        Ok(hash_count > 0)
    }

    async fn contains_height(&self, height: u64) -> bool {
        let Ok(stored_ranges) = self.get_stored_header_ranges().await else {
            return false;
        };
        stored_ranges.contains(height)
    }

    async fn update_sampling_metadata(
        &self,
        height: u64,
        status: SamplingStatus,
        cids: Vec<Cid>,
    ) -> Result<()> {
        if !self.contains_height(height).await {
            return Err(StoreError::NotFound);
        }

        let height_key = to_value(&height)?;

        let tx = self
            .db
            .transaction(&[SAMPLING_STORE_NAME], TransactionMode::ReadWrite)?;
        let sampling_store = tx.store(SAMPLING_STORE_NAME)?;

        let previous_entry = sampling_store.get(&height_key).await?;

        let new_entry = if previous_entry.is_falsy() {
            SamplingMetadata { status, cids }
        } else {
            let mut value: SamplingMetadata = from_value(previous_entry)?;

            value.status = status;

            for cid in cids {
                if !value.cids.contains(&cid) {
                    value.cids.push(cid);
                }
            }

            value
        };

        let metadata_jsvalue = to_value(&new_entry)?;

        sampling_store
            .put(&metadata_jsvalue, Some(&height_key))
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn get_sampling_metadata(&self, height: u64) -> Result<Option<SamplingMetadata>> {
        if !self.contains_height(height).await {
            return Err(StoreError::NotFound);
        }

        let tx = self
            .db
            .transaction(&[SAMPLING_STORE_NAME], TransactionMode::ReadOnly)?;
        let store = tx.store(SAMPLING_STORE_NAME)?;

        let height_key = to_value(&height)?;
        let sampling_entry = store.get(&height_key).await?;

        if sampling_entry.is_falsy() {
            return Ok(None);
        }

        Ok(Some(from_value(sampling_entry)?))
    }
}

#[async_trait]
impl Store for IndexedDbStore {
    async fn get_head(&self) -> Result<ExtendedHeader> {
        self.get_head()
    }

    async fn get_by_hash(&self, hash: &Hash) -> Result<ExtendedHeader> {
        let fut = SendWrapper::new(self.get_by_hash(hash));
        fut.await
    }

    async fn get_by_height(&self, height: u64) -> Result<ExtendedHeader> {
        let fut = SendWrapper::new(self.get_by_height(height));
        fut.await
    }

    async fn wait_new_head(&self) -> u64 {
        let head = self.get_head_height().unwrap_or(0);
        let mut notifier = pin!(self.header_added_notifier.notified());

        loop {
            let new_head = self.get_head_height().unwrap_or(0);

            if head != new_head {
                return new_head;
            }

            // Await for a notification
            notifier.as_mut().await;

            // Reset notifier
            notifier.set(self.header_added_notifier.notified());
        }
    }

    async fn wait_height(&self, height: u64) -> Result<()> {
        let mut notifier = pin!(self.header_added_notifier.notified());

        loop {
            let fut = SendWrapper::new(self.contains_height(height));
            if fut.await {
                return Ok(());
            }

            // Await for a notification
            notifier.as_mut().await;

            // Reset notifier
            notifier.set(self.header_added_notifier.notified());
        }
    }

    async fn head_height(&self) -> Result<u64> {
        self.get_head_height()
    }

    async fn has(&self, hash: &Hash) -> bool {
        let fut = SendWrapper::new(self.contains_hash(hash));
        fut.await.unwrap_or(false)
    }

    async fn has_at(&self, height: u64) -> bool {
        let fut = SendWrapper::new(self.contains_height(height));
        fut.await
    }

    async fn update_sampling_metadata(
        &self,
        height: u64,
        status: SamplingStatus,
        cids: Vec<Cid>,
    ) -> Result<()> {
        let fut = SendWrapper::new(self.update_sampling_metadata(height, status, cids));
        fut.await
    }

    async fn get_sampling_metadata(&self, height: u64) -> Result<Option<SamplingMetadata>> {
        let fut = SendWrapper::new(self.get_sampling_metadata(height));
        fut.await
    }

    async fn insert(&self, header: Vec<ExtendedHeader>, verify_neighbours: bool) -> Result<()> {
        let fut = SendWrapper::new(self.insert(header, verify_neighbours));
        fut.await
    }

    async fn get_stored_header_ranges(&self) -> Result<HeaderRanges> {
        let fut = SendWrapper::new(self.get_stored_header_ranges());
        fut.await
    }
}

impl From<rexie::Error> for StoreError {
    fn from(error: rexie::Error) -> StoreError {
        use rexie::Error as E;
        match error {
            e @ E::AsyncChannelError => StoreError::ExecutorError(e.to_string()),
            other => StoreError::FatalDatabaseError(other.to_string()),
        }
    }
}

impl From<serde_wasm_bindgen::Error> for StoreError {
    fn from(error: serde_wasm_bindgen::Error) -> StoreError {
        StoreError::StoredDataError(format!("Error de/serializing: {error}"))
    }
}

async fn get_head_from_database(db: &Rexie) -> Result<ExtendedHeader> {
    let tx = db.transaction(&[HEADER_STORE_NAME], TransactionMode::ReadOnly)?;
    let store = tx.store(HEADER_STORE_NAME)?;

    let store_head = store
        .get_all(None, Some(1), None, Some(Direction::Prev))
        .await?
        .first()
        .ok_or(StoreError::NotFound)?
        .1
        .to_owned();

    let serialized_header = from_value::<ExtendedHeaderEntry>(store_head)?.header;

    ExtendedHeader::decode(serialized_header.as_ref())
        .map_err(|e| StoreError::CelestiaTypes(e.into()))
}

async fn try_insert_to_range(
    ranges_store: &rexie::Store,
    new_range: HeaderRange,
) -> Result<(bool, bool)> {
    let stored_ranges = ranges_store
        .get_all(None, None, None, Some(Direction::Next))
        .await?
        .into_iter()
        .map(|(_k, v)| from_value::<(u64, u64)>(v).map(|(start, end)| start..=end))
        .collect::<Result<HeaderRanges, _>>()?;

    let RangeScanResult {
        range_index,
        range,
        range_to_remove,
    } = stored_ranges.check_range_insert(&new_range)?;

    if let Some(to_remove) = range_to_remove {
        let jsvalue_key_to_remove = to_value(&to_remove)?;
        ranges_store.delete(&jsvalue_key_to_remove).await?;
    }

    let jsvalue_range = to_value(&(*range.start(), *range.end()))?;
    let jsvalue_key = to_value(&range_index)?;

    ranges_store.put(&jsvalue_range, Some(&jsvalue_key)).await?;

    let prev_exists = new_range.start() != range.start();
    let next_exists = new_range.end() != range.end();

    Ok((prev_exists, next_exists))
}

async fn get_by_height(header_store: &rexie::Store, height: u64) -> Result<ExtendedHeader> {
    let height_index = header_store.index(HEIGHT_INDEX_NAME)?;

    let height_key = to_value(&height)?;
    let header_entry = height_index.get(&height_key).await?;

    // querying unset key returns empty value
    if header_entry.is_falsy() {
        return Err(StoreError::NotFound);
    }

    let serialized_header = from_value::<ExtendedHeaderEntry>(header_entry)?.header;
    ExtendedHeader::decode(serialized_header.as_ref())
        .map_err(|e| StoreError::CelestiaTypes(e.into()))
}

async fn verify_against_neighbours(
    header_store: &rexie::Store,
    lowest_header: &ExtendedHeader,
    highest_header: &ExtendedHeader,
    neighbours_exist: (bool, bool),
) -> Result<()> {
    debug_assert!(lowest_header.height().value() <= highest_header.height().value());
    let (prev_exists, next_exists) = neighbours_exist;

    if prev_exists {
        let prev = get_by_height(header_store, lowest_header.height().value() - 1)
            .await
            .map_err(|e| {
                if let StoreError::NotFound = e {
                    StoreError::StoredDataError(
                        "inconsistency between headers and ranges table".into(),
                    )
                } else {
                    e
                }
            })?;
        prev.verify(lowest_header)?;
    }

    if next_exists {
        let next = get_by_height(header_store, highest_header.height().value() + 1)
            .await
            .map_err(|e| {
                if let StoreError::NotFound = e {
                    StoreError::StoredDataError(
                        "inconsistency between headers and ranges table".into(),
                    )
                } else {
                    e
                }
            })?;
        highest_header.verify(&next)?;
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use celestia_types::test_utils::ExtendedHeaderGenerator;
    use function_name::named;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[named]
    #[wasm_bindgen_test]
    async fn test_large_db() {
        let store_name = function_name!();
        Rexie::delete(store_name).await.unwrap();
        let s = IndexedDbStore::new(store_name)
            .await
            .expect("creating test store failed");

        let mut gen = ExtendedHeaderGenerator::new();
        let expected_height = 1_000;

        for h in 1..=expected_height {
            s.append_single_unchecked(gen.next())
                .await
                .expect("inserting test data failed");
            s.update_sampling_metadata(h, SamplingStatus::Accepted, vec![])
                .await
                .expect("marking sampled failed");
        }

        assert_eq!(s.get_head().unwrap().height().value(), expected_height);

        drop(s);
        // re-open the store, to force re-calculation of the cached heights
        let s = IndexedDbStore::new(store_name)
            .await
            .expect("re-opening large test store failed");

        assert_eq!(s.get_head().unwrap().height().value(), expected_height);
    }

    #[named]
    #[wasm_bindgen_test]
    async fn test_persistence() {
        let (original_store, mut gen) = gen_filled_store(0, function_name!()).await;
        let mut original_headers = gen.next_many(20);

        for h in &original_headers {
            original_store
                .append_single_unchecked(h.clone())
                .await
                .expect("inserting test data failed");
        }
        drop(original_store);

        let reopened_store = IndexedDbStore::new(function_name!())
            .await
            .expect("failed to reopen store");

        assert_eq!(
            original_headers.last().unwrap().height().value(),
            reopened_store.head_height().await.unwrap()
        );
        for original_header in &original_headers {
            let stored_header = reopened_store
                .get_by_height(original_header.height().value())
                .await
                .unwrap();
            assert_eq!(original_header, &stored_header);
        }

        let mut new_headers = gen.next_many(10);
        for h in &new_headers {
            reopened_store
                .append_single_unchecked(h.clone())
                .await
                .expect("failed to insert data");
        }
        drop(reopened_store);

        original_headers.append(&mut new_headers);

        let reopened_store = IndexedDbStore::new(function_name!())
            .await
            .expect("failed to reopen store");

        assert_eq!(
            original_headers.last().unwrap().height().value(),
            reopened_store.head_height().await.unwrap()
        );
        for original_header in &original_headers {
            let stored_header = reopened_store
                .get_by_height(original_header.height().value())
                .await
                .unwrap();
            assert_eq!(original_header, &stored_header);
        }
    }

    #[named]
    #[wasm_bindgen_test]
    async fn test_delete_db() {
        let (original_store, _) = gen_filled_store(3, function_name!()).await;
        assert_eq!(original_store.get_head_height().unwrap(), 3);

        original_store.delete_db().await.unwrap();

        let same_name_store = IndexedDbStore::new(function_name!())
            .await
            .expect("creating test store failed");

        assert!(matches!(
            same_name_store.get_head_height(),
            Err(StoreError::NotFound)
        ));
    }

    mod migration_v1 {
        use super::*;

        const PREVIOUS_DB_VERSION: u32 = 1;

        // this fn sets upt the store manually with v1 schema (original, ExtendedHeader only),
        // and fills it with `hs` headers. It is a caller responsibility to make sure provided
        // headers are correct and in order.
        async fn init_store(name: &str, hs: Vec<ExtendedHeader>) {
            Rexie::delete(name).await.unwrap();
            let rexie = Rexie::builder(name)
                .version(PREVIOUS_DB_VERSION)
                .add_object_store(
                    ObjectStore::new("headers")
                        .key_path("id")
                        .auto_increment(true)
                        .add_index(Index::new("hash", "hash").unique(true))
                        .add_index(Index::new("height", "height").unique(true)),
                )
                .build()
                .await
                .unwrap();

            let tx = rexie
                .transaction(&["headers"], TransactionMode::ReadWrite)
                .unwrap();
            let header_store = tx.store("headers").unwrap();

            for header in hs {
                let header_entry = ExtendedHeaderEntry {
                    height: header.height().value(),
                    hash: header.hash(),
                    header: header.encode_vec().unwrap(),
                };

                header_store
                    .add(&to_value(&header_entry).unwrap(), None)
                    .await
                    .unwrap();
            }
        }

        #[named]
        #[wasm_bindgen_test]
        async fn migration_test() {
            let store_name = function_name!();
            let mut gen = ExtendedHeaderGenerator::new();
            let headers = gen.next_many(20);

            init_store(store_name, headers.clone()).await;

            let store = IndexedDbStore::new(store_name)
                .await
                .expect("opening migrated store failed");

            for header in headers {
                let height = header.height().value();

                let header_by_hash = store.get_by_hash(&header.hash()).await.unwrap();
                assert_eq!(header, header_by_hash);

                let header_by_height = store.get_by_height(height).await.unwrap();
                assert_eq!(header, header_by_height);

                // migrated headers should be marked as not sampled
                let sampling_data = store.get_sampling_metadata(height).await.unwrap();
                assert!(sampling_data.is_none());
            }

            store
                .update_sampling_metadata(1, SamplingStatus::Accepted, vec![])
                .await
                .unwrap();
            let sampling_data = store.get_sampling_metadata(1).await.unwrap().unwrap();
            assert_eq!(sampling_data.status, SamplingStatus::Accepted);
        }
    }

    // open IndexedDB with unique per-test name to avoid interference and make cleanup easier
    pub async fn gen_filled_store(
        amount: u64,
        name: &str,
    ) -> (IndexedDbStore, ExtendedHeaderGenerator) {
        Rexie::delete(name).await.unwrap();
        let s = IndexedDbStore::new(name)
            .await
            .expect("creating test store failed");
        let mut gen = ExtendedHeaderGenerator::new();

        let headers = gen.next_many(amount);

        for header in headers {
            s.append_single_unchecked(header)
                .await
                .expect("inserting test data failed");
        }

        (s, gen)
    }
}
