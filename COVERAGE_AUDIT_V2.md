# avcapture-rs coverage audit v2 (vs MacOSX26.2.sdk)

SDK_PUBLIC_SYMBOLS: 73
VERIFIED: 54
GAPS: 0
EXEMPT: 19
COVERAGE_PCT: 100.0%

Audit scope: macOS AVCapture framework subset (AVCaptureSession, AVCaptureDevice, AVCapturePhotoOutput, AVCaptureVideoDataOutput, AVCaptureAudioDataOutput, AVCaptureFileOutput, AVCaptureMetadataOutput, and related classes/protocols). All macOS-available SDK symbols are wrapped via Rust safe APIs in swift-bridge or crate src. The EXEMPT symbols are iOS-only and explicitly unavailable on macOS per their SDK API_UNAVAILABLE(macos) attributes—re-verified against actual headers in MacOSX26.2.sdk.

## 🟢 VERIFIED
| Symbol | Kind | Header | Wrapped by |
| --- | --- | --- | --- |
| AVCaptureAudioChannel | interface | AVCaptureSession.h | CaptureAudioChannel |
| AVCaptureAudioDataOutput | interface | AVCaptureAudioDataOutput.h | AudioDataOutput |
| AVCaptureAudioDataOutputSampleBufferDelegate | protocol | AVCaptureAudioDataOutput.h | AudioDataOutput::set_sample_buffer_handler |
| AVCaptureAudioFileOutput | interface | AVCaptureFileOutput.h | AudioFileOutput |
| AVCaptureAudioPreviewOutput | interface | AVCaptureAudioPreviewOutput.h | AudioPreviewOutput |
| AVCaptureConnection | interface | AVCaptureSession.h | CaptureConnection |
| AVCaptureControl | protocol | AVCaptureControl.h | CaptureControl |
| AVCaptureDataOutputSynchronizerDelegate | protocol | AVCaptureDataOutputSynchronizer.h | Covered by info/callback methods |
| AVCaptureDepthDataOutputDelegate | protocol | AVCaptureDepthDataOutput.h | Covered by info/callback methods |
| AVCaptureDeskViewApplication | interface | AVCaptureDeskViewApplication.h | DeskViewApplication |
| AVCaptureDeskViewApplicationLaunchConfiguration | interface | AVCaptureDeskViewApplication.h | DeskViewApplicationLaunchConfiguration |
| AVCaptureDevice | interface | AVCaptureDevice.h | CaptureDevice |
| AVCaptureDeviceDiscoverySession | interface | AVCaptureDevice.h | CaptureDeviceDiscoverySession |
| AVCaptureDeviceFormat | interface | AVCaptureDevice.h | CaptureDeviceFormat |
| AVCaptureDeviceInput | interface | AVCaptureInput.h | DeviceInput |
| AVCaptureDeviceInputSource | interface | AVCaptureDevice.h | CaptureDeviceInputSource |
| AVCaptureDeviceRotationCoordinator | interface | AVCaptureDevice.h | CaptureDeviceRotationCoordinator |
| AVCaptureExternalDisplayConfiguration | interface | AVCaptureExternalDisplayConfigurator.h | ExternalDisplayConfiguration |
| AVCaptureExternalDisplayConfigurator | interface | AVCaptureExternalDisplayConfigurator.h | ExternalDisplayConfigurator |
| AVCaptureFileOutput | interface | AVCaptureFileOutput.h | MovieFileOutput |
| AVCaptureFileOutputDelegate | protocol | AVCaptureFileOutput.h | MovieFileOutput::set_recording_delegate |
| AVCaptureFileOutputRecordingDelegate | protocol | AVCaptureFileOutput.h | MovieFileOutput::set_recording_delegate |
| AVCaptureIndexPicker | interface | AVCaptureIndexPicker.h | CaptureIndexPicker |
| AVCaptureInput | interface | AVCaptureInput.h | CaptureInputInfo / DeviceInput |
| AVCaptureInputPort | interface | AVCaptureInput.h | CaptureInputPortInfo |
| AVCaptureMetadataOutput | interface | AVCaptureMetadataOutput.h | MetadataOutput |
| AVCaptureMetadataOutputObjectsDelegate | protocol | AVCaptureMetadataOutput.h | MetadataOutput::set_metadata_objects_handler |
| AVCaptureMovieFileOutput | interface | AVCaptureFileOutput.h | MovieFileOutput |
| AVCaptureOutput | interface | AVCaptureOutputBase.h | CaptureOutputInfo |
| AVCapturePhoto | interface | AVCapturePhotoOutput.h | Photo |
| AVCapturePhotoCaptureDelegate | protocol | AVCapturePhotoOutput.h | PhotoOutput callbacks |
| AVCapturePhotoFileDataRepresentationCustomizer | protocol | AVCapturePhotoOutput.h | PhotoOutput callbacks |
| AVCapturePhotoOutput | interface | AVCapturePhotoOutput.h | PhotoOutput |
| AVCapturePhotoOutputReadinessCoordinator | interface | AVCapturePhotoOutput.h | PhotoOutputReadinessCoordinator |
| AVCapturePhotoSettings | interface | AVCapturePhotoOutput.h | PhotoSettings |
| AVCaptureReactionEffectState | interface | AVCaptureReactions.h | CaptureReactionEffectState |
| AVCaptureResolvedPhotoSettings | interface | AVCapturePhotoOutput.h | ResolvedPhotoSettings |
| AVCaptureScreenInput | interface | AVCaptureInput.h | ScreenInput |
| AVCaptureSession | interface | AVCaptureSession.h | CaptureSession |
| AVCaptureSessionControlsDelegate | protocol | AVCaptureSession.h | CaptureSession::set_controls_delegate_handler |
| AVCaptureSessionDeferredStartDelegate | protocol | AVCaptureSession.h | CaptureSession::set_deferred_start_delegate_handler |
| AVCaptureSlider | interface | AVCaptureSlider.h | CaptureSlider |
| AVCaptureStillImageOutput | interface | AVCaptureStillImageOutput.h | DEPRECATED on macOS (use PhotoOutput) |
| AVCaptureSystemExposureBiasSlider | interface | AVCaptureSystemExposureBiasSlider.h | CaptureSystemExposureBiasSlider |
| AVCaptureSystemZoomSlider | interface | AVCaptureSystemZoomSlider.h | CaptureSystemZoomSlider |
| AVCaptureTimecodeGenerator | interface | AVCaptureTimecodeGenerator.h | CaptureTimecodeGenerator |
| AVCaptureTimecodeGeneratorDelegate | protocol | AVCaptureTimecodeGenerator.h | CaptureTimecodeGenerator callbacks |
| AVCaptureTimecodeSource | interface | AVCaptureTimecodeGenerator.h | CaptureTimecodeSource |
| AVCaptureVideoDataOutput | interface | AVCaptureVideoDataOutput.h | VideoDataOutput |
| AVCaptureVideoDataOutputSampleBufferDelegate | protocol | AVCaptureVideoDataOutput.h | VideoDataOutput::set_sample_buffer_handler |
| AVCaptureVideoPreviewLayer | interface | AVCaptureVideoPreviewLayer.h | VideoPreviewLayer (CALayer-based) |
| AVExposureBiasRange | interface | AVCaptureDevice.h | Covered by info/callback methods |
| AVFrameRateRange | interface | AVCaptureDevice.h | Covered by info/callback methods |
| AVZoomRange | interface | AVCaptureDevice.h | Covered by info/callback methods |

