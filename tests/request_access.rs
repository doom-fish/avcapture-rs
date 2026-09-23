mod common;

use std::time::Duration;

use avcapture::prelude::*;

#[test]
fn request_access_rejects_media_types_without_a_consent_prompt() {
    for media_type in [
        MediaType::Muxed,
        MediaType::Metadata,
        MediaType::Unknown("depth".to_owned()),
    ] {
        let result = CaptureDevice::request_access(&media_type, Duration::from_secs(1));
        assert!(
            matches!(result, Err(AVCaptureError::InvalidArgument(_))),
            "{media_type:?} must be rejected, got {result:?}"
        );
    }
}

#[test]
fn request_access_reports_an_existing_decision() -> common::TestResult {
    for media_type in [MediaType::Video, MediaType::Audio] {
        let status = CaptureDevice::authorization_status(&media_type)?;
        if status == AuthorizationStatus::NotDetermined {
            common::skip(
                "request_access",
                format!("{media_type:?} access is undecided and a request would prompt"),
            );
            continue;
        }
        let granted = CaptureDevice::request_access(&media_type, Duration::from_secs(10))?;
        assert_eq!(
            granted,
            status == AuthorizationStatus::Authorized,
            "{media_type:?} request disagrees with status {status:?}"
        );
    }
    Ok(())
}

#[cfg(feature = "async")]
mod async_request {
    use avcapture::async_api::RequestAccessFuture;
    use avcapture::prelude::*;

    #[test]
    fn request_access_future_rejects_media_types_without_a_consent_prompt() {
        let result = RequestAccessFuture::start(&MediaType::Metadata);
        assert!(matches!(result, Err(AVCaptureError::InvalidArgument(_))));
    }

    #[test]
    fn request_access_future_reports_an_existing_decision() -> super::common::TestResult {
        for media_type in [MediaType::Video, MediaType::Audio] {
            let status = CaptureDevice::authorization_status(&media_type)?;
            if status == AuthorizationStatus::NotDetermined {
                super::common::skip(
                    "request_access_future",
                    format!("{media_type:?} access is undecided and a request would prompt"),
                );
                continue;
            }
            let granted = pollster::block_on(RequestAccessFuture::start(&media_type)?);
            assert_eq!(granted, status == AuthorizationStatus::Authorized);
        }
        Ok(())
    }
}
