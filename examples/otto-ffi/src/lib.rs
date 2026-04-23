//! Otto-style AI autocomplete backend, exported through eqswift.
//!
//! This example demonstrates how eqswift replaces manual C FFI
//! (`#[no_mangle] pub unsafe extern "C" fn`, `*const c_char`, `CString`)
//! with simple annotations.
//!
//! # Build
//! ```bash
//! cargo build
//! ```
//!
//! # Generate Swift bindings
//! ```bash
//! cargo eqswift swift --out-dir swift
//! ```

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// One-time setup for the UniFFI proc-macro system.
// (Inside the eqswift crate itself this is auto-emitted by the first
// `#[eqswift::export]`, but in downstream crates you must call it explicitly.)
eqswift::setup!();

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Information about an AI model.
#[derive(eqswift::Record, Clone)]
pub struct ModelInfo {
    pub name: String,
    pub repo_id: String,
    pub size_gb: f64,
    pub params: u32,
    pub quantization: String,
    pub is_downloaded: bool,
}

/// A completion suggestion for a given prefix.
#[derive(eqswift::Record)]
pub struct Completion {
    pub prefix: String,
    pub suggestion: String,
    pub confidence: f32,
    pub is_ml_based: bool,
}

/// Result of a grammar check.
#[derive(eqswift::Record)]
pub struct GrammarCheck {
    pub original: String,
    pub corrected: String,
    pub suggestions: Vec<String>,
    pub confidence: f32,
}

/// Result of a code reshape operation.
#[derive(eqswift::Record)]
pub struct CodeReshape {
    pub original: String,
    pub reshaped: String,
    pub operation: String,
    pub confidence: f32,
}

// ---------------------------------------------------------------------------
// Model manager (object with methods)
// ---------------------------------------------------------------------------

static MODELS: OnceLock<Mutex<HashMap<String, ModelInfo>>> = OnceLock::new();
static CURRENT_MODEL: OnceLock<Mutex<String>> = OnceLock::new();

#[derive(eqswift::Object)]
pub struct ModelManager;

#[eqswift::export]
impl ModelManager {
    /// Default constructor.
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    /// List all available models and their download status.
    pub fn list_models(&self) -> Vec<ModelInfo> {
        let models = MODELS.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert(
                "zeta-2".to_string(),
                ModelInfo {
                    name: "zeta-2".to_string(),
                    repo_id: "NexVeridian/zeta-2-4bit".to_string(),
                    size_gb: 0.6,
                    params: 1_000_000_000,
                    quantization: "4bit".to_string(),
                    is_downloaded: false,
                },
            );
            m.insert(
                "qwen-3.5".to_string(),
                ModelInfo {
                    name: "qwen-3.5".to_string(),
                    repo_id: "mlx-community/Qwen3.5-0.8B-OptiQ-4bit".to_string(),
                    size_gb: 0.5,
                    params: 800_000_000,
                    quantization: "4bit".to_string(),
                    is_downloaded: false,
                },
            );
            m.insert(
                "gemma-4".to_string(),
                ModelInfo {
                    name: "gemma-4".to_string(),
                    repo_id: "mlx-community/gemma-4-e2b-it-4bit".to_string(),
                    size_gb: 0.7,
                    params: 1_000_000_000,
                    quantization: "4bit".to_string(),
                    is_downloaded: false,
                },
            );
            Mutex::new(m)
        });
        models.lock().unwrap().values().cloned().collect()
    }

    /// Get info for a specific model.
    pub fn get_model(&self, name: String) -> Option<ModelInfo> {
        let models = MODELS.get()?;
        models.lock().unwrap().get(&name).cloned()
    }

    /// Mark a model as downloaded.
    pub fn mark_downloaded(&self, name: String) -> bool {
        let models = match MODELS.get() {
            Some(m) => m,
            None => return false,
        };
        if let Some(model) = models.lock().unwrap().get_mut(&name) {
            model.is_downloaded = true;
            true
        } else {
            false
        }
    }

    /// Set the current active model.
    pub fn set_current_model(&self, name: String) -> bool {
        let models = match MODELS.get() {
            Some(m) => m,
            None => return false,
        };
        if !models.lock().unwrap().contains_key(&name) {
            return false;
        }
        let current = CURRENT_MODEL.get_or_init(|| Mutex::new("zeta-2".to_string()));
        *current.lock().unwrap() = name;
        true
    }

    /// Get the name of the currently active model.
    pub fn current_model(&self) -> String {
        let current = CURRENT_MODEL.get_or_init(|| Mutex::new("zeta-2".to_string()));
        current.lock().unwrap().clone()
    }
}

