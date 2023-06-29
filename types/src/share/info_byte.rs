use crate::consts::appconsts;
use crate::{Error, Result};

/// InfoByte is a byte with the following structure: the first 7 bits are
/// reserved for version information in big endian form (initially `0000000`).
/// The last bit is a "sequence start indicator", that is `1` if this is the
/// first share of a sequence and `0` if this is a continuation share.
#[repr(transparent)]
pub struct InfoByte(u8);

impl InfoByte {
    pub fn new(version: u8, is_sequence_start: bool) -> Result<Self> {
        if version > appconsts::MAX_SHARE_VERSION {
            Err(Error::MaxShareVersionExceeded(version))
        } else {
            let prefix = version << 1;
            let sequence_start = if is_sequence_start { 1 } else { 0 };
            Ok(Self(prefix + sequence_start))
        }
    }

    pub fn version(&self) -> u8 {
        self.0 >> 1
    }

    pub fn is_sequence_start(&self) -> bool {
        self.0 % 2 == 1
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }
}
