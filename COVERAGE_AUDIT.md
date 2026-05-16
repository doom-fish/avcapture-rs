# avcapture-rs coverage audit (vs MacOSX26.2.sdk)

Scope: top-level symbols from `AVCapture*.h` only (`@interface`, `@protocol`, typedef enums/structs, exported constants, and top-level C functions). Deprecated or `API_UNAVAILABLE(macos)` symbols are EXEMPT. Delegate protocols are counted as VERIFIED when `avcapture-rs` exposes an equivalent Rust callback surface.

SDK_PUBLIC_SYMBOLS: 112
VERIFIED: 43
GAPS: 69
EXEMPT: 65
COVERAGE_PCT: 38.4%

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
| AVCaptureFlashMode | enum | AVCaptureDevice.h | CaptureFlashMode |
| AVCaptureTorchMode | enum | AVCaptureDevice.h | CaptureTorchMode |
| AVCaptureFileOutput | interface | AVCaptureFileOutput.h | MovieFileOutput (base recording properties/methods) |
| AVCaptureFileOutputRecordingDelegate | protocol | AVCaptureFileOutput.h | MovieFileOutput::start_recording_with_handler |
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
| AVCapturePhotoOutputCaptureReadiness | enum | AVCapturePhotoOutput.h | PhotoOutput::capture_readiness / PhotoOutputInfo::capture_readiness (raw i32) |
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

