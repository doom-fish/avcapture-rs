#![allow(clippy::must_use_candidate)]

use serde::{Deserialize, Serialize};

/// Physical position of a capture device on the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[non_exhaustive]
pub enum CaptureDevicePosition {
    Unspecified,
    Back,
    Front,
    Unknown(i32),
}

impl CaptureDevicePosition {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Unspecified,
            1 => Self::Back,
            2 => Self::Front,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Unspecified => 0,
            Self::Back => 1,
            Self::Front => 2,
            Self::Unknown(raw) => raw,
        }
    }
}

impl From<i32> for CaptureDevicePosition {
    fn from(value: i32) -> Self {
        Self::from_raw(value)
    }
}

impl From<CaptureDevicePosition> for i32 {
    fn from(value: CaptureDevicePosition) -> Self {
        value.as_raw()
    }
}
