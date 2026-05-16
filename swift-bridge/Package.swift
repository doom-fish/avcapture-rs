// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "AVCaptureBridge",
    platforms: [
        .macOS(.v12)
    ],
    products: [
        .library(name: "AVCaptureBridge", type: .static, targets: ["AVCaptureBridge"])
    ],
    targets: [
        .target(
            name: "AVCaptureBridge",
            dependencies: [],
            path: "Sources/AVCaptureBridge",
            publicHeadersPath: "include"
        )
    ]
)
