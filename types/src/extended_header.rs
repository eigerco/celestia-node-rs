use std::time::Duration;

use celestia_proto::header::pb::ExtendedHeader as RawExtendedHeader;
use serde::{Deserialize, Serialize};
use tendermint::block::header::Header;
use tendermint::block::{Commit, Height};
use tendermint::chain::id::Id;
use tendermint::{validator, Hash, Time};
use tendermint_proto::Protobuf;

use crate::trust_level::DEFAULT_TRUST_LEVEL;
use crate::validator_set::ValidatorSetExt;
use crate::{
    bail_validation, bail_verification, DataAvailabilityHeader, Error, Result, ValidateBasic,
};

pub type Validator = validator::Info;
pub type ValidatorSet = validator::Set;

const VERIFY_CLOCK_DRIFT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "RawExtendedHeader", into = "RawExtendedHeader")]
pub struct ExtendedHeader {
    pub header: Header,
    pub commit: Commit,
    pub validator_set: ValidatorSet,
    pub dah: DataAvailabilityHeader,
}

impl ExtendedHeader {
    pub fn chain_id(&self) -> &Id {
        &self.header.chain_id
    }

    pub fn height(&self) -> Height {
        self.header.height
    }

    pub fn time(&self) -> Time {
        self.header.time
    }

    pub fn hash(&self) -> Hash {
        self.commit.block_id.hash
    }

    pub fn last_header_hash(&self) -> Hash {
        self.header
            .last_block_id
            .map(|block_id| block_id.hash)
            .unwrap_or_default()
    }

    pub fn validate(&self) -> Result<()> {
        self.header.validate_basic()?;
        self.commit.validate_basic()?;
        self.validator_set.validate_basic()?;

        // make sure the validator set is consistent with the header
        if self.validator_set.hash() != self.header.validators_hash {
            bail_validation!(
                "validator_set hash ({}) != header validators_hash ({})",
                self.validator_set.hash(),
                self.header.validators_hash,
            )
        }

        // ensure data root from raw header matches computed root
        if Hash::Sha256(self.dah.hash) != self.header.data_hash {
            bail_validation!(
                "dah hash ({}) != header dah hash ({})",
                Hash::Sha256(self.dah.hash),
                self.header.data_hash,
            )
        }

        // Make sure the header is consistent with the commit.
        if self.commit.height != self.height() {
            bail_validation!(
                "commit height ({}) != header height ({})",
                self.commit.height,
                self.height(),
            )
        }

        if self.commit.block_id.hash != self.header.hash() {
            bail_validation!(
                "commit block_id hash ({}) != header hash ({})",
                self.commit.block_id.hash,
                self.header.hash(),
            )
        }

        self.validator_set.verify_commit_light(
            &self.header.chain_id,
            &self.height(),
            &self.commit,
        )?;

        self.dah.validate_basic()?;

        Ok(())
    }

    pub fn verify(&self, untrusted: &ExtendedHeader) -> Result<()> {
        if untrusted.height() <= self.height() {
            bail_verification!(
                "untrusted header height({}) <= current trusted header({})",
                untrusted.height(),
                self.height()
            );
        }

        if untrusted.chain_id() != self.chain_id() {
            bail_verification!(
                "untrusted header has different chain {}, not {}",
                untrusted.chain_id(),
                self.chain_id()
            );
        }

        if !untrusted.time().after(self.time()) {
            bail_verification!(
                "untrusted header time ({}) mut be after current trusted header ({})",
                untrusted.time(),
                self.time()
            );
        }

        let now = Time::now();
        let valid_until = now.checked_add(VERIFY_CLOCK_DRIFT).unwrap();

        if !untrusted.time().before(valid_until) {
            bail_verification!(
                "new untrusted header has a time from the future {} (now: {}, clock_drift: {:?})",
                untrusted.time(),
                now,
                VERIFY_CLOCK_DRIFT
            );
        }

        // If we are verifying an adjacent header
        if self.height().increment() == untrusted.height() {
            if untrusted.header.validators_hash != self.header.next_validators_hash {
                bail_verification!(
                    "expected old header next validators ({}) to match those from new header ({})",
                    self.header.next_validators_hash,
                    untrusted.header.validators_hash,
                );
            }

            if untrusted.last_header_hash() != self.hash() {
                bail_verification!(
                    "expected new header to point to last header hash ({}), but got {}",
                    self.hash(),
                    untrusted.last_header_hash()
                );
            }

            return Ok(());
        }

        self.validator_set.verify_commit_light_trusting(
            self.chain_id(),
            &untrusted.commit,
            DEFAULT_TRUST_LEVEL,
        )?;

        Ok(())
    }
}

impl Protobuf<RawExtendedHeader> for ExtendedHeader {}

impl TryFrom<RawExtendedHeader> for ExtendedHeader {
    type Error = Error;

    fn try_from(value: RawExtendedHeader) -> Result<Self, Self::Error> {
        let header = value.header.ok_or(Error::MissingHeader)?.try_into()?;
        let commit = value.commit.ok_or(Error::MissingCommit)?.try_into()?;
        let validator_set = value
            .validator_set
            .ok_or(Error::MissingValidatorSet)?
            .try_into()?;
        let dah = value
            .dah
            .ok_or(Error::MissingDataAvailabilityHeader)?
            .try_into()?;

        Ok(ExtendedHeader {
            header,
            commit,
            validator_set,
            dah,
        })
    }
}