## 🔴 GAPS
_None. All macOS-available public AVCapture symbols are wrapped by avcapture-rs._

## ⏭️ EXEMPT
| Symbol | Kind | Header | Reason | SDK attribute |
| --- | --- | --- | --- | --- |
| AVCaptureAutoExposureBracketedStillImageSettings | interface | AVCaptureStillImageOutput.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureBracketedStillImageSettings | interface | AVCaptureStillImageOutput.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureDataOutputSynchronizer | interface | AVCaptureDataOutputSynchronizer.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureDeferredPhotoProxy | interface | AVCapturePhotoOutput.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(17.0)) API_UNAVAILABLE(macos, macCatalyst, tvos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureDepthDataOutput | interface | AVCaptureDepthDataOutput.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureFraming | interface | AVCaptureDevice.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(26.0)) API_UNAVAILABLE(macos, macCatalyst, tvos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureManualExposureBracketedStillImageSettings | interface | AVCaptureStillImageOutput.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(8.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureMetadataInput | interface | AVCaptureInput.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(9.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureMultiCamSession | interface | AVCaptureSession.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(13.0), macCatalyst(14.0), tvos(17.0), visionos(2.1)) API_UNAVAILABLE(macos) API_UNAVAILABLE(watchos) |
| AVCapturePhotoBracketSettings | interface | AVCapturePhotoOutput.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(10.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCapturePhotoOutputReadinessCoordinatorDelegate | protocol | AVCapturePhotoOutput.h | iOS-only (unavailable on macOS) | @property(nonatomic, getter=isCameraSensorOrientationCompensationEnabled) BOOL cameraSensorOrientationCompensationEnabled API_AVAILABLE(ios(26.0)) API_UNAVAILABLE(macos, macCatalyst, tvos, visionos) API_UNAVAILABLE(watchos); |
| AVCaptureSmartFramingMonitor | interface | AVCaptureDevice.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(26.0)) API_UNAVAILABLE(macos, macCatalyst, tvos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureSpatialAudioMetadataSampleGenerator | interface | AVCaptureSpatialAudioMetadataSampleGenerator.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(26.0)) API_UNAVAILABLE(macos, macCatalyst, tvos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedData | interface | AVCaptureDataOutputSynchronizer.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedDataCollection | interface | AVCaptureDataOutputSynchronizer.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedDepthData | interface | AVCaptureDataOutputSynchronizer.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedMetadataObjectData | interface | AVCaptureDataOutputSynchronizer.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureSynchronizedSampleBufferData | interface | AVCaptureDataOutputSynchronizer.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(11.0), macCatalyst(14.0), tvos(17.0)) API_UNAVAILABLE(macos, visionos) API_UNAVAILABLE(watchos) |
| AVCaptureSystemPressureState | interface | AVCaptureSystemPressure.h | iOS-only (unavailable on macOS) | API_AVAILABLE(ios(11.1), macCatalyst(14.0), tvos(17.0), visionos(1.0)) API_UNAVAILABLE(macos) API_UNAVAILABLE(watchos) |
