# avcapture-rs coverage audit (vs MacOSX26.2.sdk)

Scope: top-level symbols from `AVCapture*.h` only (`@interface`, `@protocol`, typedef enums/structs, exported constants, and top-level C functions). Deprecated or `API_UNAVAILABLE(macos)` symbols are EXEMPT. Delegate protocols are counted as VERIFIED when `avcapture-rs` exposes an equivalent Rust callback surface.

SDK_PUBLIC_SYMBOLS: 112
VERIFIED: 112
GAPS: 0
EXEMPT: 65
COVERAGE_PCT: 100.0%

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| AVCaptureAudioDataOutput | interface | AVCaptureAudioDataOutput.h | AudioDataOutput |
| AVCaptureAudioDataOutputSampleBufferDelegate | protocol | AVCaptureAudioDataOutput.h | AudioDataOutput::set_sample_buffer_handler |
| AVCaptureDevice | interface | AVCaptureDevice.h | CaptureDevice / CaptureDeviceInfo / CaptureDeviceDetails |
| AVCaptureDeviceDiscoverySession | interface | AVCaptureDevice.h | CaptureDeviceDiscoverySession |
| AVCaptureDeviceFormat | interface | AVCaptureDevice.h | CaptureDeviceFormat / CaptureDeviceFormatInfo |
| AVCaptureDevicePosition | enum | AVCaptureDevice.h | CaptureDevicePosition |
| AVCaptureDeviceTypeBuiltInWideAngleCamera | constant | AVCaptureDevice.h | CaptureDeviceType::BuiltInWideAngleCamera |
| AVCaptureDeviceTypeContinuityCamera | constant | AVCaptureDevice.h | CaptureDeviceType::ContinuityCamera |
| AVCaptureDeviceTypeDeskViewCamera | constant | AVCaptureDevice.h | CaptureDeviceType::DeskViewCamera |
| AVCaptureDeviceTypeExternal | constant | AVCaptureDevice.h | CaptureDeviceType::External |
| AVCaptureDeviceTypeMicrophone | constant | AVCaptureDevice.h | CaptureDeviceType::Microphone |
| AVCaptureExposureMode | enum | AVCaptureDevice.h | CaptureExposureMode / CaptureDevice::{details, exposure_mode, is_exposure_mode_supported} / CaptureDeviceConfigurationLock::set_exposure_mode |
| AVCaptureFlashMode | enum | AVCaptureDevice.h | CaptureFlashMode |
| AVCaptureTorchMode | enum | AVCaptureDevice.h | CaptureTorchMode |
| AVCaptureFileOutput | interface | AVCaptureFileOutput.h | MovieFileOutput / AudioFileOutput (base recording properties/methods) |
| AVCaptureFileOutputRecordingDelegate | protocol | AVCaptureFileOutput.h | MovieFileOutput::start_recording_with_handler / AudioFileOutput::start_recording_with_handler |
| AVCaptureMovieFileOutput | interface | AVCaptureFileOutput.h | MovieFileOutput / MovieFileOutputInfo |
| AVCaptureDeviceInput | interface | AVCaptureInput.h | DeviceInput / DeviceInputInfo |
| AVCaptureInput | interface | AVCaptureInput.h | CaptureInputRef / CaptureInputInfo |
| AVCaptureInputPort | interface | AVCaptureInput.h | CaptureInputRef::ports -> CaptureInputPortInfo |
| AVCaptureScreenInput | interface | AVCaptureInput.h | ScreenInput / ScreenInputInfo |
| AVCaptureMetadataOutput | interface | AVCaptureMetadataOutput.h | MetadataOutput / MetadataOutputInfo |
| AVCaptureMetadataOutputObjectsDelegate | protocol | AVCaptureMetadataOutput.h | MetadataOutput::set_metadata_objects_handler |
| AVCaptureOutput | interface | AVCaptureOutputBase.h | CaptureOutputRef / CaptureOutputInfo |
| AVCapturePhotoCaptureDelegate | protocol | AVCapturePhotoOutput.h | PhotoOutput::capture_photo |
| AVCapturePhotoOutput | interface | AVCapturePhotoOutput.h | PhotoOutput / PhotoOutputInfo |
| AVCapturePhotoOutputCaptureReadiness | enum | AVCapturePhotoOutput.h | PhotoOutputCaptureReadiness / PhotoOutput::capture_readiness / PhotoOutputInfo::capture_readiness |
| AVCapturePhoto | interface | AVCapturePhotoOutput.h | Photo / PhotoInfo / PhotoCaptureEvent::photo |
| AVCapturePhotoQualityPrioritization | enum | AVCapturePhotoOutput.h | PhotoQualityPrioritization / PhotoSettings / PhotoOutput |
| AVCapturePhotoSettings | interface | AVCapturePhotoOutput.h | PhotoSettings / PhotoSettingsInfo / PhotoOutput::capture_photo_with_settings |
| AVCaptureConnection | interface | AVCaptureSession.h | CaptureConnection / CaptureConnectionInfo |
| AVCaptureSession | interface | AVCaptureSession.h | CaptureSession / CaptureSessionInfo |
| AVCaptureSessionPreset1280x720 | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::Hd1280x720 |
| AVCaptureSessionPreset1920x1080 | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::FullHd1920x1080 |
| AVCaptureSessionPreset320x240 | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::Qvga320x240 |
| AVCaptureSessionPreset352x288 | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::Cif352x288 |
| AVCaptureSessionPreset3840x2160 | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::Uhd3840x2160 |
| AVCaptureSessionPreset640x480 | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::Vga640x480 |
| AVCaptureSessionPreset960x540 | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::Qhd960x540 |
| AVCaptureSessionPresetHigh | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::High |
| AVCaptureSessionPresetLow | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::Low |
| AVCaptureSessionPresetMedium | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::Medium |
| AVCaptureSessionPresetPhoto | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::Photo |
| AVCaptureSessionPresetiFrame1280x720 | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::IFrame1280x720 |
| AVCaptureSessionPresetiFrame960x540 | constant | AVCaptureSessionPreset.h | CaptureSessionPreset::IFrame960x540 |
| AVCaptureVideoDataOutput | interface | AVCaptureVideoDataOutput.h | VideoDataOutput / VideoDataOutputInfo |
| AVCaptureVideoDataOutputSampleBufferDelegate | protocol | AVCaptureVideoDataOutput.h | VideoDataOutput::set_sample_buffer_handler |
| AVCaptureVideoPreviewLayer | interface | AVCaptureVideoPreviewLayer.h | VideoPreviewLayer / VideoPreviewLayerInfo |

