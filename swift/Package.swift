// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "EqSwift",
    platforms: [.macOS(.v14), .iOS(.v17)],
    products: [
        .library(name: "EqSwift", targets: ["EqSwift"])
    ],
    targets: [
        .target(
            name: "EqSwift",
            dependencies: [],
            path: "Sources/EqSwift",
            exclude: [],
            publicHeadersPath: "include"
        )
    ]
)
