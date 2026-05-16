import AVFoundation

func avcDecodeDevicePosition(_ raw: Int32) -> AVCaptureDevice.Position {
    AVCaptureDevice.Position(rawValue: Int(raw)) ?? .unspecified
}

func avcEncodeDevicePosition(_ position: AVCaptureDevice.Position) -> Int32 {
    Int32(position.rawValue)
}
