import AVFoundation
import CoreGraphics
import Foundation
import simd

private struct CapturePointPayload: Codable {
    let x: Double
    let y: Double

    init(_ point: CGPoint) {
        x = point.x
        y = point.y
    }
}

private struct CaptureSizePayload: Codable {
    let width: Double
    let height: Double

    init(_ size: CGSize) {
        width = size.width
        height = size.height
    }
}

private struct CameraIntrinsicMatrixPayload: Codable {
    let rows: [[Float]]

    init(_ matrix: matrix_float3x3) {
        rows = [
            [matrix[0][0], matrix[1][0], matrix[2][0]],
            [matrix[0][1], matrix[1][1], matrix[2][1]],
            [matrix[0][2], matrix[1][2], matrix[2][2]]
        ]
    }
}

private struct CameraExtrinsicMatrixPayload: Codable {
    let rows: [[Float]]

    init(_ matrix: matrix_float4x3) {
        rows = [
            [matrix[0][0], matrix[1][0], matrix[2][0], matrix[3][0]],
            [matrix[0][1], matrix[1][1], matrix[2][1], matrix[3][1]],
            [matrix[0][2], matrix[1][2], matrix[2][2], matrix[3][2]]
        ]
    }
}

private struct CameraCalibrationDataInfoPayload: Codable {
    let intrinsicMatrix: CameraIntrinsicMatrixPayload
    let intrinsicMatrixReferenceDimensions: CaptureSizePayload
    let extrinsicMatrix: CameraExtrinsicMatrixPayload
    let pixelSize: Float
    let lensDistortionLookupTable: [Float]?
    let inverseLensDistortionLookupTable: [Float]?
    let lensDistortionCenter: CapturePointPayload
}

final class CameraCalibrationDataBox: NSObject {
    let cameraCalibrationData: AVCameraCalibrationData

    init(_ cameraCalibrationData: AVCameraCalibrationData) {
        self.cameraCalibrationData = cameraCalibrationData
    }
}

private func avcCameraCalibrationDataBox(
    _ ptr: UnsafeMutableRawPointer
) -> CameraCalibrationDataBox {
    avcUnretained(ptr, as: CameraCalibrationDataBox.self)
}

private func avcFloatLookupTable(_ data: Data?) throws -> [Float]? {
    guard let data else {
        return nil
    }
    let floatStride = MemoryLayout<Float>.stride
    guard data.count % floatStride == 0 else {
        throw BridgeError.message("unexpected lookup table byte count: \(data.count)")
    }
    let count = data.count / floatStride
    var values = Array(repeating: Float.zero, count: count)
    let copied = values.withUnsafeMutableBytes { rawBuffer in
        data.copyBytes(to: rawBuffer.bindMemory(to: UInt8.self))
    }
    guard copied == data.count else {
        throw BridgeError.message("failed to copy lookup table bytes")
    }
    return values
}

private func cameraCalibrationDataInfoPayload(
    from cameraCalibrationData: AVCameraCalibrationData
) throws -> CameraCalibrationDataInfoPayload {
    CameraCalibrationDataInfoPayload(
        intrinsicMatrix: CameraIntrinsicMatrixPayload(cameraCalibrationData.intrinsicMatrix),
        intrinsicMatrixReferenceDimensions: CaptureSizePayload(
            cameraCalibrationData.intrinsicMatrixReferenceDimensions
        ),
        extrinsicMatrix: CameraExtrinsicMatrixPayload(cameraCalibrationData.extrinsicMatrix),
        pixelSize: cameraCalibrationData.pixelSize,
        lensDistortionLookupTable: try avcFloatLookupTable(
            cameraCalibrationData.lensDistortionLookupTable
        ),
        inverseLensDistortionLookupTable: try avcFloatLookupTable(
            cameraCalibrationData.inverseLensDistortionLookupTable
        ),
        lensDistortionCenter: CapturePointPayload(cameraCalibrationData.lensDistortionCenter)
    )
}

@_cdecl("av_camera_calibration_data_release")
public func av_camera_calibration_data_release(
    _ cameraCalibrationDataPtr: UnsafeMutableRawPointer?
) {
    avcRelease(cameraCalibrationDataPtr, as: CameraCalibrationDataBox.self)
}

@_cdecl("av_camera_calibration_data_info_json")
public func av_camera_calibration_data_info_json(
    _ cameraCalibrationDataPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let cameraCalibrationData = avcCameraCalibrationDataBox(cameraCalibrationDataPtr)
        .cameraCalibrationData
    do {
        return ffiString(
            try avcEncodeJSON(cameraCalibrationDataInfoPayload(from: cameraCalibrationData))
        )
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}