// ---------------------------------------------------------------------------
// Completion engine
// ---------------------------------------------------------------------------

static COMPLETIONS: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();

/// Initialize the completion dictionary.
#[eqswift::export]
pub fn initialize_completions() {
    COMPLETIONS.get_or_init(|| {
        let mut map = HashMap::new();
        map.insert(
            "hel".to_string(),
            vec!["hello".to_string(), "help".to_string(), "helm".to_string()],
        );
        map.insert(
            "wor".to_string(),
            vec!["world".to_string(), "work".to_string(), "worth".to_string()],
        );
        map.insert(
            "fun".to_string(),
            vec!["function".to_string(), "funny".to_string(), "fund".to_string()],
        );
        map.insert(
            "imp".to_string(),
            vec!["import".to_string(), "implement".to_string(), "imply".to_string()],
        );
        map.insert(
            "str".to_string(),
            vec!["string".to_string(), "struct".to_string(), "stream".to_string()],
        );
        map
    });
}

/// Get a completion suggestion for the given text prefix.
#[eqswift::export]
pub fn get_completion(text: String) -> Option<Completion> {
    let dict = COMPLETIONS.get()?;

    // Try exact prefix match
    if let Some(suggestions) = dict.get(&text) {
        return Some(Completion {
            prefix: text.clone(),
            suggestion: suggestions.first()?.clone(),
            confidence: 0.95,
            is_ml_based: false,
        });
    }

    // Try fuzzy prefix match
    for (prefix, suggestions) in dict.iter() {
        if text.starts_with(prefix) || prefix.starts_with(&text) {
            return Some(Completion {
                prefix: text.clone(),
                suggestion: suggestions.first()?.clone(),
                confidence: 0.75,
                is_ml_based: false,
            });
        }
    }

    None
}

/// Get all suggestions for a prefix.
#[eqswift::export]
pub fn get_completion_suggestions(text: String) -> Vec<String> {
    let dict = match COMPLETIONS.get() {
        Some(d) => d,
        None => return Vec::new(),
    };
    dict.get(&text).cloned().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Grammar check
// ---------------------------------------------------------------------------

/// Check grammar in a piece of text and return corrections.
#[eqswift::export]
pub fn check_grammar(text: String) -> GrammarCheck {
    let corrections: HashMap<&str, &str> = [
        ("teh", "the"),
        ("recieve", "receive"),
        ("occured", "occurred"),
        ("seperate", "separate"),
        ("definately", "definitely"),
    ]
    .into_iter()
    .collect();

    let mut corrected = text.clone();
    let mut suggestions = Vec::new();

    for (wrong, right) in corrections.iter() {
        if corrected.contains(wrong) {
            corrected = corrected.replace(wrong, right);
            suggestions.push(format!("'{}' → '{}'", wrong, right));
        }
    }

    let confidence = if suggestions.is_empty() { 1.0 } else { 0.85 };

    GrammarCheck {
        original: text,
        corrected,
        suggestions,
        confidence,
    }
}

// ---------------------------------------------------------------------------
// Code reshape
// ---------------------------------------------------------------------------

/// Reshape code by applying a transformation.
#[eqswift::export]
pub fn reshape_code(code: String, operation: String) -> CodeReshape {
    let reshaped = match operation.as_str() {
        "uppercase" => code.to_uppercase(),
        "lowercase" => code.to_lowercase(),
        "snake_case" => code.replace(" ", "_").to_lowercase(),
        "camelCase" => to_camel_case(&code),
        "remove_comments" => remove_comments(&code),
        _ => code.clone(),
    };

    CodeReshape {
        original: code,
        reshaped,
        operation,
        confidence: 0.98,
    }
}

fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = false;
    for c in s.chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            capitalize = true;
        } else if capitalize {
            result.push(c.to_ascii_uppercase());
            capitalize = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn remove_comments(code: &str) -> String {
    code.lines()
        .map(|line| {
            if let Some(idx) = line.find("//") {
                &line[..idx]
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

/// Get the backend version.
#[eqswift::export]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
