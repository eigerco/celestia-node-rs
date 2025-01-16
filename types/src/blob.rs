//! Types related to creation and submission of blobs.

use std::iter;

use serde::{Deserialize, Serialize};

mod commitment;
mod msg_pay_for_blobs;

use crate::consts::appconsts;
use crate::consts::appconsts::{subtree_root_threshold, AppVersion};
use crate::nmt::Namespace;
use crate::{bail_validation, Error, Result, Share};

pub use self::commitment::Commitment;
pub use self::msg_pay_for_blobs::MsgPayForBlobs;
pub use celestia_proto::celestia::blob::v1::MsgPayForBlobs as RawMsgPayForBlobs;
pub use celestia_proto::proto::blob::v1::BlobProto as RawBlob;
pub use celestia_proto::proto::blob::v1::BlobTx as RawBlobTx;
#[cfg(all(feature = "wasm-bindgen", target_arch = "wasm32"))]
use wasm_bindgen::prelude::*;

/// Arbitrary data that can be stored in the network within certain [`Namespace`].
// NOTE: We don't use the `serde(try_from)` pattern for this type
// becase JSON representation needs to have `commitment` field but
// Protobuf definition doesn't.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(feature = "wasm-bindgen", target_arch = "wasm32"),
    wasm_bindgen(getter_with_clone, inspectable)
)]
pub struct Blob {
    /// A [`Namespace`] the [`Blob`] belongs to.
    pub namespace: Namespace,
    /// Data stored within the [`Blob`].
    #[serde(with = "tendermint_proto::serializers::bytes::base64string")]
    pub data: Vec<u8>,
    /// Version indicating the format in which [`Share`]s should be created from this [`Blob`].
    ///
    /// [`Share`]: crate::share::Share
    pub share_version: u8,
    /// A [`Commitment`] computed from the [`Blob`]s data.
    pub commitment: Commitment,
    /// Index of the blob's first share in the EDS. Only set for blobs retrieved from chain.
    // note: celestia supports deserializing blobs without index, so we should too
    #[serde(default, with = "index_serde")]
    pub index: Option<u64>,
}

/// Params defines the parameters for the blob module.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BlobParams {
    /// Gas cost per blob byte
    pub gas_per_blob_byte: u32,
    /// Max square size
    pub gov_max_square_size: u64,
}

impl Blob {
    /// Create a new blob with the given data within the [`Namespace`].
    ///
    /// # Errors
    ///
    /// This function propagates any error from the [`Commitment`] creation.
    ///
    /// # Example
    ///
    /// ```
    /// use celestia_types::{AppVersion, Blob, nmt::Namespace};
    ///
    /// let my_namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
    /// let blob = Blob::new(my_namespace, b"some data to store on blockchain".to_vec(), AppVersion::V2)
    ///     .expect("Failed to create a blob");
    ///
    /// assert_eq!(
    ///     &serde_json::to_string_pretty(&blob).unwrap(),
    ///     indoc::indoc! {r#"{
    ///       "namespace": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQIDBAU=",
    ///       "data": "c29tZSBkYXRhIHRvIHN0b3JlIG9uIGJsb2NrY2hhaW4=",
    ///       "share_version": 0,
    ///       "commitment": "m0A4feU6Fqd5Zy9td3M7lntG8A3PKqe6YdugmAsWz28=",
    ///       "index": -1
    ///     }"#},
    /// );
    /// ```
    pub fn new(namespace: Namespace, data: Vec<u8>, app_version: AppVersion) -> Result<Blob> {
        let subtree_root_threshold = subtree_root_threshold(app_version);
        let commitment = Commitment::from_blob(namespace, &data[..], 0, subtree_root_threshold)?;

        Ok(Blob {
            namespace,
            data,
            share_version: 0,
            commitment,
            index: None,
        })
    }

    /// Creates a `Blob` from [`RawBlob`] and an [`AppVersion`].
    pub fn from_raw(raw: RawBlob, app_version: AppVersion) -> Result<Blob> {
        let subtree_root_threshold = subtree_root_threshold(app_version);

        let namespace = Namespace::new(raw.namespace_version as u8, &raw.namespace_id)?;
        let commitment = Commitment::from_blob(
            namespace,
            &raw.data[..],
            raw.share_version as u8,
            subtree_root_threshold,
        )?;

        Ok(Blob {
            namespace,
            data: raw.data,
            share_version: raw.share_version as u8,
            commitment,
            index: None,
        })
    }