## 🔴 GAPS
| Symbol | Kind | Header | Notes |
| --- | --- | --- | --- |
| AVCaptureAudioPreviewOutput | interface | AVCaptureAudioPreviewOutput.h | No audio-preview output wrapper. |
| AVCaptureControl | interface | AVCaptureControl.h | No capture-control UI wrappers. |
| AVCaptureDeskViewApplication | interface | AVCaptureDeskViewApplication.h | No Desk View application/configuration wrapper. |
| AVCaptureDeskViewApplicationLaunchConfiguration | interface | AVCaptureDeskViewApplication.h | No Desk View application/configuration wrapper. |
| AVCaptureAutoFocusSystem | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureCameraLensSmudgeDetectionStatus | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureCenterStageControlMode | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureCinematicVideoFocusMode | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureColorSpace | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureDeviceInputSource | interface | AVCaptureDevice.h | No public wrapper for device input sources. |
| AVCaptureDeviceRotationCoordinator | interface | AVCaptureDevice.h | Connection rotation angle is exposed, but not the coordinator type. |
| AVCaptureDeviceTransportControlsPlaybackMode | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureDeviceWasConnectedNotification | constant | AVCaptureDevice.h | No notification/observer surface. |
| AVCaptureDeviceWasDisconnectedNotification | constant | AVCaptureDevice.h | No notification/observer surface. |
| AVCaptureExposureMode | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureFocusMode | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureMaxAvailableTorchLevel | constant | AVCaptureDevice.h | Torch mode is wrapped, but level-setting APIs/constants are not. |
| AVCaptureMicrophoneMode | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCapturePrimaryConstituentDeviceRestrictedSwitchingBehaviorConditions | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCapturePrimaryConstituentDeviceSwitchingBehavior | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureSceneMonitoringStatusNotEnoughLight | constant | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureSystemUserInterface | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureWhiteBalanceMode | enum | AVCaptureDevice.h | Device wrapper omits this control/status surface. |
| AVCaptureExternalDisplayConfiguration | interface | AVCaptureExternalDisplayConfigurator.h | No external-display configuration wrapper. |
| AVCaptureExternalDisplayConfigurator | interface | AVCaptureExternalDisplayConfigurator.h | No external-display configuration wrapper. |
| AVCaptureAudioFileOutput | interface | AVCaptureFileOutput.h | Movie file output is wrapped; audio file output is not. |
| AVCaptureFileOutputDelegate | protocol | AVCaptureFileOutput.h | Recording delegate callback is wrapped; base file-output delegate is not. |
| AVCaptureIndexPicker | interface | AVCaptureIndexPicker.h | No capture-control UI wrappers. |
| AVCaptureInputPortFormatDescriptionDidChangeNotification | constant | AVCaptureInput.h | No notification/observer surface. |
| AVCaptureMultichannelAudioMode | enum | AVCaptureInput.h | DeviceInput wrapper does not expose multichannel audio mode. |
| AVCaptureOutputDataDroppedReason | enum | AVCaptureOutputBase.h | No dropped-sample reason surface. |
| AVCapturePhoto | interface | AVCapturePhotoOutput.h | Photo output is wrapped, but this specific photo/settings/readiness type is not. |
| AVCapturePhotoOutputReadinessCoordinator | interface | AVCapturePhotoOutput.h | Photo output is wrapped, but this specific photo/settings/readiness type is not. |
| AVCapturePhotoOutputReadinessCoordinatorDelegate | protocol | AVCapturePhotoOutput.h | Photo output is wrapped, but this specific photo/settings/readiness type is not. |
| AVCapturePhotoQualityPrioritization | enum | AVCapturePhotoOutput.h | Photo output is wrapped, but this specific photo/settings/readiness type is not. |
| AVCapturePhotoSettings | interface | AVCapturePhotoOutput.h | Photo output is wrapped, but this specific photo/settings/readiness type is not. |
| AVCaptureResolvedPhotoSettings | interface | AVCapturePhotoOutput.h | Photo output is wrapped, but this specific photo/settings/readiness type is not. |
| AVCaptureReactionEffectState | interface | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureReactionSystemImageNameForType | function | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureReactionTypeBalloons | constant | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureReactionTypeConfetti | constant | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureReactionTypeFireworks | constant | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureReactionTypeHeart | constant | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureReactionTypeLasers | constant | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureReactionTypeRain | constant | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureReactionTypeThumbsDown | constant | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureReactionTypeThumbsUp | constant | AVCaptureReactions.h | Reaction types/effects are not exposed. |
| AVCaptureAudioChannel | interface | AVCaptureSession.h | Connection wrapper omits audio-channel inspection. |
| AVCaptureSessionControlsDelegate | protocol | AVCaptureSession.h | No session-controls/deferred-start delegate wrappers. |
| AVCaptureSessionDeferredStartDelegate | protocol | AVCaptureSession.h | No session-controls/deferred-start delegate wrappers. |
| AVCaptureSessionDidStartRunningNotification | constant | AVCaptureSession.h | No notification/observer surface. |
| AVCaptureSessionDidStopRunningNotification | constant | AVCaptureSession.h | No notification/observer surface. |
| AVCaptureSessionErrorKey | constant | AVCaptureSession.h | No notification/observer surface. |
| AVCaptureSessionInterruptionEndedNotification | constant | AVCaptureSession.h | No notification/observer surface. |
| AVCaptureSessionRuntimeErrorNotification | constant | AVCaptureSession.h | No notification/observer surface. |
| AVCaptureSessionWasInterruptedNotification | constant | AVCaptureSession.h | No notification/observer surface. |
| AVCaptureSlider | interface | AVCaptureSlider.h | No capture-control UI wrappers. |
| AVCaptureSystemExposureBiasSlider | interface | AVCaptureSystemExposureBiasSlider.h | No capture-control UI wrappers. |
| AVCaptureSystemZoomSlider | interface | AVCaptureSystemZoomSlider.h | No capture-control UI wrappers. |
| AVCaptureTimecode | struct | AVCaptureTimecodeGenerator.h | Timecode generation APIs are not wrapped. |
| AVCaptureTimecodeAdvancedByFrames | function | AVCaptureTimecodeGenerator.h | Timecode generation APIs are not wrapped. |
| AVCaptureTimecodeCreateMetadataSampleBufferAssociatedWithPresentationTimeStamp | function | AVCaptureTimecodeGenerator.h | Timecode generation APIs are not wrapped. |
| AVCaptureTimecodeCreateMetadataSampleBufferForDuration | function | AVCaptureTimecodeGenerator.h | Timecode generation APIs are not wrapped. |
| AVCaptureTimecodeGenerator | interface | AVCaptureTimecodeGenerator.h | Timecode generation APIs are not wrapped. |
| AVCaptureTimecodeGeneratorDelegate | protocol | AVCaptureTimecodeGenerator.h | Timecode generation APIs are not wrapped. |
| AVCaptureTimecodeGeneratorSynchronizationStatus | enum | AVCaptureTimecodeGenerator.h | Timecode generation APIs are not wrapped. |
| AVCaptureTimecodeSource | interface | AVCaptureTimecodeGenerator.h | Timecode generation APIs are not wrapped. |
| AVCaptureTimecodeSourceType | enum | AVCaptureTimecodeGenerator.h | Timecode generation APIs are not wrapped. |
| AVCaptureVideoPreviewLayer | interface | AVCaptureVideoPreviewLayer.h | No preview-layer wrapper. |

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
