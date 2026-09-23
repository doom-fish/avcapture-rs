#import <AVFoundation/AVFoundation.h>

NS_ASSUME_NONNULL_BEGIN

BOOL AVCTrySetActiveFormat(
    AVCaptureDevice *device,
    AVCaptureDeviceFormat *format,
    NSError * _Nullable * _Nullable error
);

BOOL AVCTrySetActiveVideoMinFrameDuration(
    AVCaptureDevice *device,
    CMTime duration,
    NSError * _Nullable * _Nullable error
);

BOOL AVCTrySetActiveVideoMaxFrameDuration(
    AVCaptureDevice *device,
    CMTime duration,
    NSError * _Nullable * _Nullable error
);

BOOL AVCTryCapturePhoto(
    AVCapturePhotoOutput *output,
    AVCapturePhotoSettings *settings,
    id<AVCapturePhotoCaptureDelegate> delegate,
    NSError * _Nullable * _Nullable error
);

BOOL AVCTryStartTrackingCaptureRequest(
    AVCapturePhotoOutputReadinessCoordinator *coordinator,
    AVCapturePhotoSettings *settings,
    NSError * _Nullable * _Nullable error
) API_AVAILABLE(macos(14.0));

NS_ASSUME_NONNULL_END