    /// Validate [`Blob`]s data with the [`Commitment`] it has.
    ///
    /// # Errors
    ///
    /// If validation fails, this function will return an error with a reason of failure.
    ///
    /// # Example
    ///
    /// ```
    /// use celestia_types::Blob;
    /// # use celestia_types::consts::appconsts::AppVersion;
    /// # use celestia_types::nmt::Namespace;
    /// #
    /// # let namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
    ///
    /// let mut blob = Blob::new(namespace, b"foo".to_vec(), AppVersion::V2).unwrap();
    ///
    /// assert!(blob.validate(AppVersion::V2).is_ok());
    ///
    /// let other_blob = Blob::new(namespace, b"bar".to_vec(), AppVersion::V2).unwrap();
    /// blob.commitment = other_blob.commitment;
    ///
    /// assert!(blob.validate(AppVersion::V2).is_err());
    /// ```
    pub fn validate(&self, app_version: AppVersion) -> Result<()> {
        let subtree_root_threshold = subtree_root_threshold(app_version);

        let computed_commitment = Commitment::from_blob(
            self.namespace,
            &self.data,
            self.share_version,
            subtree_root_threshold,
        )?;

        if self.commitment != computed_commitment {
            bail_validation!("blob commitment != localy computed commitment")
        }

        Ok(())
    }

    /// Encode the blob into a sequence of shares.
    ///
    /// Check the [`Share`] documentation for more information about the share format.
    ///
    /// # Errors
    ///
    /// This function will return an error if [`InfoByte`] creation fails
    /// or the data length overflows [`u32`].
    ///
    /// # Example
    ///
    /// ```
    /// use celestia_types::Blob;
    /// # use celestia_types::consts::appconsts::AppVersion;
    /// # use celestia_types::nmt::Namespace;
    /// # let namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
    ///
    /// let blob = Blob::new(namespace, b"foo".to_vec(), AppVersion::V2).unwrap();
    /// let shares = blob.to_shares().unwrap();
    ///
    /// assert_eq!(shares.len(), 1);
    /// ```
    ///
    /// [`Share`]: crate::share::Share
    /// [`InfoByte`]: crate::share::InfoByte
    pub fn to_shares(&self) -> Result<Vec<Share>> {
        commitment::split_blob_to_shares(self.namespace, self.share_version, &self.data)
    }

    /// Reconstructs a blob from shares.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - there is not enough shares to reconstruct the blob
    /// - blob doesn't start with the first share
    /// - shares are from any reserved namespace
    /// - shares for the blob have different namespaces / share version
    ///
    /// # Example
    ///
    /// ```
    /// use celestia_types::{AppVersion, Blob};
    /// # use celestia_types::nmt::Namespace;
    /// # let namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
    ///
    /// let blob = Blob::new(namespace, b"foo".to_vec(), AppVersion::V2).unwrap();
    /// let shares = blob.to_shares().unwrap();
    ///
    /// let reconstructed = Blob::reconstruct(&shares, AppVersion::V2).unwrap();
    ///
    /// assert_eq!(blob, reconstructed);
    /// ```
    pub fn reconstruct<'a, I>(shares: I, app_version: AppVersion) -> Result<Self>
    where
        I: IntoIterator<Item = &'a Share>,
    {
        let mut shares = shares.into_iter();
        let first_share = shares.next().ok_or(Error::MissingShares)?;
        let blob_len = first_share
            .sequence_length()
            .ok_or(Error::ExpectedShareWithSequenceStart)?;
        let namespace = first_share.namespace();
        if namespace.is_reserved() {
            return Err(Error::UnexpectedReservedNamespace);
        }
        let share_version = first_share.info_byte().expect("non parity").version();

        let shares_needed = shares_needed_for_blob(blob_len as usize);
        let mut data =
            Vec::with_capacity(shares_needed * appconsts::CONTINUATION_SPARSE_SHARE_CONTENT_SIZE);
        data.extend_from_slice(first_share.payload().expect("non parity"));

        for _ in 1..shares_needed {
            let share = shares.next().ok_or(Error::MissingShares)?;
            if share.namespace() != namespace {
                return Err(Error::BlobSharesMetadataMismatch(format!(
                    "expected namespace ({:?}) got ({:?})",
                    namespace,
                    share.namespace()
                )));
            }
            let version = share.info_byte().expect("non parity").version();
            if version != share_version {
                return Err(Error::BlobSharesMetadataMismatch(format!(
                    "expected share version ({}) got ({})",
                    share_version, version
                )));
            }
            if share.sequence_length().is_some() {
                return Err(Error::UnexpectedSequenceStart);
            }
            data.extend_from_slice(share.payload().expect("non parity"));
        }

        // remove padding
        data.truncate(blob_len as usize);

        Self::new(namespace, data, app_version)
    }

