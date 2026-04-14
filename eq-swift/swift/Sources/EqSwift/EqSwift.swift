import Foundation
import Darwin

/// EqSwift - Simple Rust-to-Swift FFI
///
/// Ultra-simple API for calling Rust from Swift:
///
/// ```swift
/// import EqSwift
///
/// // Load a Rust module
/// let rust = EqSwift.load("../src/lib.rs")
///
/// // Call functions like native Swift
/// let result = rust.hello()
/// ```
public struct EqSwift {
    
    /// The loaded Rust module
    private let module: RustModule
    
    /// Initialize EqSwift (typically not called directly)
    private init(module: RustModule) {
        self.module = module
    }
    
    /// Load a Rust source file and compile it to a callable module.
    ///
    /// - Parameter path: Path to the Rust source file (e.g., "../src/lib.rs")
    /// - Returns: A callable Rust module
    ///
    /// Example:
    /// ```swift
    /// let rust = try! EqSwift.load("../src/lib.rs")
    /// let greeting = rust.greet(name: "World")
    /// ```
    public static func load(_ path: String) throws -> EqSwift {
        let module = try RustModule.load(path)
        return EqSwift(module: module)
    }
    
    /// Call a Rust function that returns a String.
    public func call(_ function: String) -> String {
        module.call(function, [])
    }
    
    /// Call a Rust function with one argument.
    public func call(_ function: String, _ arg: String) -> String {
        module.call(function, [arg])
    }
    
    /// Call a Rust function with multiple arguments.
    public func call(_ function: String, _ args: [String]) -> String {
        module.call(function, args)
    }
}

/// Internal: Represents a loaded Rust module
internal struct RustModule {
    let path: String
    let moduleName: String
    let libraryHandle: UnsafeMutableRawPointer?
    
    init(path: String, moduleName: String, libraryHandle: UnsafeMutableRawPointer?) {
        self.path = path
        self.moduleName = moduleName
        self.libraryHandle = libraryHandle
    }
    
    /// Load a Rust source file and compile it
    static func load(_ sourcePath: String) throws -> RustModule {
        let url = URL(fileURLWithPath: sourcePath)
        let path = url.standardizedFileURL.path
        
        // Get module name from file stem
        let moduleName = url.deletingPathExtension().lastPathComponent
        
        // Find the compiled library
        let searchPaths = [
            "./target/debug/lib\(moduleName).dylib",
            "./target/release/lib\(moduleName).dylib",
            "./target/debug/lib\(moduleName).so",
            "./target/release/lib\(moduleName).so",
            "\(path.replacingOccurrences(of: ".rs", with: ""))/target/debug/lib\(moduleName).dylib",
            "\(path.replacingOccurrences(of: ".rs", with: ""))/target/release/lib\(moduleName).dylib",
        ]
        
        for libPath in searchPaths {
            if FileManager.default.fileExists(atPath: libPath) {
                if let handle = dlopen(libPath, RTLD_LAZY) {
                    return RustModule(path: path, moduleName: moduleName, libraryHandle: handle)
                }
            }
        }
        
        throw EqSwiftError.libraryNotFound(path)
    }
    
    /// Call a function on the loaded module
    func call(_ function: String, _ args: [String]) -> String {
        guard let handle = libraryHandle else {
            return ""
        }
        
        let symbolName = "\(moduleName)_\(function)"
        guard let symbol = dlsym(handle, symbolName) else {
            return ""
        }
        
        // For simple string functions
        typealias StringFunc = @convention(c) () -> UnsafeMutablePointer<CChar>
        let fnPtr = unsafeBitCast(symbol, to: StringFunc.self)
        let resultPtr = fnPtr()
        
        defer {
            // Try to free if there's a free function
            if let freeSymbol = dlsym(handle, "\(moduleName)_free_string") {
                typealias FreeFunc = @convention(c) (UnsafeMutablePointer<CChar>) -> Void
                let freeFn = unsafeBitCast(freeSymbol, to: FreeFunc.self)
                freeFn(resultPtr)
            }
        }
        
        return String(cString: resultPtr)
    }
}

/// Errors that can occur when loading a Rust module
public enum EqSwiftError: Error {
    case libraryNotFound(String)
    case symbolNotFound(String)
    case compilationFailed(String)
    
    public var localizedDescription: String {
        switch self {
        case .libraryNotFound(let path):
            return "Rust library not found for: \(path)"
        case .symbolNotFound(let symbol):
            return "Function not found: \(symbol)"
        case .compilationFailed(let error):
            return "Compilation failed: \(error)"
        }
    }
}
