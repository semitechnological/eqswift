// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "EqSwift",
    platforms: [.macOS(.v12), .iOS(.v15)],
    products: [
        .library(
            name: "EqSwift",
            targets: ["EqSwift"]
        ),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "EqSwift",
            dependencies: [],
            path: "Sources/EqSwift"
        ),
    ]
)