    /// Reconstructs all the blobs from shares.
    ///
    /// This function will seek shares that indicate start of the next blob (with
    /// [`Share::sequence_length`]) and pass them to [`Blob::reconstruct`].
    /// It will automatically ignore all shares that are within reserved namespaces
    /// e.g. it is completely fine to pass whole [`ExtendedDataSquare`] to this
    /// function and get all blobs in the block.
    ///
    /// # Errors
    ///
    /// This function propagates any errors from [`Blob::reconstruct`].
    ///
    /// # Example
    ///
    /// ```
    /// use celestia_types::{AppVersion, Blob};
    /// # use celestia_types::nmt::Namespace;
    /// # let namespace1 = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
    /// # let namespace2 = Namespace::new_v0(&[2, 3, 4, 5, 6]).expect("Invalid namespace");
    ///
    /// let blobs = vec![
    ///     Blob::new(namespace1, b"foo".to_vec(), AppVersion::V2).unwrap(),
    ///     Blob::new(namespace2, b"bar".to_vec(), AppVersion::V2).unwrap(),
    /// ];
    /// let shares: Vec<_> = blobs.iter().flat_map(|blob| blob.to_shares().unwrap()).collect();
    ///
    /// let reconstructed = Blob::reconstruct_all(&shares, AppVersion::V2).unwrap();
    ///
    /// assert_eq!(blobs, reconstructed);
    /// ```
    ///
    /// [`ExtendedDataSquare`]: crate::ExtendedDataSquare
    pub fn reconstruct_all<'a, I>(shares: I, app_version: AppVersion) -> Result<Vec<Self>>
    where
        I: IntoIterator<Item = &'a Share>,
    {
        let mut shares = shares
            .into_iter()
            .filter(|shr| !shr.namespace().is_reserved());
        let mut blobs = Vec::with_capacity(2);

        loop {
            let mut blob = {
                // find next share from blobs namespace that is sequence start
                let Some(start) = shares.find(|&shr| shr.sequence_length().is_some()) else {
                    break;
                };
                iter::once(start).chain(&mut shares)
            };
            blobs.push(Blob::reconstruct(&mut blob, app_version)?);
        }

        Ok(blobs)
    }

    /// Get the amount of shares needed to encode this blob.
    ///
    /// # Example
    ///
    /// ```
    /// use celestia_types::{AppVersion, Blob};
    /// # use celestia_types::nmt::Namespace;
    /// # let namespace = Namespace::new_v0(&[1, 2, 3, 4, 5]).expect("Invalid namespace");
    ///
    /// let blob = Blob::new(namespace, b"foo".to_vec(), AppVersion::V3).unwrap();
    /// let shares_len = blob.shares_len();
    ///
    /// let blob_shares = blob.to_shares().unwrap();
    ///
    /// assert_eq!(shares_len, blob_shares.len());
    /// ```
    pub fn shares_len(&self) -> usize {
        let Some(without_first_share) = self
            .data
            .len()
            .checked_sub(appconsts::FIRST_SPARSE_SHARE_CONTENT_SIZE)
        else {
            return 1;
        };
        1 + without_first_share.div_ceil(appconsts::CONTINUATION_SPARSE_SHARE_CONTENT_SIZE)
    }
}

impl From<Blob> for RawBlob {
    fn from(value: Blob) -> RawBlob {
        RawBlob {
            namespace_id: value.namespace.id().to_vec(),
            namespace_version: value.namespace.version() as u32,
            data: value.data,
            share_version: value.share_version as u32,
            signer: Vec::new(),
        }
    }
}

