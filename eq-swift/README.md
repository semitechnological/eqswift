# eq-swift

Rust-to-Swift FFI the same way `equilibrium::load()` works.

## Quick Start

```swift
import EqSwift

// Load a Rust module
let rust = try! EqSwift.load("../src/lib.rs")

// Call functions like native Swift
let result = rust.call("hello")
```

## Architecture

```
eq-swift/
├── Cargo.toml           # Rust library (compiles to .dylib)
├── src/
│   ├── lib.rs          # Your Rust code with #[uniffi::export]
│   └── eq_swift.udl    # UniFFI interface definition
└── swift/
    ├── Package.swift    # Swift package
    └── Sources/EqSwift/
        └── EqSwift.swift  # Simple load() API
```

## Usage

### 1. Write your Rust code

```rust
// src/lib.rs
uniffi::include_scaffolding!("lib");

#[uniffi::export]
pub fn hello() -> String {
    "Hello from Rust!".to_string()
}
```

### 2. Define the UDL interface

```idl
// src/lib.udl
namespace lib {};
interface lib {};
```

### 3. Build the Rust library

```bash
cargo build --manifest-path eq-swift/Cargo.toml
```

### 4. Use from Swift

```swift
import EqSwift

let rust = try! EqSwift.load("../path/to/lib.rs")
let greeting = rust.call("hello")  // "Hello from Rust!"
```

## API

### EqSwift.load(path: String)

Loads a Rust module from a path.

```swift
let rust = try! EqSwift.load("../src/lib.rs")
```

### rust.call(function: String) -> String

Calls a Rust function that returns a String.

```swift
let result = rust.call("get_greeting")
```

### rust.call(function: String, arg: String) -> String

Calls a Rust function with one String argument.

```swift
let result = rust.call("greet", "World")
```

## Build

```bash
# Build Rust library
cargo build --manifest-path eq-swift/Cargo.toml

# Build Swift package
cd eq-swift/swift && swift build
```

## Generated Bindings

UniFFI generates:
- Swift types from Rust structs/enums
- Native Swift API from `#[uniffi::export]` macros
- FFI bridging code automatically