impl From<ExtendedHeader> for RawExtendedHeader {
    fn from(value: ExtendedHeader) -> RawExtendedHeader {
        RawExtendedHeader {
            header: Some(value.header.into()),
            commit: Some(value.commit.into()),
            validator_set: Some(value.validator_set.into()),
            dah: Some(value.dah.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_eh_chain_1_block_1() -> ExtendedHeader {
        let s = include_str!("../test_data/chain1/extended_header_block_1.json");
        serde_json::from_str(s).unwrap()
    }

    fn sample_eh_chain_1_block_27() -> ExtendedHeader {
        let s = include_str!("../test_data/chain1/extended_header_block_27.json");
        serde_json::from_str(s).unwrap()
    }

    fn sample_eh_chain_2_block_1() -> ExtendedHeader {
        let s = include_str!("../test_data/chain2/extended_header_block_1.json");
        serde_json::from_str(s).unwrap()
    }

    fn sample_eh_chain_2_block_27() -> ExtendedHeader {
        let s = include_str!("../test_data/chain2/extended_header_block_27.json");
        serde_json::from_str(s).unwrap()
    }

    fn sample_eh_chain_2_block_28() -> ExtendedHeader {
        let s = include_str!("../test_data/chain2/extended_header_block_28.json");
        serde_json::from_str(s).unwrap()
    }

    fn sample_eh_chain_2_block_35() -> ExtendedHeader {
        let s = include_str!("../test_data/chain2/extended_header_block_35.json");
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn validate_correct() {
        sample_eh_chain_1_block_1().validate().unwrap();
        sample_eh_chain_1_block_27().validate().unwrap();

        sample_eh_chain_2_block_1().validate().unwrap();
        sample_eh_chain_2_block_27().validate().unwrap();
        sample_eh_chain_2_block_28().validate().unwrap();
        sample_eh_chain_2_block_35().validate().unwrap();
    }

    #[test]
    fn validate_validator_hash_mismatch() {
        let mut eh = sample_eh_chain_1_block_27();
        eh.header.validators_hash = Hash::None;

        eh.validate().unwrap_err();
    }

    #[test]
    fn validate_dah_hash_mismatch() {
        let mut eh = sample_eh_chain_1_block_27();
        eh.dah.hash = [0; 32];

        eh.validate().unwrap_err();
    }

    #[test]
    fn validate_commit_height_mismatch() {
        let mut eh = sample_eh_chain_1_block_27();
        eh.commit.height = 0xdeadbeefu32.into();

        eh.validate().unwrap_err();
    }

    #[test]
    fn validate_commit_block_hash_mismatch() {
        let mut eh = sample_eh_chain_1_block_27();
        eh.commit.block_id.hash = Hash::None;

        eh.validate().unwrap_err();
    }

    #[test]
    fn verify() {
        let eh_block_1 = sample_eh_chain_1_block_1();
        let eh_block_27 = sample_eh_chain_1_block_27();

        eh_block_1.verify(&eh_block_27).unwrap();

        let eh_block_1 = sample_eh_chain_2_block_1();
        let eh_block_27 = sample_eh_chain_2_block_27();

        eh_block_1.verify(&eh_block_27).unwrap();
    }

    #[test]
    fn verify_adjacent() {
        let eh_block_27 = sample_eh_chain_2_block_27();
        let eh_block_28 = sample_eh_chain_2_block_28();

        eh_block_27.verify(&eh_block_28).unwrap();
    }

    #[test]
    fn verify_invalid_validator() {
        let eh_block_27 = sample_eh_chain_2_block_27();
        let mut eh_block_28 = sample_eh_chain_2_block_28();

        eh_block_28.header.validators_hash = Hash::None;

        eh_block_27.verify(&eh_block_28).unwrap_err();
    }

    #[test]
    fn verify_invalid_last_block_hash() {
        let eh_block_27 = sample_eh_chain_2_block_27();
        let mut eh_block_28 = sample_eh_chain_2_block_28();

        eh_block_28.header.last_block_id.as_mut().unwrap().hash = Hash::None;

        eh_block_27.verify(&eh_block_28).unwrap_err();
    }

    #[test]
    fn verify_invalid_adjacent() {
        let eh_block_27 = sample_eh_chain_1_block_27();
        let eh_block_28 = sample_eh_chain_2_block_28();

        eh_block_27.verify(&eh_block_28).unwrap_err();
    }

    #[test]
    fn verify_same_chain_id_but_different_chain() {
        let eh_block_1 = sample_eh_chain_1_block_1();
        let eh_block_27 = sample_eh_chain_2_block_27();

        eh_block_1.verify(&eh_block_27).unwrap_err();
    }

    #[test]
    fn verify_invalid_height() {
        let eh_block_27 = sample_eh_chain_1_block_27();
        eh_block_27.verify(&eh_block_27).unwrap_err();
    }

    #[test]
    fn verify_invalid_chain_id() {
        let eh_block_1 = sample_eh_chain_1_block_1();
        let mut eh_block_27 = sample_eh_chain_1_block_27();

        eh_block_27.header.chain_id = "1112222".parse().unwrap();
        eh_block_1.verify(&eh_block_27).unwrap_err();
    }

    #[test]
    fn verify_invalid_time() {
        let eh_block_1 = sample_eh_chain_1_block_1();
        let mut eh_block_27 = sample_eh_chain_1_block_27();

        eh_block_27.header.time = eh_block_1.header.time;
        eh_block_1.verify(&eh_block_27).unwrap_err();
    }

    #[test]
    fn verify_time_from_the_future() {
        let eh_block_1 = sample_eh_chain_1_block_1();
        let mut eh_block_27 = sample_eh_chain_1_block_27();

        eh_block_27.header.time = Time::now().checked_add(Duration::from_secs(60)).unwrap();
        eh_block_1.verify(&eh_block_27).unwrap_err();
    }
}