#[cfg(all(feature = "wasm-bindgen", target_arch = "wasm32"))]
#[wasm_bindgen]
impl Blob {
    /// Create a new blob with the given data within the [`Namespace`].
    #[wasm_bindgen(constructor)]
    pub fn js_new(
        namespace: &Namespace,
        data: Vec<u8>,
        app_version: &appconsts::JsAppVersion,
    ) -> Result<Blob> {
        Self::new(*namespace, data, (*app_version).into())
    }

    /// Clone a blob creating a new deep copy of it.
    #[wasm_bindgen(js_name = clone)]
    pub fn js_clone(&self) -> Blob {
        self.clone()
    }
}

fn shares_needed_for_blob(blob_len: usize) -> usize {
    let Some(without_first_share) =
        blob_len.checked_sub(appconsts::FIRST_SPARSE_SHARE_CONTENT_SIZE)
    else {
        return 1;
    };
    1 + without_first_share.div_ceil(appconsts::CONTINUATION_SPARSE_SHARE_CONTENT_SIZE)
}

mod index_serde {
    use serde::ser::Error;
    use serde::{Deserialize, Deserializer, Serializer};

    /// Serialize [`Option<u64>`] as `i64` with `None` represented as `-1`.
    pub fn serialize<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let x = value
            .map(i64::try_from)
            .transpose()
            .map_err(S::Error::custom)?
            .unwrap_or(-1);
        serializer.serialize_i64(x)
    }

    /// Deserialize [`Option<u64>`] from `i64` with negative values as `None`.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        i64::deserialize(deserializer).map(|val| if val >= 0 { Some(val as u64) } else { None })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nmt::{NS_ID_SIZE, NS_SIZE};
    use crate::test_utils::random_bytes;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    fn sample_blob() -> Blob {
        serde_json::from_str(
            r#"{
              "namespace": "AAAAAAAAAAAAAAAAAAAAAAAAAAAADCBNOWAP3dM=",
              "data": "8fIMqAB+kQo7+LLmHaDya8oH73hxem6lQWX1",
              "share_version": 0,
              "commitment": "D6YGsPWdxR8ju2OcOspnkgPG2abD30pSHxsFdiPqnVk=",
              "index": -1
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn create_from_raw() {
        let expected = sample_blob();
        let raw = RawBlob::from(expected.clone());
        let created = Blob::from_raw(raw, AppVersion::V2).unwrap();

        assert_eq!(created, expected);
    }

    #[test]
    fn validate_blob() {
        sample_blob().validate(AppVersion::V2).unwrap();
    }

    #[test]
    fn validate_blob_commitment_mismatch() {
        let mut blob = sample_blob();
        blob.commitment = Commitment::new([7; 32]);

        blob.validate(AppVersion::V2).unwrap_err();
    }

    #[test]
    fn deserialize_blob_with_missing_index() {
        serde_json::from_str::<Blob>(
            r#"{
              "namespace": "AAAAAAAAAAAAAAAAAAAAAAAAAAAADCBNOWAP3dM=",
              "data": "8fIMqAB+kQo7+LLmHaDya8oH73hxem6lQWX1",
              "share_version": 0,
              "commitment": "D6YGsPWdxR8ju2OcOspnkgPG2abD30pSHxsFdiPqnVk="
            }"#,
        )
        .unwrap();
    }

    #[test]
    fn reconstruct() {
        for _ in 0..10 {
            let len = rand::random::<usize>() % 1024 * 1024;
            let data = random_bytes(len);
            let ns = Namespace::const_v0(rand::random());
            let blob = Blob::new(ns, data, AppVersion::V2).unwrap();

            let shares = blob.to_shares().unwrap();
            assert_eq!(blob, Blob::reconstruct(&shares, AppVersion::V2).unwrap());
        }
    }

    #[test]
    fn reconstruct_empty() {
        assert!(matches!(
            Blob::reconstruct(&Vec::<Share>::new(), AppVersion::V2),
            Err(Error::MissingShares)
        ));
    }

    #[test]
    fn reconstruct_not_sequence_start() {
        let len = rand::random::<usize>() % 1024 * 1024;
        let data = random_bytes(len);
        let ns = Namespace::const_v0(rand::random());
        let mut shares = Blob::new(ns, data, AppVersion::V2)
            .unwrap()
            .to_shares()
            .unwrap();

        // modify info byte to remove sequence start bit
        shares[0].as_mut()[NS_SIZE] &= 0b11111110;

        assert!(matches!(
            Blob::reconstruct(&shares, AppVersion::V2),
            Err(Error::ExpectedShareWithSequenceStart)
        ));
    }

    #[test]
    fn reconstruct_reserved_namespace() {
        for ns in (0..255).flat_map(|n| {
            let mut v0 = [0; NS_ID_SIZE];
            *v0.last_mut().unwrap() = n;
            let mut v255 = [0xff; NS_ID_SIZE];
            *v255.last_mut().unwrap() = n;

            [Namespace::new_v0(&v0), Namespace::new_v255(&v255)]
        }) {
            let len = (rand::random::<usize>() % 1023 + 1) * 2;
            let data = random_bytes(len);
            let shares = Blob::new(ns.unwrap(), data, AppVersion::V2)
                .unwrap()
                .to_shares()
                .unwrap();

            assert!(matches!(
                Blob::reconstruct(&shares, AppVersion::V2),
                Err(Error::UnexpectedReservedNamespace)
            ));
        }
    }

    #[test]
    fn reconstruct_not_enough_shares() {
        let len = rand::random::<usize>() % 1024 * 1024 + 2048;
        let data = random_bytes(len);
        let ns = Namespace::const_v0(rand::random());
        let shares = Blob::new(ns, data, AppVersion::V2)
            .unwrap()
            .to_shares()
            .unwrap();

        assert!(matches!(
            // minimum for len is 4 so 3 will break stuff
            Blob::reconstruct(&shares[..2], AppVersion::V2),
            Err(Error::MissingShares)
        ));
    }

    #[test]
    fn reconstruct_inconsistent_share_version() {
        let len = rand::random::<usize>() % 1024 * 1024 + 512;
        let data = random_bytes(len);
        let ns = Namespace::const_v0(rand::random());
        let mut shares = Blob::new(ns, data, AppVersion::V2)
            .unwrap()
            .to_shares()
            .unwrap();

        // change share version in second share
        shares[1].as_mut()[NS_SIZE] = 0b11111110;

        assert!(matches!(
            Blob::reconstruct(&shares, AppVersion::V2),
            Err(Error::BlobSharesMetadataMismatch(..))
        ));
    }

    #[test]
    fn reconstruct_inconsistent_namespace() {
        let len = rand::random::<usize>() % 1024 * 1024 + 512;
        let data = random_bytes(len);
        let ns = Namespace::const_v0(rand::random());
        let ns2 = Namespace::const_v0(rand::random());
        let mut shares = Blob::new(ns, data, AppVersion::V2)
            .unwrap()
            .to_shares()
            .unwrap();

        // change namespace in second share
        shares[1].as_mut()[..NS_SIZE].copy_from_slice(ns2.as_bytes());

        assert!(matches!(
            Blob::reconstruct(&shares, AppVersion::V2),
            Err(Error::BlobSharesMetadataMismatch(..))
        ));
    }

    #[test]
    fn reconstruct_unexpected_sequence_start() {
        let len = rand::random::<usize>() % 1024 * 1024 + 512;
        let data = random_bytes(len);
        let ns = Namespace::const_v0(rand::random());
        let mut shares = Blob::new(ns, data, AppVersion::V2)
            .unwrap()
            .to_shares()
            .unwrap();

        // modify info byte to add sequence start bit
        shares[1].as_mut()[NS_SIZE] |= 0b00000001;

        assert!(matches!(
            Blob::reconstruct(&shares, AppVersion::V2),
            Err(Error::UnexpectedSequenceStart)
        ));
    }

    #[test]
    fn reconstruct_all() {
        let blobs: Vec<_> = (0..rand::random::<usize>() % 16 + 3)
            .map(|_| {
                let len = rand::random::<usize>() % 1024 * 1024 + 512;
                let data = random_bytes(len);
                let ns = Namespace::const_v0(rand::random());
                Blob::new(ns, data, AppVersion::V2).unwrap()
            })
            .collect();

        let shares: Vec<_> = blobs
            .iter()
            .flat_map(|blob| blob.to_shares().unwrap())
            .collect();
        let reconstructed = Blob::reconstruct_all(&shares, AppVersion::V2).unwrap();

        assert_eq!(blobs, reconstructed);
    }
}
