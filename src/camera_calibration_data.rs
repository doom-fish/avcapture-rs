#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

use core::ffi::{c_char, c_void};
use core::ptr;

use serde::{Deserialize, Serialize};

use crate::error::{from_swift, AVCaptureError};
use crate::ffi;
use crate::helpers::{parse_json_and_free, CapturePoint, CaptureSize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// Wraps the row-major values of `AVCameraCalibrationData.intrinsicMatrix`.
pub struct CameraIntrinsicMatrix {
    /// The row-major matrix values.
    pub rows: [[f32; 3]; 3],
}

impl CameraIntrinsicMatrix {
    #[must_use]
    /// Creates a new intrinsic matrix wrapper from row-major values.
    pub const fn new(rows: [[f32; 3]; 3]) -> Self {
        Self { rows }
    }

    #[must_use]
    /// Returns the row-major matrix values.
    pub const fn as_rows(&self) -> &[[f32; 3]; 3] {
        &self.rows
    }

    #[must_use]
    /// Returns the x-axis focal length in pixels.
    pub const fn focal_length_x(&self) -> f32 {
        self.rows[0][0]
    }

    #[must_use]
    /// Returns the y-axis focal length in pixels.
    pub const fn focal_length_y(&self) -> f32 {
        self.rows[1][1]
    }

    #[must_use]
    /// Returns the principal point in pixels.
    pub fn principal_point(&self) -> CapturePoint {
        CapturePoint::new(f64::from(self.rows[0][2]), f64::from(self.rows[1][2]))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// Wraps the row-major values of `AVCameraCalibrationData.extrinsicMatrix`.
pub struct CameraExtrinsicMatrix {
    /// The row-major matrix values.
    pub rows: [[f32; 4]; 3],
}

impl CameraExtrinsicMatrix {
    #[must_use]
    /// Creates a new extrinsic matrix wrapper from row-major values.
    pub const fn new(rows: [[f32; 4]; 3]) -> Self {
        Self { rows }
    }

    #[must_use]
    /// Returns the row-major matrix values.
    pub const fn as_rows(&self) -> &[[f32; 4]; 3] {
        &self.rows
    }

    #[must_use]
    /// Returns the unitless 3x3 rotation matrix.
    pub const fn rotation_matrix(&self) -> [[f32; 3]; 3] {
        [
            [self.rows[0][0], self.rows[0][1], self.rows[0][2]],
            [self.rows[1][0], self.rows[1][1], self.rows[1][2]],
            [self.rows[2][0], self.rows[2][1], self.rows[2][2]],
        ]
    }

    #[must_use]
    /// Returns the translation vector in millimeters.
    pub const fn translation_vector(&self) -> [f32; 3] {
        [self.rows[0][3], self.rows[1][3], self.rows[2][3]]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Snapshot of `AVCameraCalibrationData` state.
pub struct CameraCalibrationDataInfo {
    /// The intrinsic matrix reported by `AVCameraCalibrationData`.
    pub intrinsic_matrix: CameraIntrinsicMatrix,
    /// The reference dimensions reported by `AVCameraCalibrationData`.
    pub intrinsic_matrix_reference_dimensions: CaptureSize,
    /// The extrinsic matrix reported by `AVCameraCalibrationData`.
    pub extrinsic_matrix: CameraExtrinsicMatrix,
    /// The pixel size reported by `AVCameraCalibrationData`, in millimeters.
    pub pixel_size: f32,
    /// The lens distortion lookup table reported by `AVCameraCalibrationData`.
    pub lens_distortion_lookup_table: Option<Vec<f32>>,
    /// The inverse lens distortion lookup table reported by `AVCameraCalibrationData`.
    pub inverse_lens_distortion_lookup_table: Option<Vec<f32>>,
    /// The lens distortion center reported by `AVCameraCalibrationData`.
    pub lens_distortion_center: CapturePoint,
}

/// Safe wrapper around `AVCameraCalibrationData`.
#[derive(Debug)]
/// Wraps `AVCameraCalibrationData`.
pub struct CameraCalibrationData {
    pub(crate) ptr: *mut c_void,
}

impl CameraCalibrationData {
    #[must_use]
    /// Wraps an existing retained `AVCameraCalibrationData` pointer.
    pub const fn from_raw(ptr: *mut c_void) -> Self {
        Self { ptr }
    }

    /// Returns a snapshot of `AVCameraCalibrationData` state.
    pub fn info(&self) -> Result<CameraCalibrationDataInfo, AVCaptureError> {
        let mut err: *mut c_char = ptr::null_mut();
        let json_ptr = unsafe {
            ffi::camera_calibration_data::av_camera_calibration_data_info_json(self.ptr, &mut err)
        };
        if json_ptr.is_null() {
            return Err(unsafe { from_swift(ffi::status::OPERATION_FAILED, err) });
        }
        parse_json_and_free(json_ptr)
    }

    /// Corresponds to `AVCameraCalibrationData.intrinsicMatrix`.
    pub fn intrinsic_matrix(&self) -> Result<CameraIntrinsicMatrix, AVCaptureError> {
        Ok(self.info()?.intrinsic_matrix)
    }

    /// Corresponds to `AVCameraCalibrationData.intrinsicMatrixReferenceDimensions`.
    pub fn intrinsic_matrix_reference_dimensions(&self) -> Result<CaptureSize, AVCaptureError> {
        Ok(self.info()?.intrinsic_matrix_reference_dimensions)
    }

    /// Corresponds to `AVCameraCalibrationData.extrinsicMatrix`.
    pub fn extrinsic_matrix(&self) -> Result<CameraExtrinsicMatrix, AVCaptureError> {
        Ok(self.info()?.extrinsic_matrix)
    }

    /// Corresponds to `AVCameraCalibrationData.pixelSize`.
    pub fn pixel_size(&self) -> Result<f32, AVCaptureError> {
        Ok(self.info()?.pixel_size)
    }

    /// Corresponds to `AVCameraCalibrationData.lensDistortionLookupTable`.
    pub fn lens_distortion_lookup_table(&self) -> Result<Option<Vec<f32>>, AVCaptureError> {
        Ok(self.info()?.lens_distortion_lookup_table)
    }

    /// Corresponds to `AVCameraCalibrationData.inverseLensDistortionLookupTable`.
    pub fn inverse_lens_distortion_lookup_table(&self) -> Result<Option<Vec<f32>>, AVCaptureError> {
        Ok(self.info()?.inverse_lens_distortion_lookup_table)
    }

    /// Corresponds to `AVCameraCalibrationData.lensDistortionCenter`.
    pub fn lens_distortion_center(&self) -> Result<CapturePoint, AVCaptureError> {
        Ok(self.info()?.lens_distortion_center)
    }
}

impl Drop for CameraCalibrationData {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::camera_calibration_data::av_camera_calibration_data_release(self.ptr) };
            self.ptr = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CameraCalibrationDataInfo, CameraExtrinsicMatrix, CameraIntrinsicMatrix};

    fn assert_f32_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1.0e-6,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn intrinsic_matrix_helpers_surface_focal_lengths_and_principal_point() {
        let matrix =
            CameraIntrinsicMatrix::new([[640.0, 0.0, 320.0], [0.0, 480.0, 240.0], [0.0, 0.0, 1.0]]);

        assert_f32_eq(matrix.focal_length_x(), 640.0);
        assert_f32_eq(matrix.focal_length_y(), 480.0);

        let principal_point = matrix.principal_point();
        assert!((principal_point.x - 320.0).abs() < f64::EPSILON);
        assert!((principal_point.y - 240.0).abs() < f64::EPSILON);
    }

    #[test]
    fn extrinsic_matrix_helpers_surface_rotation_and_translation() {
        let matrix = CameraExtrinsicMatrix::new([
            [1.0, 0.0, 0.0, 10.0],
            [0.0, 0.0, -1.0, 20.0],
            [0.0, 1.0, 0.0, 30.0],
        ]);
        let rotation = matrix.rotation_matrix();
        let expected_rotation = [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]];
        for (actual_row, expected_row) in rotation.iter().zip(expected_rotation.iter()) {
            for (&actual, &expected) in actual_row.iter().zip(expected_row.iter()) {
                assert_f32_eq(actual, expected);
            }
        }

        let translation = matrix.translation_vector();
        let expected_translation = [10.0, 20.0, 30.0];
        for (&actual, &expected) in translation.iter().zip(expected_translation.iter()) {
            assert_f32_eq(actual, expected);
        }
    }

    #[test]
    fn calibration_info_deserializes_bridge_payload() {
        let info: CameraCalibrationDataInfo = serde_json::from_str(
            r#"{
                "intrinsicMatrix": {
                    "rows": [[900.0, 0.0, 320.0], [0.0, 901.0, 240.0], [0.0, 0.0, 1.0]]
                },
                "intrinsicMatrixReferenceDimensions": {
                    "width": 640.0,
                    "height": 480.0
                },
                "extrinsicMatrix": {
                    "rows": [[1.0, 0.0, 0.0, 11.0], [0.0, 1.0, 0.0, 12.0], [0.0, 0.0, 1.0, 13.0]]
                },
                "pixelSize": 0.0014,
                "lensDistortionLookupTable": [0.0, 0.01, 0.02],
                "inverseLensDistortionLookupTable": [0.0, -0.01, -0.02],
                "lensDistortionCenter": {
                    "x": 319.5,
                    "y": 239.5
                }
            }"#,
        )
        .expect("bridge payload should decode");

        assert_f32_eq(info.intrinsic_matrix.focal_length_x(), 900.0);
        assert_f32_eq(info.intrinsic_matrix.focal_length_y(), 901.0);
        assert_f32_eq(info.pixel_size, 0.0014);
        assert_eq!(
            info.lens_distortion_lookup_table.as_deref(),
            Some(&[0.0, 0.01, 0.02][..])
        );
        assert_eq!(
            info.inverse_lens_distortion_lookup_table.as_deref(),
            Some(&[0.0, -0.01, -0.02][..])
        );
        assert!((info.lens_distortion_center.x - 319.5).abs() < f64::EPSILON);
        assert!((info.lens_distortion_center.y - 239.5).abs() < f64::EPSILON);
    }
}
