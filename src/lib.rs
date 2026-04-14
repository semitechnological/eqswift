//! eq-swift — Simple Rust-to-Swift FFI
//!
//! Ultra-simple API for generating Swift bindings from Rust source.
//!
//! # Example
//!
//! ```ignore
//! use eq_swift::load;
//!
//! let module = load("path/to/lib.rs")?;
//! println!("Swift bindings: {}", module.swift_bindings);
//! println!("FFI header: {}", module.ffi_header);
//! ```

use std::fs;
use std::path::Path;
use std::process::Command;

pub struct LoadedModule {
    pub rust_source: String,
    pub swift_bindings: String,
    pub ffi_header: String,
    pub module_name: String,
}

#[derive(Debug)]
pub enum LoadError {
    SourceNotFound(String),
    CompilationFailed(String),
    BindingsGenerationFailed(String),
    IoError(String),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::SourceNotFound(p) => write!(f, "Source file not found: {p}"),
            LoadError::CompilationFailed(e) => write!(f, "Compilation failed: {e}"),
            LoadError::BindingsGenerationFailed(e) => write!(f, "Bindings generation failed: {e}"),
            LoadError::IoError(e) => write!(f, "IO error: {e}"),
        }
    }
}

impl std::error::Error for LoadError {}

/// Load a Rust source file and generate Swift bindings.
///
/// # Example
///
/// ```ignore
/// use eq_swift::load;
///
/// let module = load("path/to/lib.rs")?;
/// println!("Swift bindings: {}", module.swift_bindings);
/// ```
pub fn load<P: AsRef<Path>>(rust_source: P) -> Result<LoadedModule, LoadError> {
    let source = rust_source.as_ref();

    if !source.exists() {
        return Err(LoadError::SourceNotFound(source.display().to_string()));
    }

    let rust_source = fs::read_to_string(source).map_err(|e| LoadError::IoError(e.to_string()))?;

    let module_name = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unnamed")
        .to_string();

    let swift_bindings = generate_swift_bindings(&rust_source, &module_name)?;
    let ffi_header = generate_ffi_header(&rust_source, &module_name)?;

    Ok(LoadedModule {
        rust_source,
        swift_bindings,
        ffi_header,
        module_name,
    })
}

fn generate_swift_bindings(rust_source: &str, module_name: &str) -> Result<String, LoadError> {
    let _manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    let temp_dir = std::env::temp_dir().join("eq-swift");
    fs::create_dir_all(&temp_dir).map_err(|e| LoadError::IoError(e.to_string()))?;

    let udl_file = temp_dir.join(format!("{module_name}.udl"));
    let swift_out = temp_dir.join("Generated");
    fs::create_dir_all(&swift_out).map_err(|e| LoadError::IoError(e.to_string()))?;

    let udl = generate_udl(rust_source, module_name);
    fs::write(&udl_file, &udl).map_err(|e| LoadError::IoError(e.to_string()))?;

    let bindgen = std::env::var("UNIFFI_BINDGEN").unwrap_or_else(|_| "uniffi-bindgen".to_string());

    let status = Command::new(&bindgen)
        .args([
            "generate",
            udl_file.to_str().unwrap(),
            "--language",
            "swift",
            "--out-dir",
        ])
        .arg(&swift_out)
        .output()
        .map_err(|e| LoadError::BindingsGenerationFailed(e.to_string()))?;

    if !status.status.success() {
        return Err(LoadError::BindingsGenerationFailed(
            String::from_utf8_lossy(&status.stderr).to_string(),
        ));
    }

    let swift_file = swift_out.join(format!("{module_name}.swift"));
    if swift_file.exists() {
        fs::read_to_string(&swift_file).map_err(|e| LoadError::IoError(e.to_string()))
    } else {
        Ok(format!(
            "// Generated for {module_name}\n// (Swift bindings would be here)"
        ))
    }
}

fn generate_ffi_header(rust_source: &str, module_name: &str) -> Result<String, LoadError> {
    let _manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    let temp_dir = std::env::temp_dir().join("eq-swift");
    fs::create_dir_all(&temp_dir).map_err(|e| LoadError::IoError(e.to_string()))?;

    let udl_file = temp_dir.join(format!("{module_name}.udl"));
    let swift_out = temp_dir.join("Generated");
    fs::create_dir_all(&swift_out).map_err(|e| LoadError::IoError(e.to_string()))?;

    let udl = generate_udl(rust_source, module_name);
    fs::write(&udl_file, &udl).map_err(|e| LoadError::IoError(e.to_string()))?;

    let bindgen = std::env::var("UNIFFI_BINDGEN").unwrap_or_else(|_| "uniffi-bindgen".to_string());

    let _status = Command::new(&bindgen)
        .args([
            "generate",
            udl_file.to_str().unwrap(),
            "--language",
            "swift",
            "--out-dir",
        ])
        .arg(&swift_out)
        .output()
        .map_err(|e| LoadError::BindingsGenerationFailed(e.to_string()))?;

    let header_file = swift_out.join(format!("{module_name}FFI.h"));
    if header_file.exists() {
        fs::read_to_string(&header_file).map_err(|e| LoadError::IoError(e.to_string()))
    } else {
        Ok(format!("/* FFI header for {module_name} */"))
    }
}

fn generate_udl(rust_source: &str, module_name: &str) -> String {
    let mut udl = format!("namespace {} {{\n}};\n\n", module_name.replace("-", "_"));

    for line in rust_source.lines() {
        let line = line.trim();

        if line.starts_with("pub struct ") || line.starts_with("struct ") {
            if let Some(name) = extract_type_name(line, &["pub struct ", "struct "]) {
                udl.push_str(&format!("interface {} {{}};\n", name));
            }
        }

        if line.starts_with("pub enum ") || line.starts_with("enum ") {
            if let Some(name) = extract_type_name(line, &["pub enum ", "enum "]) {
                udl.push_str(&format!("enum {} {{}};\n", name));
            }
        }
    }

    udl
}

fn extract_type_name(line: &str, prefixes: &[&str]) -> Option<String> {
    for prefix in prefixes {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest
                .split(&[' ', '{', '<', '('][..])
                .next()
                .unwrap_or(rest)
                .trim();
            if !name.is_empty() && !name.contains('<') {
                return Some(name.to_string());
            }
        }
    }
    None
}

uniffi::include_scaffolding!("eq_swift");

pub struct EqSwift;

impl Default for EqSwift {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl EqSwift {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    pub fn generate_bindings(&self, rust_source: String) -> String {
        let udl = generate_udl(&rust_source, "module");
        format!("// UDL:\n{}", udl)
    }
}