## 🟢 VERIFIED (continued)
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| AVCaptureAudioPreviewOutput | interface | AVCaptureAudioPreviewOutput.h | AudioPreviewOutput / AudioPreviewOutputInfo |
| AVCaptureControl | interface | AVCaptureControl.h | CaptureControl / CaptureControlInfo |
| AVCaptureDeskViewApplication | interface | AVCaptureDeskViewApplication.h | DeskViewApplication / DeskViewApplicationInfo |
| AVCaptureDeskViewApplicationLaunchConfiguration | interface | AVCaptureDeskViewApplication.h | DeskViewApplicationLaunchConfiguration / DeskViewApplicationLaunchConfigurationInfo |
| AVCaptureAutoFocusSystem | enum | AVCaptureDevice.h | CaptureAutoFocusSystem / CaptureDevice::{details, auto_focus_system} |
| AVCaptureCameraLensSmudgeDetectionStatus | enum | AVCaptureDevice.h | CaptureCameraLensSmudgeDetectionStatus / CaptureDevice::{details, camera_lens_smudge_detection_status} |
| AVCaptureCenterStageControlMode | enum | AVCaptureDevice.h | CaptureCenterStageControlMode / CaptureDevice::center_stage_control_mode |
| AVCaptureCinematicVideoFocusMode | enum | AVCaptureDevice.h | CaptureCinematicVideoFocusMode (typed enum export) |
| AVCaptureColorSpace | enum | AVCaptureDevice.h | CaptureColorSpace / CaptureDevice::{details, active_color_space, supported_color_spaces} |
| AVCaptureDeviceInputSource | interface | AVCaptureDevice.h | CaptureDeviceInputSource / CaptureDeviceInputSourceInfo / CaptureDevice::{input_sources, active_input_source} |
| AVCaptureDeviceRotationCoordinator | interface | AVCaptureDevice.h | CaptureDeviceRotationCoordinator / CaptureDeviceRotationCoordinatorInfo / CaptureDevice::rotation_coordinator |
| AVCaptureDeviceTransportControlsPlaybackMode | enum | AVCaptureDevice.h | CaptureDeviceTransportControlsPlaybackMode / CaptureDevice::{details, transport_controls_playback_mode} |
| AVCaptureDeviceWasConnectedNotification | constant | AVCaptureDevice.h | CaptureDevice::WAS_CONNECTED_NOTIFICATION |
| AVCaptureDeviceWasDisconnectedNotification | constant | AVCaptureDevice.h | CaptureDevice::WAS_DISCONNECTED_NOTIFICATION |
| AVCaptureFocusMode | enum | AVCaptureDevice.h | CaptureFocusMode / CaptureDevice::{details, focus_mode, is_focus_mode_supported} / CaptureDeviceConfigurationLock::set_focus_mode |
| AVCaptureMaxAvailableTorchLevel | constant | AVCaptureDevice.h | CaptureDevice::max_available_torch_level |
| AVCaptureMicrophoneMode | enum | AVCaptureDevice.h | CaptureMicrophoneMode / CaptureDevice::{preferred_microphone_mode, active_microphone_mode} |
| AVCapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions | enum | AVCaptureDevice.h | CapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions / CaptureDevice::{details, primary_constituent_device_restricted_switching_behavior_conditions, active_primary_constituent_device_restricted_switching_behavior_conditions} |
| AVCapturePrimaryConstituentDeviceSwitchingBehavior | enum | AVCaptureDevice.h | CapturePrimaryConstituentDeviceSwitchingBehavior / CaptureDevice::{details, primary_constituent_device_switching_behavior, active_primary_constituent_device_switching_behavior} |
| AVCaptureSceneMonitoringStatusNotEnoughLight | constant | AVCaptureDevice.h | CaptureSceneMonitoringStatus::NotEnoughLight / CaptureDevice::scene_monitoring_status_not_enough_light |
| AVCaptureSystemUserInterface | enum | AVCaptureDevice.h | CaptureSystemUserInterface / CaptureDevice::show_system_user_interface |
| AVCaptureWhiteBalanceMode | enum | AVCaptureDevice.h | CaptureWhiteBalanceMode / CaptureDevice::{details, white_balance_mode, is_white_balance_mode_supported} / CaptureDeviceConfigurationLock::set_white_balance_mode |
| AVCaptureExternalDisplayConfiguration | interface | AVCaptureExternalDisplayConfigurator.h | ExternalDisplayConfiguration / ExternalDisplayConfigurationInfo |
| AVCaptureExternalDisplayConfigurator | interface | AVCaptureExternalDisplayConfigurator.h | ExternalDisplayConfigurator / ExternalDisplayConfiguratorInfo / VideoPreviewLayer::external_display_configurator |
| AVCaptureAudioFileOutput | interface | AVCaptureFileOutput.h | AudioFileOutput / AudioFileOutputInfo |
| AVCaptureFileOutputDelegate | protocol | AVCaptureFileOutput.h | MovieFileOutput::set_sample_buffer_boundary_handler / AudioFileOutput::set_sample_buffer_boundary_handler |
| AVCaptureIndexPicker | interface | AVCaptureIndexPicker.h | CaptureIndexPicker / CaptureIndexPickerInfo |
| AVCaptureInputPortFormatDescriptionDidChangeNotification | constant | AVCaptureInput.h | DeviceInput::INPUT_PORT_FORMAT_DESCRIPTION_DID_CHANGE_NOTIFICATION |
| AVCaptureMultichannelAudioMode | enum | AVCaptureInput.h | CaptureMultichannelAudioMode / DeviceInput::{info, multichannel_audio_mode, is_multichannel_audio_mode_supported, set_multichannel_audio_mode} |
| AVCaptureOutputDataDroppedReason | enum | AVCaptureOutputBase.h | AVCaptureOutputDataDroppedReason / CaptureOutputDataDroppedReason / {AudioDataOutput, VideoDataOutput}::last_dropped_sample_reason |
| AVCapturePhotoOutputReadinessCoordinator | interface | AVCapturePhotoOutput.h | PhotoOutputReadinessCoordinator / PhotoOutput::readiness_coordinator |
| AVCapturePhotoOutputReadinessCoordinatorDelegate | protocol | AVCapturePhotoOutput.h | PhotoOutputReadinessCoordinator::set_capture_readiness_handler |
| AVCaptureResolvedPhotoSettings | interface | AVCapturePhotoOutput.h | ResolvedPhotoSettings / ResolvedPhotoSettingsInfo / Photo::resolved_settings |
| AVCaptureReactionEffectState | interface | AVCaptureReactions.h | CaptureReactionEffectState / CaptureDevice::reaction_effects_in_progress |
| AVCaptureReactionSystemImageNameForType | function | AVCaptureReactions.h | CaptureReactionType::system_image_name |
| AVCaptureReactionTypeBalloons | constant | AVCaptureReactions.h | CaptureReactionType::Balloons / CaptureDevice::reaction_type_balloons |
| AVCaptureReactionTypeConfetti | constant | AVCaptureReactions.h | CaptureReactionType::Confetti / CaptureDevice::reaction_type_confetti |
| AVCaptureReactionTypeFireworks | constant | AVCaptureReactions.h | CaptureReactionType::Fireworks / CaptureDevice::reaction_type_fireworks |
| AVCaptureReactionTypeHeart | constant | AVCaptureReactions.h | CaptureReactionType::Heart / CaptureDevice::reaction_type_heart |
| AVCaptureReactionTypeLasers | constant | AVCaptureReactions.h | CaptureReactionType::Lasers / CaptureDevice::reaction_type_lasers |
| AVCaptureReactionTypeRain | constant | AVCaptureReactions.h | CaptureReactionType::Rain / CaptureDevice::reaction_type_rain |
| AVCaptureReactionTypeThumbsDown | constant | AVCaptureReactions.h | CaptureReactionType::ThumbsDown / CaptureDevice::reaction_type_thumbs_down |
| AVCaptureReactionTypeThumbsUp | constant | AVCaptureReactions.h | CaptureReactionType::ThumbsUp / CaptureDevice::reaction_type_thumbs_up |
| AVCaptureAudioChannel | interface | AVCaptureSession.h | CaptureAudioChannel / CaptureAudioChannelInfo / CaptureConnection::{audio_channels, audio_channels_info} |
| AVCaptureSessionControlsDelegate | protocol | AVCaptureSession.h | CaptureSession::set_controls_delegate_handler |
| AVCaptureSessionDeferredStartDelegate | protocol | AVCaptureSession.h | CaptureSession::set_deferred_start_delegate_handler |
| AVCaptureSessionDidStartRunningNotification | constant | AVCaptureSession.h | CaptureSession::DID_START_RUNNING_NOTIFICATION |
| AVCaptureSessionDidStopRunningNotification | constant | AVCaptureSession.h | CaptureSession::DID_STOP_RUNNING_NOTIFICATION |
| AVCaptureSessionErrorKey | constant | AVCaptureSession.h | CaptureSession::ERROR_KEY |
| AVCaptureSessionInterruptionEndedNotification | constant | AVCaptureSession.h | CaptureSession::INTERRUPTION_ENDED_NOTIFICATION |
| AVCaptureSessionRuntimeErrorNotification | constant | AVCaptureSession.h | CaptureSession::RUNTIME_ERROR_NOTIFICATION |
| AVCaptureSessionWasInterruptedNotification | constant | AVCaptureSession.h | CaptureSession::WAS_INTERRUPTED_NOTIFICATION |
| AVCaptureSlider | interface | AVCaptureSlider.h | CaptureSlider / CaptureSliderInfo |
| AVCaptureSystemExposureBiasSlider | interface | AVCaptureSystemExposureBiasSlider.h | CaptureSystemExposureBiasSlider / CaptureSession::system_exposure_bias_slider |
| AVCaptureSystemZoomSlider | interface | AVCaptureSystemZoomSlider.h | CaptureSystemZoomSlider / CaptureSession::system_zoom_slider |
| AVCaptureTimecode | struct | AVCaptureTimecodeGenerator.h | CaptureTimecode |
| AVCaptureTimecodeAdvancedByFrames | function | AVCaptureTimecodeGenerator.h | CaptureTimecode::advanced_by_frames |
| AVCaptureTimecodeCreateMetadataSampleBufferAssociatedWithPresentationTimeStamp | function | AVCaptureTimecodeGenerator.h | CaptureTimecode::create_metadata_sample_buffer_associated_with_presentation_time_stamp |
| AVCaptureTimecodeCreateMetadataSampleBufferForDuration | function | AVCaptureTimecodeGenerator.h | CaptureTimecode::create_metadata_sample_buffer_for_duration |
| AVCaptureTimecodeGenerator | interface | AVCaptureTimecodeGenerator.h | CaptureTimecodeGenerator / VideoDataOutput::timecode_generator |
| AVCaptureTimecodeGeneratorDelegate | protocol | AVCaptureTimecodeGenerator.h | CaptureTimecodeGenerator::set_delegate_handler |
| AVCaptureTimecodeGeneratorSynchronizationStatus | enum | AVCaptureTimecodeGenerator.h | CaptureTimecodeGeneratorSynchronizationStatus |
| AVCaptureTimecodeSource | interface | AVCaptureTimecodeGenerator.h | CaptureTimecodeSource / CaptureTimecodeSourceInfo / VideoDataOutput::{frame_count_timecode_source, real_time_clock_timecode_source} |
| AVCaptureTimecodeSourceType | enum | AVCaptureTimecodeGenerator.h | CaptureTimecodeSourceType |

