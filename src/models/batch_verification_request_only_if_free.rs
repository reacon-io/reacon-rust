// Copyright Reacon contributors. Licensed under Apache-2.0.
use serde::{Deserialize, Serialize};

/// Scalar union accepted by verification requests. Preserves boolean vs string.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BatchVerificationRequestOnlyIfFree {
    Boolean(bool),
    String(OnlyIfFreeString),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OnlyIfFreeString {
    #[serde(rename = "true")]
    True,
    #[serde(rename = "false")]
    False,
}

impl From<bool> for BatchVerificationRequestOnlyIfFree {
    fn from(value: bool) -> Self { Self::Boolean(value) }
}

impl TryFrom<&str> for BatchVerificationRequestOnlyIfFree {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "true" => Ok(Self::String(OnlyIfFreeString::True)),
            "false" => Ok(Self::String(OnlyIfFreeString::False)),
            _ => Err("Expected the string true or false"),
        }
    }
}
