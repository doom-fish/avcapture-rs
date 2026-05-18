#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};
use core::ptr;
use std::ffi::CString;

use apple_cf::cm::CMTime;
use serde::{Deserialize, Serialize};

use crate::device_format::CaptureDeviceFormat;
use crate::device_position::CaptureDevicePosition;
use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{cm_time_serde, cstring, parse_json_and_free};
use crate::session::CaptureSessionPreset;

macro_rules! raw_i32_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($(#[$variant_meta:meta])* $variant:ident = $raw:expr),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(from = "i32", into = "i32")]
        #[non_exhaustive]
        $vis enum $name {
            $($(#[$variant_meta])*
            #[doc = concat!("Corresponds to the `", stringify!($variant), "` case.")]
            $variant,)+
            #[doc = "A value not recognized by this crate."]
            Unknown(i32),
        }

        impl $name {
            #[doc = "Creates the enum from its raw SDK value."]
            #[must_use]
            /// Wraps an existing `AVCapture` pointer.
            pub const fn from_raw(raw: i32) -> Self {
                match raw {
                    $($raw => Self::$variant,)+
                    other => Self::Unknown(other),
                }
            }

            #[doc = "Returns the raw SDK value."]
            #[must_use]
            /// Returns the raw SDK value for `AVCapture`.
            pub const fn as_raw(self) -> i32 {
                match self {
                    $(Self::$variant => $raw,)+
                    Self::Unknown(raw) => raw,
                }
            }
        }

        impl From<i32> for $name {
            fn from(value: i32) -> Self {
                Self::from_raw(value)
            }
        }

        impl From<$name> for i32 {
            fn from(value: $name) -> Self {
                value.as_raw()
            }
        }
    };
}

macro_rules! raw_string_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($(#[$variant_meta:meta])* $variant:ident = $raw:expr),+ $(,)? }) => {
        $(#[$meta])*
        #[allow(clippy::unsafe_derive_deserialize)]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(from = "String", into = "String")]
        #[non_exhaustive]
        $vis enum $name {
            $($(#[$variant_meta])*
            #[doc = concat!("Corresponds to the `", stringify!($variant), "` case.")]
            $variant,)+
            #[doc = "A value not recognized by this crate."]
            Unknown(String),
        }

        impl $name {
            #[doc = "Returns the raw SDK value."]
            #[must_use]
            /// Returns the raw SDK value for `AVCapture`.
            pub fn as_raw(&self) -> &str {
                match self {
                    $(Self::$variant => $raw,)+
                    Self::Unknown(raw) => raw.as_str(),
                }
            }

            #[doc = "Creates the enum from its raw SDK value."]
            #[must_use]
            /// Wraps an existing `AVCapture` pointer.
            pub fn from_raw(raw: &str) -> Self {
                match raw {
                    $($raw => Self::$variant,)+
                    other => Self::Unknown(other.to_owned()),
                }
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::from_raw(&value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::from_raw(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.as_raw().to_owned()
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
#[non_exhaustive]
/// `AVMediaType` values.
pub enum MediaType {
    /// Corresponds to the `Audio` case.
    Audio,
    /// Corresponds to the `Video` case.
    Video,
    /// Corresponds to the `Muxed` case.
    Muxed,
    /// Corresponds to the `Metadata` case.
    Metadata,
    /// A value not recognized by this crate.
    Unknown(String),
}

impl MediaType {
    #[must_use]
    /// Returns the raw SDK value for `AVMediaType`.
    pub fn as_raw(&self) -> &str {
        match self {
            Self::Audio => "audio",
            Self::Video => "video",
            Self::Muxed => "muxed",
            Self::Metadata => "metadata",
            Self::Unknown(raw) => raw.as_str(),
        }
    }

    #[must_use]
    /// Wraps an existing `AVMediaType` pointer.
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "audio" => Self::Audio,
            "video" => Self::Video,
            "muxed" => Self::Muxed,
            "metadata" => Self::Metadata,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl From<String> for MediaType {
    fn from(value: String) -> Self {
        Self::from_raw(&value)
    }
}

impl From<MediaType> for String {
    fn from(value: MediaType) -> Self {
        value.as_raw().to_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// `AVAuthorizationStatus` values.
pub enum AuthorizationStatus {
    /// Corresponds to the `NotDetermined` case.
    NotDetermined,
    /// Corresponds to the `Restricted` case.
    Restricted,
    /// Corresponds to the `Denied` case.
    Denied,
    /// Corresponds to the `Authorized` case.
    Authorized,
    /// Corresponds to the `Limited` case.
    Limited,
    /// A value not recognized by this crate.
    Unknown,
}

impl AuthorizationStatus {
    #[must_use]
    /// Wraps an existing `AVAuthorizationStatus` pointer.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::NotDetermined,
            1 => Self::Restricted,
            2 => Self::Denied,
            3 => Self::Authorized,
            4 => Self::Limited,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
#[non_exhaustive]
/// `AVCaptureDeviceType` values.
pub enum CaptureDeviceType {
    /// Corresponds to the `External` case.
    External,
    /// Corresponds to the `Microphone` case.
    Microphone,
    /// Corresponds to the `BuiltInWideAngleCamera` case.
    BuiltInWideAngleCamera,
    /// Corresponds to the `ContinuityCamera` case.
    ContinuityCamera,
    /// Corresponds to the `DeskViewCamera` case.
    DeskViewCamera,
    /// A value not recognized by this crate.
    Unknown(String),
}

impl CaptureDeviceType {
    #[must_use]
    /// Returns the raw SDK value for `AVCaptureDeviceType`.
    pub fn as_raw(&self) -> &str {
        match self {
            Self::External => "AVCaptureDeviceTypeExternal",
            Self::Microphone => "AVCaptureDeviceTypeMicrophone",
            Self::BuiltInWideAngleCamera => "AVCaptureDeviceTypeBuiltInWideAngleCamera",
            Self::ContinuityCamera => "AVCaptureDeviceTypeContinuityCamera",
            Self::DeskViewCamera => "AVCaptureDeviceTypeDeskViewCamera",
            Self::Unknown(raw) => raw.as_str(),
        }
    }

    #[must_use]
    /// Wraps an existing `AVCaptureDeviceType` pointer.
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "AVCaptureDeviceTypeExternal" | "AVCaptureDeviceTypeExternalUnknown" => Self::External,
            "AVCaptureDeviceTypeMicrophone" | "AVCaptureDeviceTypeBuiltInMicrophone" => {
                Self::Microphone
            }
            "AVCaptureDeviceTypeBuiltInWideAngleCamera" => Self::BuiltInWideAngleCamera,
            "AVCaptureDeviceTypeContinuityCamera" => Self::ContinuityCamera,
            "AVCaptureDeviceTypeDeskViewCamera" => Self::DeskViewCamera,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl From<String> for CaptureDeviceType {
    fn from(value: String) -> Self {
        Self::from_raw(&value)
    }
}

impl From<CaptureDeviceType> for String {
    fn from(value: CaptureDeviceType) -> Self {
        value.as_raw().to_owned()
    }
}

raw_i32_enum! {
    /// `AVCaptureFlashMode` values.
    pub enum CaptureFlashMode {
        /// Corresponds to the `Off` case.
        Off = 0,
        /// Corresponds to the `On` case.
        On = 1,
        /// Corresponds to the `Auto` case.
        Auto = 2,
    }
}

raw_i32_enum! {
    /// `AVCaptureTorchMode` values.
    pub enum CaptureTorchMode {
        /// Corresponds to the `Off` case.
        Off = 0,
        /// Corresponds to the `On` case.
        On = 1,
        /// Corresponds to the `Auto` case.
        Auto = 2,
    }
}

raw_i32_enum! {
    /// `AVCaptureExposureMode` values.
    pub enum CaptureExposureMode {
        /// Corresponds to the `Locked` case.
        Locked = 0,
        /// Corresponds to the `AutoExpose` case.
        AutoExpose = 1,
        /// Corresponds to the `ContinuousAutoExposure` case.
        ContinuousAutoExposure = 2,
        /// Corresponds to the `Custom` case.
        Custom = 3,
    }
}

raw_i32_enum! {
    /// `AVCaptureFocusMode` values.
    pub enum CaptureFocusMode {
        /// Corresponds to the `Locked` case.
        Locked = 0,
        /// Corresponds to the `AutoFocus` case.
        AutoFocus = 1,
        /// Corresponds to the `ContinuousAutoFocus` case.
        ContinuousAutoFocus = 2,
    }
}

raw_i32_enum! {
    /// `AVCaptureWhiteBalanceMode` values.
    pub enum CaptureWhiteBalanceMode {
        /// Corresponds to the `Locked` case.
        Locked = 0,
        /// Corresponds to the `AutoWhiteBalance` case.
        AutoWhiteBalance = 1,
        /// Corresponds to the `ContinuousAutoWhiteBalance` case.
        ContinuousAutoWhiteBalance = 2,
    }
}

raw_i32_enum! {
    /// `AVCaptureAutoFocusSystem` values.
    pub enum CaptureAutoFocusSystem {
        /// Corresponds to the `None` case.
        None = 0,
        /// Corresponds to the `ContrastDetection` case.
        ContrastDetection = 1,
        /// Corresponds to the `PhaseDetection` case.
        PhaseDetection = 2,
    }
}

raw_i32_enum! {
    /// `AVCaptureColorSpace` values.
    pub enum CaptureColorSpace {
        /// Corresponds to the `Srgb` case.
        Srgb = 0,
        /// Corresponds to the `P3D65` case.
        P3D65 = 1,
    }
}

raw_i32_enum! {
    /// `AVCaptureDeviceTransportControlsPlaybackMode` values.
    pub enum CaptureDeviceTransportControlsPlaybackMode {
        /// Corresponds to the `NotPlaying` case.
        NotPlaying = 0,
        /// Corresponds to the `Playing` case.
        Playing = 1,
    }
}

raw_i32_enum! {
    /// `AVCaptureCenterStageControlMode` values.
    pub enum CaptureCenterStageControlMode {
        /// Corresponds to the `User` case.
        User = 0,
        /// Corresponds to the `App` case.
        App = 1,
        /// Corresponds to the `Cooperative` case.
        Cooperative = 2,
    }
}

raw_i32_enum! {
    /// `AVCaptureCinematicVideoFocusMode` values.
    pub enum CaptureCinematicVideoFocusMode {
        /// Corresponds to the `None` case.
        None = 0,
        /// Corresponds to the `Strong` case.
        Strong = 1,
        /// Corresponds to the `Weak` case.
        Weak = 2,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
#[non_exhaustive]
/// `AVCaptureCameraLensSmudgeDetectionStatus` values.
pub enum CaptureCameraLensSmudgeDetectionStatus {
    /// Corresponds to the `Disabled` case.
    Disabled,
    /// Corresponds to the `SmudgeNotDetected` case.
    SmudgeNotDetected,
    /// Corresponds to the `Smudged` case.
    Smudged,
    /// A value not recognized by this crate.
    UnknownStatus,
    /// A value not recognized by this crate.
    Unknown(i32),
}

impl CaptureCameraLensSmudgeDetectionStatus {
    #[must_use]
    /// Wraps an existing `AVCaptureCameraLensSmudgeDetectionStatus` pointer.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            0 => Self::Disabled,
            1 => Self::SmudgeNotDetected,
            2 => Self::Smudged,
            3 => Self::UnknownStatus,
            other => Self::Unknown(other),
        }
    }

    #[must_use]
    /// Returns the raw SDK value for `AVCaptureCameraLensSmudgeDetectionStatus`.
    pub const fn as_raw(self) -> i32 {
        match self {
            Self::Disabled => 0,
            Self::SmudgeNotDetected => 1,
            Self::Smudged => 2,
            Self::UnknownStatus => 3,
            Self::Unknown(raw) => raw,
        }
    }
}

impl From<i32> for CaptureCameraLensSmudgeDetectionStatus {
    fn from(value: i32) -> Self {
        Self::from_raw(value)
    }
}

impl From<CaptureCameraLensSmudgeDetectionStatus> for i32 {
    fn from(value: CaptureCameraLensSmudgeDetectionStatus) -> Self {
        value.as_raw()
    }
}

raw_i32_enum! {
    /// `AVCaptureMicrophoneMode` values.
    pub enum CaptureMicrophoneMode {
        /// Corresponds to the `Standard` case.
        Standard = 0,
        /// Corresponds to the `WideSpectrum` case.
        WideSpectrum = 1,
        /// Corresponds to the `VoiceIsolation` case.
        VoiceIsolation = 2,
    }
}

raw_i32_enum! {
    /// `AVCapturePrimaryConstituentDeviceSwitchingBehavior` values.
    pub enum CapturePrimaryConstituentDeviceSwitchingBehavior {
        /// Corresponds to the `Unsupported` case.
        Unsupported = 0,
        /// Corresponds to the `Auto` case.
        Auto = 1,
        /// Corresponds to the `Restricted` case.
        Restricted = 2,
        /// Corresponds to the `Locked` case.
        Locked = 3,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Bitflags wrapping `AVCapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions`.
pub struct CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions(u64);

impl CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions {
    /// No flags are set.
    pub const NONE: Self = Self(0);
    /// Flag indicating that video zoom changed.
    pub const VIDEO_ZOOM_CHANGED: Self = Self(1 << 0);
    /// Flag indicating that the focus mode changed.
    pub const FOCUS_MODE_CHANGED: Self = Self(1 << 1);
    /// Flag indicating that the exposure mode changed.
    pub const EXPOSURE_MODE_CHANGED: Self = Self(1 << 2);

    #[must_use]
    /// Wraps an existing `AVCapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions` pointer.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[must_use]
    /// Returns the raw SDK value for `AVCapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions`.
    pub const fn as_raw(self) -> u64 {
        self.0
    }

    #[must_use]
    /// Corresponds to `AVCapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions.contains`.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl Default for CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions {
    fn default() -> Self {
        Self::NONE
    }
}

impl BitOr for CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl From<u64> for CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions {
    fn from(value: u64) -> Self {
        Self::from_raw(value)
    }
}

impl From<CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions> for u64 {
    fn from(value: CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions) -> Self {
        value.as_raw()
    }
}

raw_i32_enum! {
    /// `AVCaptureSystemUserInterface` values.
    pub enum CaptureSystemUserInterface {
        /// Corresponds to the `VideoEffects` case.
        VideoEffects = 1,
        /// Corresponds to the `MicrophoneModes` case.
        MicrophoneModes = 2,
    }
}

raw_string_enum! {
    /// `AVCaptureSceneMonitoringStatus` values.
    pub enum CaptureSceneMonitoringStatus {
        /// Corresponds to the `NotEnoughLight` case.
        NotEnoughLight = "AVCaptureSceneMonitoringStatusNotEnoughLight",
    }
}

raw_string_enum! {
    /// `AVCaptureReactionType` values.
    pub enum CaptureReactionType {
        /// Corresponds to the `ThumbsUp` case.
        ThumbsUp = "ReactionThumbsUp",
        /// Corresponds to the `ThumbsDown` case.
        ThumbsDown = "ReactionThumbsDown",
        /// Corresponds to the `Balloons` case.
        Balloons = "ReactionBalloons",
        /// Corresponds to the `Heart` case.
        Heart = "ReactionHeart",
        /// Corresponds to the `Fireworks` case.
        Fireworks = "ReactionFireworks",
        /// Corresponds to the `Rain` case.
        Rain = "ReactionRain",
        /// Corresponds to the `Confetti` case.
        Confetti = "ReactionConfetti",
        /// Corresponds to the `Lasers` case.
        Lasers = "ReactionLasers",
    }
}

impl CaptureReactionType {
    /// Returns the system image name associated with the reaction type.
    pub fn system_image_name(&self) -> Result<String, AVCaptureError> {
        let reaction_type = cstring(self.as_raw(), "reaction type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let string_ptr = unsafe {
            ffi::device::av_capture_reaction_system_image_name_for_type(
                reaction_type.as_ptr(),
                &mut err,
            )
        };
        if string_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        let value: String = parse_json_string_and_free(string_ptr);
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureDevice` state.
pub struct CaptureDeviceInfo {
    /// The unique id reported by `AVCaptureDevice`.
    pub unique_id: String,
    /// The localized name reported by `AVCaptureDevice`.
    pub localized_name: String,
    /// The manufacturer reported by `AVCaptureDevice`.
    pub manufacturer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureDeviceInputSource` state.
pub struct CaptureDeviceInputSourceInfo {
    /// The input source id reported by `AVCaptureDeviceInputSource`.
    pub input_source_id: String,
    /// The localized name reported by `AVCaptureDeviceInputSource`.
    pub localized_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `AVCaptureReactionEffectState`.
pub struct CaptureReactionEffectState {
    /// The reaction type reported by `AVCaptureReactionEffectState`.
    pub reaction_type: CaptureReactionType,
    #[serde(with = "cm_time_serde")]
    /// The start time reported by `AVCaptureReactionEffectState`.
    pub start_time: CMTime,
    #[serde(with = "cm_time_serde")]
    /// The end time reported by `AVCaptureReactionEffectState`.
    pub end_time: CMTime,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCaptureDeviceRotationCoordinator` state.
pub struct CaptureDeviceRotationCoordinatorInfo {
    /// The video rotation angle for horizon level preview reported by `AVCaptureDeviceRotationCoordinator`.
    pub video_rotation_angle_for_horizon_level_preview: f64,
    /// The video rotation angle for horizon level capture reported by `AVCaptureDeviceRotationCoordinator`.
    pub video_rotation_angle_for_horizon_level_capture: f64,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Wraps `AVCaptureDevice`.
pub struct CaptureDeviceDetails {
    /// The unique id reported by `AVCaptureDevice`.
    pub unique_id: String,
    /// The localized name reported by `AVCaptureDevice`.
    pub localized_name: String,
    /// The manufacturer reported by `AVCaptureDevice`.
    pub manufacturer: String,
    /// The transport type reported by `AVCaptureDevice`.
    pub transport_type: Option<i32>,
    /// The media types reported by `AVCaptureDevice`.
    pub media_types: Vec<MediaType>,
    /// The position reported by `AVCaptureDevice`.
    pub position: CaptureDevicePosition,
    /// The device type reported by `AVCaptureDevice`.
    pub device_type: CaptureDeviceType,
    /// The has flash reported by `AVCaptureDevice`.
    pub has_flash: bool,
    /// The flash available reported by `AVCaptureDevice`.
    pub flash_available: bool,
    /// The has torch reported by `AVCaptureDevice`.
    pub has_torch: bool,
    /// The torch available reported by `AVCaptureDevice`.
    pub torch_available: bool,
    /// The torch level reported by `AVCaptureDevice`.
    pub torch_level: Option<f32>,
    /// The exposure mode reported by `AVCaptureDevice`.
    pub exposure_mode: Option<CaptureExposureMode>,
    /// The formats count reported by `AVCaptureDevice`.
    pub formats_count: usize,
    #[serde(with = "cm_time_serde")]
    /// The active video min frame duration reported by `AVCaptureDevice`.
    pub active_video_min_frame_duration: CMTime,
    #[serde(with = "cm_time_serde")]
    /// The active video max frame duration reported by `AVCaptureDevice`.
    pub active_video_max_frame_duration: CMTime,
    #[serde(default)]
    /// The focus mode reported by `AVCaptureDevice`.
    pub focus_mode: Option<CaptureFocusMode>,
    #[serde(default)]
    /// The white balance mode reported by `AVCaptureDevice`.
    pub white_balance_mode: Option<CaptureWhiteBalanceMode>,
    #[serde(default)]
    /// The auto focus system reported by `AVCaptureDevice`.
    pub auto_focus_system: Option<CaptureAutoFocusSystem>,
    #[serde(default)]
    /// The active color space reported by `AVCaptureDevice`.
    pub active_color_space: Option<CaptureColorSpace>,
    #[serde(default)]
    /// The supported color spaces reported by `AVCaptureDevice`.
    pub supported_color_spaces: Vec<CaptureColorSpace>,
    #[serde(default)]
    /// The transport controls supported reported by `AVCaptureDevice`.
    pub transport_controls_supported: bool,
    #[serde(default)]
    /// The transport controls playback mode reported by `AVCaptureDevice`.
    pub transport_controls_playback_mode: Option<CaptureDeviceTransportControlsPlaybackMode>,
    #[serde(default)]
    /// The transport controls speed reported by `AVCaptureDevice`.
    pub transport_controls_speed: Option<f32>,
    #[serde(default)]
    /// The input sources reported by `AVCaptureDevice`.
    pub input_sources: Vec<CaptureDeviceInputSourceInfo>,
    #[serde(default)]
    /// The active input source id reported by `AVCaptureDevice`.
    pub active_input_source_id: Option<String>,
    #[serde(default)]
    /// The primary constituent device switching behavior reported by `AVCaptureDevice`.
    pub primary_constituent_device_switching_behavior:
        Option<CapturePrimaryConstituentDeviceSwitchingBehavior>,
    #[serde(default)]
    /// The primary constituent device restricted switching behavior conditions reported by `AVCaptureDevice`.
    pub primary_constituent_device_restricted_switching_behavior_conditions:
        Option<CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions>,
    #[serde(default)]
    /// The active primary constituent device switching behavior reported by `AVCaptureDevice`.
    pub active_primary_constituent_device_switching_behavior:
        Option<CapturePrimaryConstituentDeviceSwitchingBehavior>,
    #[serde(default)]
    /// The active primary constituent device restricted switching behavior conditions reported by `AVCaptureDevice`.
    pub active_primary_constituent_device_restricted_switching_behavior_conditions:
        Option<CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions>,
    #[serde(default)]
    /// The center stage control mode reported by `AVCaptureDevice`.
    pub center_stage_control_mode: Option<CaptureCenterStageControlMode>,
    #[serde(default)]
    /// The center stage enabled reported by `AVCaptureDevice`.
    pub center_stage_enabled: Option<bool>,
    #[serde(default)]
    /// The center stage active reported by `AVCaptureDevice`.
    pub center_stage_active: Option<bool>,
    #[serde(default)]
    /// The preferred microphone mode reported by `AVCaptureDevice`.
    pub preferred_microphone_mode: Option<CaptureMicrophoneMode>,
    #[serde(default)]
    /// The active microphone mode reported by `AVCaptureDevice`.
    pub active_microphone_mode: Option<CaptureMicrophoneMode>,
    #[serde(default)]
    /// The reaction effects enabled reported by `AVCaptureDevice`.
    pub reaction_effects_enabled: Option<bool>,
    #[serde(default)]
    /// The reaction effect gestures enabled reported by `AVCaptureDevice`.
    pub reaction_effect_gestures_enabled: Option<bool>,
    #[serde(default)]
    /// The can perform reaction effects reported by `AVCaptureDevice`.
    pub can_perform_reaction_effects: Option<bool>,
    #[serde(default)]
    /// The available reaction types reported by `AVCaptureDevice`.
    pub available_reaction_types: Vec<CaptureReactionType>,
    #[serde(default)]
    /// The reaction effects in progress reported by `AVCaptureDevice`.
    pub reaction_effects_in_progress: Vec<CaptureReactionEffectState>,
    #[serde(default)]
    /// The camera lens smudge detection enabled reported by `AVCaptureDevice`.
    pub camera_lens_smudge_detection_enabled: Option<bool>,
    #[serde(with = "cm_time_serde")]
    /// The camera lens smudge detection interval reported by `AVCaptureDevice`.
    pub camera_lens_smudge_detection_interval: CMTime,
    #[serde(default)]
    /// The camera lens smudge detection status reported by `AVCaptureDevice`.
    pub camera_lens_smudge_detection_status: Option<CaptureCameraLensSmudgeDetectionStatus>,
    #[serde(default)]
    /// The cinematic video capture scene monitoring statuses reported by `AVCaptureDevice`.
    pub cinematic_video_capture_scene_monitoring_statuses: Vec<CaptureSceneMonitoringStatus>,
}

/// Safe wrapper around `AVCaptureDevice`.
#[derive(Debug)]
/// Wraps `AVCaptureDevice`.
pub struct CaptureDevice {
    pub(crate) ptr: *mut c_void,
}

impl Drop for CaptureDevice {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::device::av_capture_device_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

/// Safe wrapper around `AVCaptureDeviceInputSource`.
#[derive(Debug)]
/// Wraps `AVCaptureDeviceInputSource`.
pub struct CaptureDeviceInputSource {
    ptr: *mut c_void,
}

impl Drop for CaptureDeviceInputSource {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::device::av_capture_device_input_source_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureDeviceInputSource {
    /// Wraps an existing `AVCaptureDeviceInputSource` pointer.
    pub const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns a snapshot of `AVCaptureDeviceInputSource` state.
    pub fn info(&self) -> Result<CaptureDeviceInputSourceInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::device::av_capture_device_input_source_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureDeviceInputSource.input_source_id`.
    pub fn input_source_id(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.input_source_id)
    }

    /// Corresponds to `AVCaptureDeviceInputSource.localized_name`.
    pub fn localized_name(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.localized_name)
    }
}

/// Safe wrapper around `AVCaptureDeviceRotationCoordinator`.
#[derive(Debug)]
/// Wraps `AVCaptureDeviceRotationCoordinator`.
pub struct CaptureDeviceRotationCoordinator {
    ptr: *mut c_void,
}

impl Drop for CaptureDeviceRotationCoordinator {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::device::av_capture_device_rotation_coordinator_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

impl CaptureDeviceRotationCoordinator {
    /// Wraps an existing `AVCaptureDeviceRotationCoordinator` pointer.
    pub const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns a snapshot of `AVCaptureDeviceRotationCoordinator` state.
    pub fn info(&self) -> Result<CaptureDeviceRotationCoordinatorInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::device::av_capture_device_rotation_coordinator_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureDeviceRotationCoordinator.video_rotation_angle_for_horizon_level_preview`.
    pub fn video_rotation_angle_for_horizon_level_preview(&self) -> Result<f64, AVCaptureError> {
        Ok(self.info()?.video_rotation_angle_for_horizon_level_preview)
    }

    /// Corresponds to `AVCaptureDeviceRotationCoordinator.video_rotation_angle_for_horizon_level_capture`.
    pub fn video_rotation_angle_for_horizon_level_capture(&self) -> Result<f64, AVCaptureError> {
        Ok(self.info()?.video_rotation_angle_for_horizon_level_capture)
    }
}

impl CaptureDevice {
    /// Notification name published by `AVCaptureDevice`.
    pub const WAS_CONNECTED_NOTIFICATION: &str = "AVCaptureDeviceWasConnectedNotification";
    /// Notification name published by `AVCaptureDevice`.
    pub const WAS_DISCONNECTED_NOTIFICATION: &str = "AVCaptureDeviceWasDisconnectedNotification";

    /// Returns the authorization status for the requested media type.
    pub fn authorization_status(
        media_type: &MediaType,
    ) -> Result<AuthorizationStatus, AVCaptureError> {
        let media_type = cstring(media_type.as_raw(), "media type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let raw =
            unsafe { ffi::device::av_capture_authorization_status(media_type.as_ptr(), &mut err) };
        if raw < 0 {
            return Err(unsafe { from_swift(raw, err) });
        }
        Ok(AuthorizationStatus::from_raw(raw))
    }

    /// Returns the devices matching the requested media type.
    pub fn devices(media_type: &MediaType) -> Result<Vec<CaptureDeviceInfo>, AVCaptureError> {
        let media_type = cstring(media_type.as_raw(), "media type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr =
            unsafe { ffi::device::av_capture_devices_json(media_type.as_ptr(), &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Returns the default device for the requested media type, if available.
    pub fn default(media_type: &MediaType) -> Result<Option<Self>, AVCaptureError> {
        let media_type = cstring(media_type.as_raw(), "media type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::device::av_capture_default_device(media_type.as_ptr(), &mut err) };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(Self { ptr }))
    }

    /// Returns the default device matching the requested type, media type, and position.
    pub fn default_with_device_type(
        device_type: &CaptureDeviceType,
        media_type: Option<&MediaType>,
        position: CaptureDevicePosition,
    ) -> Result<Option<Self>, AVCaptureError> {
        let device_type = cstring(device_type.as_raw(), "device type")?;
        let media_type = media_type
            .map(|value| cstring(value.as_raw(), "media type"))
            .transpose()?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::device::av_capture_default_device_for_type(
                device_type.as_ptr(),
                media_type
                    .as_ref()
                    .map_or(ptr::null(), |value| value.as_ptr()),
                position.as_raw(),
                &mut err,
            )
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(Self { ptr }))
    }

    /// Returns the device matching the supplied unique ID, if available.
    pub fn with_unique_id(unique_id: impl AsRef<str>) -> Result<Option<Self>, AVCaptureError> {
        let unique_id = CString::new(unique_id.as_ref()).map_err(|error| {
            AVCaptureError::InvalidArgument(format!("device unique ID contains NUL byte: {error}"))
        })?;
        let mut err: *mut c_char = ptr::null_mut();
        let ptr =
            unsafe { ffi::device::av_capture_device_with_unique_id(unique_id.as_ptr(), &mut err) };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(Self { ptr }))
    }

    /// Returns a snapshot of `AVCaptureDevice` state.
    pub fn info(&self) -> Result<CaptureDeviceInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::device::av_capture_device_info_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Returns the detailed state snapshot for `AVCaptureDevice`.
    pub fn details(&self) -> Result<CaptureDeviceDetails, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe { ffi::device::av_capture_device_details_json(self.ptr, &mut err) };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCaptureDevice.unique_id`.
    pub fn unique_id(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.unique_id)
    }

    /// Corresponds to `AVCaptureDevice.localized_name`.
    pub fn localized_name(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.localized_name)
    }

    /// Corresponds to `AVCaptureDevice.manufacturer`.
    pub fn manufacturer(&self) -> Result<String, AVCaptureError> {
        Ok(self.info()?.manufacturer)
    }

    /// Corresponds to `AVCaptureDevice.position`.
    pub fn position(&self) -> Result<CaptureDevicePosition, AVCaptureError> {
        Ok(self.details()?.position)
    }

    /// Corresponds to `AVCaptureDevice.device_type`.
    pub fn device_type(&self) -> Result<CaptureDeviceType, AVCaptureError> {
        Ok(self.details()?.device_type)
    }

    /// Corresponds to `AVCaptureDevice.media_types`.
    pub fn media_types(&self) -> Result<Vec<MediaType>, AVCaptureError> {
        Ok(self.details()?.media_types)
    }

    /// Corresponds to `AVCaptureDevice.transport_type`.
    pub fn transport_type(&self) -> Result<Option<i32>, AVCaptureError> {
        Ok(self.details()?.transport_type)
    }

    /// Returns whether `AVCaptureDevice` has flash.
    pub fn has_flash(&self) -> Result<bool, AVCaptureError> {
        Ok(self.details()?.has_flash)
    }

    /// Corresponds to `AVCaptureDevice.flash_available`.
    pub fn flash_available(&self) -> Result<bool, AVCaptureError> {
        Ok(self.details()?.flash_available)
    }

    /// Returns whether `AVCaptureDevice` has torch.
    pub fn has_torch(&self) -> Result<bool, AVCaptureError> {
        Ok(self.details()?.has_torch)
    }

    /// Corresponds to `AVCaptureDevice.torch_available`.
    pub fn torch_available(&self) -> Result<bool, AVCaptureError> {
        Ok(self.details()?.torch_available)
    }

    /// Corresponds to `AVCaptureDevice.torch_level`.
    pub fn torch_level(&self) -> Result<Option<f32>, AVCaptureError> {
        Ok(self.details()?.torch_level)
    }

    /// Corresponds to `AVCaptureDevice.exposure_mode`.
    pub fn exposure_mode(&self) -> Result<Option<CaptureExposureMode>, AVCaptureError> {
        Ok(self.details()?.exposure_mode)
    }

    /// Corresponds to `AVCaptureDevice.focus_mode`.
    pub fn focus_mode(&self) -> Result<Option<CaptureFocusMode>, AVCaptureError> {
        Ok(self.details()?.focus_mode)
    }

    /// Corresponds to `AVCaptureDevice.white_balance_mode`.
    pub fn white_balance_mode(&self) -> Result<Option<CaptureWhiteBalanceMode>, AVCaptureError> {
        Ok(self.details()?.white_balance_mode)
    }

    /// Corresponds to `AVCaptureDevice.auto_focus_system`.
    pub fn auto_focus_system(&self) -> Result<Option<CaptureAutoFocusSystem>, AVCaptureError> {
        Ok(self.details()?.auto_focus_system)
    }

    /// Corresponds to `AVCaptureDevice.active_color_space`.
    pub fn active_color_space(&self) -> Result<Option<CaptureColorSpace>, AVCaptureError> {
        Ok(self.details()?.active_color_space)
    }

    /// Corresponds to `AVCaptureDevice.supported_color_spaces`.
    pub fn supported_color_spaces(&self) -> Result<Vec<CaptureColorSpace>, AVCaptureError> {
        Ok(self.details()?.supported_color_spaces)
    }

    /// Corresponds to `AVCaptureDevice.transport_controls_supported`.
    pub fn transport_controls_supported(&self) -> Result<bool, AVCaptureError> {
        Ok(self.details()?.transport_controls_supported)
    }

    /// Corresponds to `AVCaptureDevice.transport_controls_playback_mode`.
    pub fn transport_controls_playback_mode(
        &self,
    ) -> Result<Option<CaptureDeviceTransportControlsPlaybackMode>, AVCaptureError> {
        Ok(self.details()?.transport_controls_playback_mode)
    }

    /// Corresponds to `AVCaptureDevice.transport_controls_speed`.
    pub fn transport_controls_speed(&self) -> Result<Option<f32>, AVCaptureError> {
        Ok(self.details()?.transport_controls_speed)
    }

    /// Corresponds to `AVCaptureDevice.input_source_infos`.
    pub fn input_source_infos(&self) -> Result<Vec<CaptureDeviceInputSourceInfo>, AVCaptureError> {
        Ok(self.details()?.input_sources)
    }

    /// Corresponds to `AVCaptureDevice.active_input_source_id`.
    pub fn active_input_source_id(&self) -> Result<Option<String>, AVCaptureError> {
        Ok(self.details()?.active_input_source_id)
    }

    /// Corresponds to `AVCaptureDevice.primary_constituent_device_switching_behavior`.
    pub fn primary_constituent_device_switching_behavior(
        &self,
    ) -> Result<Option<CapturePrimaryConstituentDeviceSwitchingBehavior>, AVCaptureError> {
        Ok(self
            .details()?
            .primary_constituent_device_switching_behavior)
    }

    /// Corresponds to `AVCaptureDevice.primary_constituent_device_restricted_switching_behavior_conditions`.
    pub fn primary_constituent_device_restricted_switching_behavior_conditions(
        &self,
    ) -> Result<
        Option<CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions>,
        AVCaptureError,
    > {
        Ok(self
            .details()?
            .primary_constituent_device_restricted_switching_behavior_conditions)
    }

    /// Corresponds to `AVCaptureDevice.active_primary_constituent_device_switching_behavior`.
    pub fn active_primary_constituent_device_switching_behavior(
        &self,
    ) -> Result<Option<CapturePrimaryConstituentDeviceSwitchingBehavior>, AVCaptureError> {
        Ok(self
            .details()?
            .active_primary_constituent_device_switching_behavior)
    }

    /// Corresponds to `AVCaptureDevice.active_primary_constituent_device_restricted_switching_behavior_conditions`.
    pub fn active_primary_constituent_device_restricted_switching_behavior_conditions(
        &self,
    ) -> Result<
        Option<CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions>,
        AVCaptureError,
    > {
        Ok(self
            .details()?
            .active_primary_constituent_device_restricted_switching_behavior_conditions)
    }

    /// Corresponds to `AVCaptureDevice.center_stage_control_mode`.
    pub fn center_stage_control_mode() -> Option<CaptureCenterStageControlMode> {
        enum_from_class_raw(unsafe { ffi::device::av_capture_device_center_stage_control_mode() })
    }

    /// Corresponds to `AVCaptureDevice.center_stage_enabled`.
    pub fn center_stage_enabled() -> Option<bool> {
        option_bool_from_raw(unsafe { ffi::device::av_capture_device_center_stage_enabled() })
    }

    /// Corresponds to `AVCaptureDevice.center_stage_active`.
    pub fn center_stage_active(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.details()?.center_stage_active)
    }

    /// Corresponds to `AVCaptureDevice.preferred_microphone_mode`.
    pub fn preferred_microphone_mode() -> Option<CaptureMicrophoneMode> {
        enum_from_class_raw(unsafe { ffi::device::av_capture_device_preferred_microphone_mode() })
    }

    /// Corresponds to `AVCaptureDevice.active_microphone_mode`.
    pub fn active_microphone_mode() -> Option<CaptureMicrophoneMode> {
        enum_from_class_raw(unsafe { ffi::device::av_capture_device_active_microphone_mode() })
    }

    /// Corresponds to `AVCaptureDevice.reaction_effects_enabled`.
    pub fn reaction_effects_enabled() -> Option<bool> {
        option_bool_from_raw(unsafe { ffi::device::av_capture_device_reaction_effects_enabled() })
    }

    /// Corresponds to `AVCaptureDevice.reaction_effect_gestures_enabled`.
    pub fn reaction_effect_gestures_enabled() -> Option<bool> {
        option_bool_from_raw(unsafe {
            ffi::device::av_capture_device_reaction_effect_gestures_enabled()
        })
    }

    /// Returns whether `AVCaptureDevice` can perform reaction effects.
    pub fn can_perform_reaction_effects(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.details()?.can_perform_reaction_effects)
    }

    /// Returns the available reaction types reported by `AVCaptureDevice`.
    pub fn available_reaction_types(&self) -> Result<Vec<CaptureReactionType>, AVCaptureError> {
        Ok(self.details()?.available_reaction_types)
    }

    /// Corresponds to `AVCaptureDevice.reaction_effects_in_progress`.
    pub fn reaction_effects_in_progress(
        &self,
    ) -> Result<Vec<CaptureReactionEffectState>, AVCaptureError> {
        Ok(self.details()?.reaction_effects_in_progress)
    }

    /// Corresponds to `AVCaptureDevice.scene_monitoring_status_not_enough_light`.
    pub const fn scene_monitoring_status_not_enough_light() -> CaptureSceneMonitoringStatus {
        CaptureSceneMonitoringStatus::NotEnoughLight
    }

    /// Corresponds to `AVCaptureDevice.cinematic_video_capture_scene_monitoring_statuses`.
    pub fn cinematic_video_capture_scene_monitoring_statuses(
        &self,
    ) -> Result<Vec<CaptureSceneMonitoringStatus>, AVCaptureError> {
        Ok(self
            .details()?
            .cinematic_video_capture_scene_monitoring_statuses)
    }

    /// Corresponds to `AVCaptureDevice.camera_lens_smudge_detection_enabled`.
    pub fn camera_lens_smudge_detection_enabled(&self) -> Result<Option<bool>, AVCaptureError> {
        Ok(self.details()?.camera_lens_smudge_detection_enabled)
    }

    /// Corresponds to `AVCaptureDevice.camera_lens_smudge_detection_interval`.
    pub fn camera_lens_smudge_detection_interval(&self) -> Result<CMTime, AVCaptureError> {
        Ok(self.details()?.camera_lens_smudge_detection_interval)
    }

    /// Corresponds to `AVCaptureDevice.camera_lens_smudge_detection_status`.
    pub fn camera_lens_smudge_detection_status(
        &self,
    ) -> Result<Option<CaptureCameraLensSmudgeDetectionStatus>, AVCaptureError> {
        Ok(self.details()?.camera_lens_smudge_detection_status)
    }

    /// Corresponds to `AVCaptureDevice.reaction_type_thumbs_up`.
    pub const fn reaction_type_thumbs_up() -> CaptureReactionType {
        CaptureReactionType::ThumbsUp
    }

    /// Corresponds to `AVCaptureDevice.reaction_type_thumbs_down`.
    pub const fn reaction_type_thumbs_down() -> CaptureReactionType {
        CaptureReactionType::ThumbsDown
    }

    /// Corresponds to `AVCaptureDevice.reaction_type_balloons`.
    pub const fn reaction_type_balloons() -> CaptureReactionType {
        CaptureReactionType::Balloons
    }

    /// Corresponds to `AVCaptureDevice.reaction_type_heart`.
    pub const fn reaction_type_heart() -> CaptureReactionType {
        CaptureReactionType::Heart
    }

    /// Corresponds to `AVCaptureDevice.reaction_type_fireworks`.
    pub const fn reaction_type_fireworks() -> CaptureReactionType {
        CaptureReactionType::Fireworks
    }

    /// Corresponds to `AVCaptureDevice.reaction_type_rain`.
    pub const fn reaction_type_rain() -> CaptureReactionType {
        CaptureReactionType::Rain
    }

    /// Corresponds to `AVCaptureDevice.reaction_type_confetti`.
    pub const fn reaction_type_confetti() -> CaptureReactionType {
        CaptureReactionType::Confetti
    }

    /// Corresponds to `AVCaptureDevice.reaction_type_lasers`.
    pub const fn reaction_type_lasers() -> CaptureReactionType {
        CaptureReactionType::Lasers
    }

    /// Corresponds to `AVCaptureDevice.perform_reaction_effect`.
    pub fn perform_reaction_effect(
        &self,
        reaction_type: impl Into<CaptureReactionType>,
    ) -> Result<(), AVCaptureError> {
        let reaction_type = cstring(reaction_type.into().as_raw(), "reaction type")?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_perform_reaction_effect(
                self.ptr,
                reaction_type.as_ptr(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Corresponds to `AVCaptureDevice.show_system_user_interface`.
    pub fn show_system_user_interface(
        system_user_interface: impl Into<CaptureSystemUserInterface>,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_show_system_user_interface(
                system_user_interface.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Corresponds to `AVCaptureDevice.max_available_torch_level`.
    pub fn max_available_torch_level() -> f32 {
        unsafe { ffi::device::av_capture_device_max_available_torch_level() }
    }

    /// Corresponds to `AVCaptureDevice.input_sources`.
    pub fn input_sources(&self) -> Result<Vec<CaptureDeviceInputSource>, AVCaptureError> {
        let count = unsafe { ffi::device::av_capture_device_input_sources_count(self.ptr) };
        let mut input_sources = Vec::with_capacity(count);
        for index in 0..count {
            let mut err: *mut c_char = ptr::null_mut();
            let ptr = unsafe {
                ffi::device::av_capture_device_input_source_at_index(self.ptr, index, &mut err)
            };
            if ptr.is_null() {
                return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
            }
            input_sources.push(CaptureDeviceInputSource::from_raw(ptr));
        }
        Ok(input_sources)
    }

    /// Corresponds to `AVCaptureDevice.active_input_source`.
    pub fn active_input_source(&self) -> Result<Option<CaptureDeviceInputSource>, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::device::av_capture_device_active_input_source(self.ptr, &mut err) };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(CaptureDeviceInputSource::from_raw(ptr)))
    }

    /// Corresponds to `AVCaptureDevice.rotation_coordinator`.
    pub fn rotation_coordinator(
        &self,
    ) -> Result<Option<CaptureDeviceRotationCoordinator>, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe {
            ffi::device::av_capture_device_rotation_coordinator_create(self.ptr, &mut err)
        };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(CaptureDeviceRotationCoordinator::from_raw(ptr)))
    }

    /// Returns whether `AVCaptureDevice` is exposure mode supported.
    pub fn is_exposure_mode_supported(&self, mode: CaptureExposureMode) -> bool {
        unsafe {
            ffi::device::av_capture_device_is_exposure_mode_supported(self.ptr, mode.as_raw())
        }
    }

    /// Returns whether `AVCaptureDevice` is focus mode supported.
    pub fn is_focus_mode_supported(&self, mode: impl Into<CaptureFocusMode>) -> bool {
        unsafe {
            ffi::device::av_capture_device_is_focus_mode_supported(self.ptr, mode.into().as_raw())
        }
    }

    /// Returns whether `AVCaptureDevice` is white balance mode supported.
    pub fn is_white_balance_mode_supported(
        &self,
        mode: impl Into<CaptureWhiteBalanceMode>,
    ) -> bool {
        unsafe {
            ffi::device::av_capture_device_is_white_balance_mode_supported(
                self.ptr,
                mode.into().as_raw(),
            )
        }
    }

    /// Returns the formats count reported by `AVCaptureDevice`.
    pub fn formats_count(&self) -> Result<usize, AVCaptureError> {
        Ok(self.details()?.formats_count)
    }

    /// Returns whether `AVCaptureDevice` supports session preset.
    pub fn supports_session_preset(
        &self,
        preset: &CaptureSessionPreset,
    ) -> Result<bool, AVCaptureError> {
        let preset = preset_cstring(preset)?;
        Ok(unsafe {
            ffi::device::av_capture_device_supports_session_preset(self.ptr, preset.as_ptr())
        })
    }

    /// Corresponds to `AVCaptureDevice.formats`.
    pub fn formats(&self) -> Result<Vec<CaptureDeviceFormat>, AVCaptureError> {
        let count = unsafe { ffi::device::av_capture_device_formats_count(self.ptr) };
        let mut formats = Vec::with_capacity(count);
        for index in 0..count {
            let mut err: *mut c_char = ptr::null_mut();
            let ptr = unsafe {
                ffi::device::av_capture_device_format_at_index(self.ptr, index, &mut err)
            };
            if ptr.is_null() {
                return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
            }
            formats.push(CaptureDeviceFormat::from_raw(ptr));
        }
        Ok(formats)
    }

    /// Corresponds to `AVCaptureDevice.active_format`.
    pub fn active_format(&self) -> Result<Option<CaptureDeviceFormat>, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let ptr = unsafe { ffi::device::av_capture_device_active_format(self.ptr, &mut err) };
        if ptr.is_null() {
            if err.is_null() {
                return Ok(None);
            }
            return Err(unsafe { from_swift(ffi::status::DEVICE_ERROR, err) });
        }
        Ok(Some(CaptureDeviceFormat::from_raw(ptr)))
    }

    /// Corresponds to `AVCaptureDevice.active_video_min_frame_duration`.
    pub fn active_video_min_frame_duration(&self) -> CMTime {
        unsafe { ffi::device::av_capture_device_active_video_min_frame_duration(self.ptr) }
    }

    /// Corresponds to `AVCaptureDevice.active_video_max_frame_duration`.
    pub fn active_video_max_frame_duration(&self) -> CMTime {
        unsafe { ffi::device::av_capture_device_active_video_max_frame_duration(self.ptr) }
    }

    /// Locks `AVCaptureDevice` for configuration changes.
    pub fn lock_for_configuration(
        &self,
    ) -> Result<CaptureDeviceConfigurationLock<'_>, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status =
            unsafe { ffi::device::av_capture_device_lock_for_configuration(self.ptr, &mut err) };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(CaptureDeviceConfigurationLock { device: self })
    }
}

#[derive(Debug)]
/// RAII configuration lock for `AVCaptureDeviceConfigurationLock`.
pub struct CaptureDeviceConfigurationLock<'a> {
    device: &'a CaptureDevice,
}

impl CaptureDeviceConfigurationLock<'_> {
    /// Sets the active format on `AVCaptureDeviceConfigurationLock`.
    pub fn set_active_format(&self, format: &CaptureDeviceFormat) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_active_format(self.device.ptr, format.ptr, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the active video min frame duration on `AVCaptureDeviceConfigurationLock`.
    pub fn set_active_video_min_frame_duration(
        &self,
        duration: CMTime,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_active_video_min_frame_duration(
                self.device.ptr,
                duration,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the active video max frame duration on `AVCaptureDeviceConfigurationLock`.
    pub fn set_active_video_max_frame_duration(
        &self,
        duration: CMTime,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_active_video_max_frame_duration(
                self.device.ptr,
                duration,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the exposure mode on `AVCaptureDeviceConfigurationLock`.
    pub fn set_exposure_mode(
        &self,
        mode: impl Into<CaptureExposureMode>,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_exposure_mode(
                self.device.ptr,
                mode.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the focus mode on `AVCaptureDeviceConfigurationLock`.
    pub fn set_focus_mode(&self, mode: impl Into<CaptureFocusMode>) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_focus_mode(
                self.device.ptr,
                mode.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the white balance mode on `AVCaptureDeviceConfigurationLock`.
    pub fn set_white_balance_mode(
        &self,
        mode: impl Into<CaptureWhiteBalanceMode>,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_white_balance_mode(
                self.device.ptr,
                mode.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the torch mode on `AVCaptureDeviceConfigurationLock`.
    pub fn set_torch_mode(&self, mode: impl Into<CaptureTorchMode>) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_torch_mode(
                self.device.ptr,
                mode.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the torch level on `AVCaptureDeviceConfigurationLock`.
    pub fn set_torch_level(&self, level: f32) -> Result<(), AVCaptureError> {
        if !level.is_finite() {
            return Err(AVCaptureError::InvalidArgument(
                "torch level must be finite".to_owned(),
            ));
        }
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_torch_level(self.device.ptr, level, &mut err)
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the active color space on `AVCaptureDeviceConfigurationLock`.
    pub fn set_active_color_space(
        &self,
        color_space: impl Into<CaptureColorSpace>,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_active_color_space(
                self.device.ptr,
                color_space.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the active input source on `AVCaptureDeviceConfigurationLock`.
    pub fn set_active_input_source(
        &self,
        input_source: &CaptureDeviceInputSource,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_active_input_source(
                self.device.ptr,
                input_source.ptr,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the transport controls playback mode on `AVCaptureDeviceConfigurationLock`.
    pub fn set_transport_controls_playback_mode(
        &self,
        mode: impl Into<CaptureDeviceTransportControlsPlaybackMode>,
        speed: f32,
    ) -> Result<(), AVCaptureError> {
        if !speed.is_finite() {
            return Err(AVCaptureError::InvalidArgument(
                "transport controls speed must be finite".to_owned(),
            ));
        }
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_transport_controls_playback_mode(
                self.device.ptr,
                mode.into().as_raw(),
                speed,
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the primary constituent device switching behavior on `AVCaptureDeviceConfigurationLock`.
    pub fn set_primary_constituent_device_switching_behavior(
        &self,
        behavior: impl Into<CapturePrimaryConstituentDeviceSwitchingBehavior>,
        conditions: impl Into<CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions>,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_primary_constituent_device_switching_behavior(
                self.device.ptr,
                behavior.into().as_raw(),
                conditions.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the camera lens smudge detection on `AVCaptureDeviceConfigurationLock`.
    pub fn set_camera_lens_smudge_detection(
        &self,
        enabled: bool,
        detection_interval: Option<CMTime>,
    ) -> Result<(), AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_camera_lens_smudge_detection(
                self.device.ptr,
                enabled,
                detection_interval.is_some(),
                detection_interval.unwrap_or(CMTime {
                    value: 0,
                    timescale: 0,
                    flags: 0,
                    epoch: 0,
                }),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the cinematic video tracking focus at point on `AVCaptureDeviceConfigurationLock`.
    pub fn set_cinematic_video_tracking_focus_at_point(
        &self,
        point: (f64, f64),
        focus_mode: impl Into<CaptureCinematicVideoFocusMode>,
    ) -> Result<(), AVCaptureError> {
        let (x, y) = validate_normalized_point(point)?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_cinematic_video_tracking_focus_at_point(
                self.device.ptr,
                x,
                y,
                focus_mode.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }

    /// Sets the cinematic video fixed focus at point on `AVCaptureDeviceConfigurationLock`.
    pub fn set_cinematic_video_fixed_focus_at_point(
        &self,
        point: (f64, f64),
        focus_mode: impl Into<CaptureCinematicVideoFocusMode>,
    ) -> Result<(), AVCaptureError> {
        let (x, y) = validate_normalized_point(point)?;
        let mut err: *mut c_char = ptr::null_mut();
        let status = unsafe {
            ffi::device::av_capture_device_set_cinematic_video_fixed_focus_at_point(
                self.device.ptr,
                x,
                y,
                focus_mode.into().as_raw(),
                &mut err,
            )
        };
        if status != ffi::status::OK {
            return Err(unsafe { from_swift(status, err) });
        }
        Ok(())
    }
}

impl Drop for CaptureDeviceConfigurationLock<'_> {
    fn drop(&mut self) {
        unsafe { ffi::device::av_capture_device_unlock_for_configuration(self.device.ptr) };
    }
}

fn preset_cstring(preset: &CaptureSessionPreset) -> Result<CString, AVCaptureError> {
    CString::new(preset.as_raw()).map_err(|error| {
        AVCaptureError::InvalidArgument(format!("preset contains NUL byte: {error}"))
    })
}

fn parse_json_string_and_free(json_ptr: *mut c_char) -> String {
    let json = unsafe { std::ffi::CStr::from_ptr(json_ptr) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::core::avc_string_free(json_ptr) };
    serde_json::from_str::<String>(&json).unwrap_or(json)
}

const fn option_bool_from_raw(raw: i32) -> Option<bool> {
    match raw {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    }
}

fn enum_from_class_raw<T>(raw: i32) -> Option<T>
where
    T: From<i32>,
{
    (raw >= 0).then(|| raw.into())
}

fn validate_normalized_point(point: (f64, f64)) -> Result<(f64, f64), AVCaptureError> {
    let (x, y) = point;
    if !x.is_finite() || !y.is_finite() {
        return Err(AVCaptureError::InvalidArgument(
            "point coordinates must be finite".to_owned(),
        ));
    }
    if !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return Err(AVCaptureError::InvalidArgument(
            "point coordinates must be normalized between 0.0 and 1.0".to_owned(),
        ));
    }
    Ok(point)
}