## 🔴 GAPS
_None. All audited macOS-available top-level `AVCapture*` symbols are now wrapped._

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| AVCaptureDataOutputSynchronizer | interface | AVCaptureDataOutputSynchronizer.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDataOutputSynchronizerDelegate | protocol | AVCaptureDataOutputSynchronizer.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedData | interface | AVCaptureDataOutputSynchronizer.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedDataCollection | interface | AVCaptureDataOutputSynchronizer.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedDepthData | interface | AVCaptureDataOutputSynchronizer.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedMetadataObjectData | interface | AVCaptureDataOutputSynchronizer.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedSampleBufferData | interface | AVCaptureDataOutputSynchronizer.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDepthDataOutput | interface | AVCaptureDepthDataOutput.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDepthDataOutputDelegate | protocol | AVCaptureDepthDataOutput.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureAspectRatio16x9 | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0)); API_UNAVAILABLE(macos, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureAspectRatio1x1 | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0)); API_UNAVAILABLE(macos, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureAspectRatio3x4 | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0)); API_UNAVAILABLE(macos, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureAspectRatio4x3 | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0)); API_UNAVAILABLE(macos, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureAspectRatio9x16 | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0)); API_UNAVAILABLE(macos, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureAutoFocusRangeRestriction | enum | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(7.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceSubjectAreaDidChangeNotification | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(5.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceTypeBuiltInDualCamera | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(10.2), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceTypeBuiltInDualWideCamera | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(13.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceTypeBuiltInDuoCamera | constant | AVCaptureDevice.h | Unavailable on macOS | API_DEPRECATED("Use AVCaptureDeviceTypeBuiltInDualCamera instead.", ios(10.0, 10.2)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(tvos, watchos) |
| AVCaptureDeviceTypeBuiltInLiDARDepthCamera | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(15.4), macCatalyst(15.4), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceTypeBuiltInMicrophone | constant | AVCaptureDevice.h | Deprecated on macOS | API_DEPRECATED_WITH_REPLACEMENT("AVCaptureDeviceTypeMicrophone", macos(10.15, 14.0), ios(10.0, 17.0), macCatalyst(14.0, 17.0)); API_UNAVAILABLE(tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceTypeBuiltInTelephotoCamera | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(10.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceTypeBuiltInTripleCamera | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(13.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceTypeBuiltInTrueDepthCamera | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceTypeBuiltInUltraWideCamera | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(13.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeviceTypeExternalUnknown | constant | AVCaptureDevice.h | Deprecated on macOS | API_DEPRECATED_WITH_REPLACEMENT("AVCaptureDeviceTypeExternal", macos(10.15, 14.0)); API_UNAVAILABLE(ios, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureExposureDurationCurrent | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureExposureTargetBiasCurrent | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureFraming | interface | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0)); API_UNAVAILABLE(macos, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureISOCurrent | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureLensPositionCurrent | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureSmartFramingMonitor | interface | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0)); API_UNAVAILABLE(macos, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureVideoStabilizationMode | enum | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureWhiteBalanceChromaticityValues | struct | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureWhiteBalanceGains | struct | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureWhiteBalanceGainsCurrent | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureWhiteBalanceTemperatureAndTintValues | struct | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureWhiteBalanceTemperatureAndTintValuesCloudy | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0), macCatalyst(26.0), tvos(26.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureWhiteBalanceTemperatureAndTintValuesDaylight | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0), macCatalyst(26.0), tvos(26.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureWhiteBalanceTemperatureAndTintValuesFluorescent | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0), macCatalyst(26.0), tvos(26.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureWhiteBalanceTemperatureAndTintValuesShadow | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0), macCatalyst(26.0), tvos(26.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureWhiteBalanceTemperatureAndTintValuesTungsten | constant | AVCaptureDevice.h | Unavailable on macOS | API_AVAILABLE(ios(26.0), macCatalyst(26.0), tvos(26.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureMetadataInput | interface | AVCaptureInput.h | Unavailable on macOS | API_AVAILABLE(ios(9.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureDeferredPhotoProxy | interface | AVCapturePhotoOutput.h | Unavailable on macOS | API_AVAILABLE(ios(17.0)); API_UNAVAILABLE(macos, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureLensStabilizationStatus | enum | AVCapturePhotoOutput.h | Unavailable on macOS | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCapturePhotoBracketSettings | interface | AVCapturePhotoOutput.h | Unavailable on macOS | API_AVAILABLE(ios(10.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCapturePhotoFileDataRepresentationCustomizer | protocol | AVCapturePhotoOutput.h | Unavailable on macOS | API_AVAILABLE(ios(12.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureMultiCamSession | interface | AVCaptureSession.h | Unavailable on macOS | API_AVAILABLE(ios(13.0), macCatalyst(14.0), tvos(17.0), visionos(2.1)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureSessionInterruptionReason | enum | AVCaptureSession.h | Unavailable on macOS | API_AVAILABLE(ios(9.0), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureSessionInterruptionReasonKey | constant | AVCaptureSession.h | Unavailable on macOS | API_AVAILABLE(ios(9.0), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureSessionInterruptionSystemPressureStateKey | constant | AVCaptureSession.h | Unavailable on macOS | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureVideoOrientation | enum | AVCaptureSession.h | Deprecated on macOS | API_DEPRECATED("Use AVCaptureDeviceRotationCoordinator instead", macos(10.7, 14.0), ios(4.0, 17.0), macCatalyst(14.0, 17.0)); API_UNAVAILABLE(tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureSessionPresetInputPriority | constant | AVCaptureSessionPreset.h | Unavailable on macOS | API_AVAILABLE(ios(7.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureSpatialAudioMetadataSampleGenerator | interface | AVCaptureSpatialAudioMetadataSampleGenerator.h | Unavailable on macOS | API_AVAILABLE(ios(26.0)); API_UNAVAILABLE(macos, macCatalyst, tvos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureAutoExposureBracketedStillImageSettings | interface | AVCaptureStillImageOutput.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureBracketedStillImageSettings | interface | AVCaptureStillImageOutput.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureManualExposureBracketedStillImageSettings | interface | AVCaptureStillImageOutput.h | Unavailable on macOS | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)); API_UNAVAILABLE(macos, visionos); API_UNAVAILABLE(watchos) |
| AVCaptureStillImageOutput | interface | AVCaptureStillImageOutput.h | Deprecated on macOS | API_DEPRECATED("Use AVCapturePhotoOutput instead.", macos(10.7, 10.15), ios(4.0, 10.0), visionos(1.0, 1.0)); API_UNAVAILABLE(tvos, watchos) |
| AVCaptureSystemPressureFactors | enum | AVCaptureSystemPressure.h | Unavailable on macOS | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureSystemPressureLevelCritical | constant | AVCaptureSystemPressure.h | Unavailable on macOS | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureSystemPressureLevelFair | constant | AVCaptureSystemPressure.h | Unavailable on macOS | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureSystemPressureLevelNominal | constant | AVCaptureSystemPressure.h | Unavailable on macOS | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureSystemPressureLevelSerious | constant | AVCaptureSystemPressure.h | Unavailable on macOS | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureSystemPressureLevelShutdown | constant | AVCaptureSystemPressure.h | Unavailable on macOS | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
| AVCaptureSystemPressureState | interface | AVCaptureSystemPressure.h | Unavailable on macOS | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0), visionos(1.0)); API_UNAVAILABLE(macos); API_UNAVAILABLE(watchos) |
