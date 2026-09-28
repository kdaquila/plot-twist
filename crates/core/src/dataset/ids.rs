use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use ts_rs::TS;

/// Identifies one successful load. Serialized as an opaque string (`"ds-7"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, TS)]
#[ts(export)]
pub struct DatasetId(#[ts(type = "string")] u64);

impl DatasetId {
    pub fn new(n: u64) -> Self {
        Self(n)
    }
}

impl fmt::Display for DatasetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ds-{}", self.0)
    }
}

impl FromStr for DatasetId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix("ds-")
            .and_then(|n| n.parse().ok())
            .map(Self)
            .ok_or(())
    }
}

impl Serialize for DatasetId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DatasetId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse()
            .map_err(|()| serde::de::Error::custom(format!("invalid dataset id \"{text}\"")))
    }
}

/// 0-based column position in the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ColumnIndex(pub u32);

/// 1-based line number in the file as seen in a text editor (header is line 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LineNumber(#[ts(type = "number")] pub u64);
