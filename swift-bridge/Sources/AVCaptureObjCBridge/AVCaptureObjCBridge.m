#import "AVCaptureObjCBridge.h"

static BOOL AVCStoreException(NSException *exception, NSError * _Nullable * _Nullable error) {
    if (error != NULL) {
        NSString *reason = exception.reason ?: exception.name;
        *error = [NSError errorWithDomain:exception.name
                                     code:1
                                 userInfo:@{NSLocalizedDescriptionKey: reason}];
    }
    return NO;
}

BOOL AVCTrySetActiveFormat(
    AVCaptureDevice *device,
    AVCaptureDeviceFormat *format,
    NSError * _Nullable * _Nullable error
) {
    @try {
        device.activeFormat = format;
        return YES;
    } @catch (NSException *exception) {
        return AVCStoreException(exception, error);
    }
}

BOOL AVCTrySetActiveVideoMinFrameDuration(
    AVCaptureDevice *device,
    CMTime duration,
    NSError * _Nullable * _Nullable error
) {
    @try {
        device.activeVideoMinFrameDuration = duration;
        return YES;
    } @catch (NSException *exception) {
        return AVCStoreException(exception, error);
    }
}

BOOL AVCTrySetActiveVideoMaxFrameDuration(
    AVCaptureDevice *device,
    CMTime duration,
    NSError * _Nullable * _Nullable error
) {
    @try {
        device.activeVideoMaxFrameDuration = duration;
        return YES;
    } @catch (NSException *exception) {
        return AVCStoreException(exception, error);
    }
}

BOOL AVCTryCapturePhoto(
    AVCapturePhotoOutput *output,
    AVCapturePhotoSettings *settings,
    id<AVCapturePhotoCaptureDelegate> delegate,
    NSError * _Nullable * _Nullable error
) {
    @try {
        [output capturePhotoWithSettings:settings delegate:delegate];
        return YES;
    } @catch (NSException *exception) {
        return AVCStoreException(exception, error);
    }
}

BOOL AVCTryStartTrackingCaptureRequest(
    AVCapturePhotoOutputReadinessCoordinator *coordinator,
    AVCapturePhotoSettings *settings,
    NSError * _Nullable * _Nullable error
) {
    @try {
        [coordinator startTrackingCaptureRequestUsingPhotoSettings:settings];
        return YES;
    } @catch (NSException *exception) {
        return AVCStoreException(exception, error);
    }
}
