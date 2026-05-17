import AVFoundation
import CoreMedia
import Foundation

private struct CaptureOutputInfoSnapshot: Codable {
    let connectionCount: Int
    let deferredStartSupported: Bool?
    let deferredStartEnabled: Bool?
}

func avcDroppedSampleReason(from sampleBuffer: CMSampleBuffer) -> String? {
    guard let attachments = CMCopyDictionaryOfAttachments(
        allocator: kCFAllocatorDefault,
        target: sampleBuffer,
        attachmentMode: kCMAttachmentMode_ShouldPropagate
    ) as? [CFString: Any],
    let rawReason = attachments[kCMSampleBufferAttachmentKey_DroppedFrameReason] as? String
    else {
        return nil
    }

    if rawReason == (kCMSampleBufferDroppedFrameReason_FrameWasLate as String) {
        return "lateData"
    }
    if rawReason == (kCMSampleBufferDroppedFrameReason_OutOfBuffers as String) {
        return "outOfBuffers"
    }
    if rawReason == (kCMSampleBufferDroppedFrameReason_Discontinuity as String) {
        return "discontinuity"
    }
    return rawReason
}

private func avcOutputInfoPayload(from output: AVCaptureOutput) -> CaptureOutputInfoSnapshot {
    let deferredStartSupported: Bool?
    let deferredStartEnabled: Bool?
    if #available(macOS 26.0, *) {
        let supported = output.isDeferredStartSupported
        deferredStartSupported = supported
        deferredStartEnabled = supported ? output.isDeferredStartEnabled : nil
    } else {
        deferredStartSupported = nil
        deferredStartEnabled = nil
    }
    return CaptureOutputInfoSnapshot(
        connectionCount: output.connections.count,
        deferredStartSupported: deferredStartSupported,
        deferredStartEnabled: deferredStartEnabled
    )
}

@_cdecl("av_capture_output_info_json")
public func av_capture_output_info_json(
    _ outputPtr: UnsafeMutableRawPointer,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutablePointer<CChar>? {
    let output = avcOutputBox(outputPtr).output
    do {
        return ffiString(try avcEncodeJSON(avcOutputInfoPayload(from: output)))
    } catch {
        outErrorMessage?.pointee = ffiString(error.localizedDescription)
        return nil
    }
}

@_cdecl("av_capture_output_connections_count")
public func av_capture_output_connections_count(_ outputPtr: UnsafeMutableRawPointer) -> Int {
    avcOutputBox(outputPtr).output.connections.count
}

@_cdecl("av_capture_output_connection_at_index")
public func av_capture_output_connection_at_index(
    _ outputPtr: UnsafeMutableRawPointer,
    _ index: Int,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let output = avcOutputBox(outputPtr).output
    guard index >= 0, index < output.connections.count else {
        outErrorMessage?.pointee = ffiString("output connection index out of range")
        return nil
    }
    return avcRetain(ConnectionBox(output.connections[index]))
}

@_cdecl("av_capture_output_connection_for_media_type")
public func av_capture_output_connection_for_media_type(
    _ outputPtr: UnsafeMutableRawPointer,
    _ mediaTypePtr: UnsafePointer<CChar>,
    _ outErrorMessage: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> UnsafeMutableRawPointer? {
    let output = avcOutputBox(outputPtr).output
    let raw = String(cString: mediaTypePtr)
    guard let mediaType = avcDecodeMediaType(raw) else {
        outErrorMessage?.pointee = ffiString("unsupported media type: \(raw)")
        return nil
    }
    guard let connection = output.connection(with: mediaType) else {
        return nil
    }
    return avcRetain(ConnectionBox(connection))
}
